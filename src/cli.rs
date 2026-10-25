// Ligne de commande : `lotpix compress <fichiers ou dossiers…>` compresse sans ouvrir de
// fenêtre, avec les derniers réglages choisis dans l'interface. C'est ce que lance l'entrée
// « Compresser avec Lotpix » du menu contextuel
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::Utc;

use crate::batch;
use crate::format::{format_file_size, format_percent};
use crate::history::{History, HistoryItem};
use crate::queue;
use crate::settings::Settings;
use crate::storage;

const USAGE: &str = "\
Utilisation :
  lotpix [fichiers…]                    ouvre l'interface avec ces images
  lotpix open <fichiers ou dossiers…>   idem, en regroupant les lancements simultanés
  lotpix compress <fichiers ou dossiers…>
                                        compresse sans interface, avec les réglages
                                        enregistrés ; les dossiers sont parcourus avec
                                        leurs sous-dossiers";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Ouvre l'interface avec ces fichiers ou dossiers.
    Open {
        paths: Vec<PathBuf>,
        gather: bool,
    },
    Compress(Vec<PathBuf>),
    Help,
}

/// Arguments sans le nom du programme. Un chemin relatif (lancement depuis un terminal) est
/// résolu depuis le dossier courant.
pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Command {
    let mut arguments = arguments.into_iter().peekable();
    let command = arguments
        .peek()
        .and_then(|first| first.to_str())
        .map(str::to_owned);
    match command.as_deref() {
        Some("compress") => {
            arguments.next();
            Command::Compress(absolute_paths(arguments))
        }
        Some("open") => {
            arguments.next();
            Command::Open {
                paths: absolute_paths(arguments),
                gather: true,
            }
        }
        Some("-h" | "--help") => Command::Help,
        _ => Command::Open {
            paths: absolute_paths(arguments),
            gather: false,
        },
    }
}

fn absolute_paths(arguments: impl Iterator<Item = OsString>) -> Vec<PathBuf> {
    arguments
        .filter_map(|argument| std::path::absolute(argument).ok())
        .collect()
}

pub fn run(command: Command) -> ExitCode {
    match command {
        Command::Help => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Command::Open { paths, gather } => {
            let paths = if gather {
                gathered(paths, "open")
            } else {
                paths
            };
            // Un lancement regroupé avec un autre n'a rien à ouvrir
            if gather && paths.is_empty() {
                return ExitCode::SUCCESS;
            }
            match crate::ui::run(paths) {
                Ok(()) => ExitCode::SUCCESS,
                Err(_) => ExitCode::FAILURE,
            }
        }
        Command::Compress(paths) => {
            let paths = gathered(paths, "compress");
            if paths.is_empty() {
                return ExitCode::SUCCESS;
            }
            compress(&storage::data_dir(), &paths)
        }
    }
}

/// Seul l'Explorateur Windows lance un processus par fichier : ailleurs, tous les fichiers
/// arrivent ensemble et il n'y a rien à attendre.
fn gathered(paths: Vec<PathBuf>, queue_name: &str) -> Vec<PathBuf> {
    if !cfg!(windows) {
        return paths;
    }
    let dir = storage::data_dir().join("queue").join(queue_name);
    queue::gather(&dir, &paths).unwrap_or(paths)
}

fn compress(data_dir: &Path, paths: &[PathBuf]) -> ExitCode {
    let report = compress_with(data_dir, paths);
    for line in &report.lines {
        println!("{line}");
    }
    notify(&report.title, &report.body);
    if report.success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[derive(Debug)]
struct Report {
    title: String,
    body: String,
    /// Une ligne par image, pour le terminal.
    lines: Vec<String>,
    success: bool,
}

fn compress_with(data_dir: &Path, paths: &[PathBuf]) -> Report {
    let settings = Settings::load(&data_dir.join("settings.json"));
    let Some(options) = settings.options() else {
        return Report {
            title: "Compression impossible".to_owned(),
            body:
                "La taille de redimensionnement enregistrée est invalide : corrigez-la dans Lotpix"
                    .to_owned(),
            lines: Vec::new(),
            success: false,
        };
    };

    let (jobs, ignored) = batch::collect(paths);
    if jobs.is_empty() {
        return Report {
            title: "Aucune image à compresser".to_owned(),
            body: "Formats pris en charge : JPEG, PNG, GIF, TIFF et WebP".to_owned(),
            lines: Vec::new(),
            success: ignored == 0,
        };
    }

    let results = batch::convert_all(&jobs, &options);

    let mut history = History::load(data_dir.join("history.json"));
    let mut lines = Vec::new();
    let mut failures = Vec::new();
    let (mut original_size, mut new_size, mut converted) = (0, 0, 0);
    for (job, result) in jobs.iter().zip(results) {
        let name = file_name(&job.source);
        match result {
            Ok(result) => {
                lines.push(format!(
                    "{name} → {} ({})",
                    result.output_path.display(),
                    format_percent(-result.compression_ratio)
                ));
                original_size += result.original_size;
                new_size += result.new_size;
                converted += 1;
                let _ = history.add(HistoryItem::new(job.source.clone(), &result, Utc::now()));
            }
            Err(error) => {
                lines.push(format!("{name} : {error}"));
                failures.push(format!("{name} : {error}"));
            }
        }
    }

    let mut title = plural(converted, "image compressée", "images compressées");
    if !failures.is_empty() {
        title.push_str(&format!(", {}", plural(failures.len(), "échec", "échecs")));
    }
    let body = if converted > 0 {
        let ratio = (1.0 - new_size as f64 / original_size as f64) * 100.0;
        format!(
            "{} → {} ({})",
            format_file_size(original_size),
            format_file_size(new_size),
            format_percent(-ratio)
        )
    } else {
        String::new()
    };
    let body = match failures.first() {
        Some(failure) if body.is_empty() => failure.clone(),
        Some(failure) => format!("{body}\n{failure}"),
        None => body,
    };
    Report {
        title,
        body,
        lines,
        success: failures.is_empty() && ignored == 0,
    }
}

/// Une notification qui échoue (pas de service de notifications) n'empêche rien : les images
/// sont déjà converties.
fn notify(title: &str, body: &str) {
    let mut notification = notify_rust::Notification::new();
    notification.summary(title).body(body).appname("Lotpix");
    #[cfg(target_os = "linux")]
    notification.icon("lotpix");
    // Windows n'affiche la notification au nom de Lotpix que si l'application est installée :
    // l'identifiant est celui du raccourci du menu Démarrer créé par l'installeur
    #[cfg(windows)]
    if is_installed() {
        notification.app_id("com.yoanmarchal.lotpix");
    }
    let _ = notification.show();
}

#[cfg(windows)]
fn is_installed() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("uninstall.exe").is_file()))
        .unwrap_or(false)
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn plural(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count > 1 { plural } else { singular })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn plain_paths_open_the_interface() {
        let command = parse(arguments(&["photo.png"]));

        let Command::Open { paths, gather } = command else {
            panic!("ouverture attendue : {command:?}");
        };
        assert!(!gather);
        assert_eq!(paths.len(), 1);
        assert!(paths[0].is_absolute());
        assert!(paths[0].ends_with("photo.png"));
    }

    #[test]
    fn recognizes_subcommands() {
        assert!(matches!(
            parse(arguments(&["compress", "a.png", "dossier"])),
            Command::Compress(paths) if paths.len() == 2
        ));
        assert!(matches!(
            parse(arguments(&["open", "a.png"])),
            Command::Open { gather: true, .. }
        ));
        assert_eq!(parse(arguments(&["--help"])), Command::Help);
        assert_eq!(
            parse(arguments(&[])),
            Command::Open {
                paths: Vec::new(),
                gather: false
            }
        );
    }

    #[test]
    fn reports_an_invalid_saved_resize() {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings {
            resize_enabled: true,
            resize_value: String::new(),
            ..Settings::default()
        };
        settings.save(&dir.path().join("settings.json")).unwrap();

        let report = compress_with(dir.path(), &[dir.path().join("a.png")]);

        assert_eq!(report.title, "Compression impossible");
        assert!(!report.success);
    }

    #[test]
    fn reports_a_selection_without_images() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("notes.txt");
        std::fs::write(&notes, "pas une image").unwrap();

        let report = compress_with(dir.path(), &[notes]);

        assert_eq!(report.title, "Aucune image à compresser");
        assert!(!report.success);
    }

    #[test]
    fn compresses_a_folder_with_the_saved_settings() {
        let dir = tempfile::tempdir().unwrap();
        let photos = dir.path().join("photos").join("vacances");
        std::fs::create_dir_all(&photos).unwrap();
        image::RgbImage::from_pixel(8, 8, image::Rgb([0x33, 0x66, 0xcc]))
            .save(photos.join("a.png"))
            .unwrap();
        let settings = Settings {
            format: crate::conversion::FormatChoice::Fixed(crate::conversion::OutputFormat::Jpg),
            ..Settings::default()
        };
        settings.save(&dir.path().join("settings.json")).unwrap();

        let report = compress_with(dir.path(), &[dir.path().join("photos")]);

        assert_eq!(report.title, "1 image compressée");
        assert!(report.success);
        assert!(photos.join("a.jpg").is_file());
        let history = History::load(dir.path().join("history.json"));
        assert_eq!(history.items().len(), 1);
    }
}
