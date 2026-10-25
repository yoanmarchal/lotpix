// Ligne de commande : compression et redimensionnement sans ouvrir de fenêtre. Les entrées
// « Compresser avec Lotpix » et « Redimensionner avec Lotpix » du menu contextuel lancent
// `lotpix compress --format <format>` et `lotpix resize --size <taille>`, avec les réglages
// recommandés (src/presets.rs)
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::Utc;

use crate::batch::{self, Job};
use crate::conversion::{ConversionError, ConversionResult, FormatChoice, OutputFormat};
use crate::format::{format_file_size, format_percent};
use crate::history::{History, HistoryItem};
use crate::presets::{self, Preset, SIZES};
use crate::queue;
use crate::settings::Settings;
use crate::storage;

fn usage() -> String {
    let sizes: Vec<String> = SIZES
        .iter()
        .map(|size| format!("  {:<20} {}", size.id, size.label))
        .collect();
    format!(
        "\
Utilisation :
  lotpix [fichiers…]                       ouvre l'interface avec ces images
  lotpix open <fichiers ou dossiers…>      idem, en regroupant les lancements simultanés
  lotpix compress --format <format> <fichiers ou dossiers…>
                                           compresse avec la qualité recommandée
  lotpix resize --size <taille> <fichiers ou dossiers…>
                                           redimensionne avec la qualité recommandée
  lotpix compress <fichiers ou dossiers…>  compresse avec les réglages de l'interface

Les dossiers sont parcourus avec leurs sous-dossiers. Avec --format et --size, les
images sont écrites à côté des originales.

Formats : webp, jpg, png, avif, original (format d'origine)

Tailles (les formats des réseaux sociaux sont recadrés au centre et écrits en JPEG) :
{}",
        sizes.join("\n")
    )
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Ouvre l'interface avec ces fichiers ou dossiers.
    Open {
        paths: Vec<PathBuf>,
        gather: bool,
    },
    /// Convertit sans interface, avec un préréglage ou, à défaut, les réglages de l'interface.
    Convert {
        paths: Vec<PathBuf>,
        preset: Option<Preset>,
    },
    Help,
    /// Arguments incompris : le message à afficher.
    Invalid(String),
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
        Some(name @ ("compress" | "resize")) => {
            arguments.next();
            let option = if name == "compress" {
                "--format"
            } else {
                "--size"
            };
            let preset = if arguments.peek().is_some_and(|argument| argument == option) {
                arguments.next();
                let value = arguments.next().unwrap_or_default();
                let preset = value.to_str().and_then(|value| {
                    if name == "compress" {
                        parse_format(value).map(Preset::Compress)
                    } else {
                        presets::size(value).map(Preset::Resize)
                    }
                });
                match preset {
                    Some(preset) => Some(preset),
                    None => {
                        let kind = if name == "compress" {
                            "Format"
                        } else {
                            "Taille"
                        };
                        return Command::Invalid(format!(
                            "{kind} inconnu(e) : {}",
                            value.to_string_lossy()
                        ));
                    }
                }
            } else if name == "resize" {
                return Command::Invalid("Taille manquante : --size <taille>".to_owned());
            } else {
                None
            };
            Command::Convert {
                paths: absolute_paths(arguments),
                preset,
            }
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

fn parse_format(value: &str) -> Option<FormatChoice> {
    Some(match value.to_ascii_lowercase().as_str() {
        "original" => FormatChoice::Original,
        "webp" => FormatChoice::Fixed(OutputFormat::Webp),
        "jpg" | "jpeg" => FormatChoice::Fixed(OutputFormat::Jpg),
        "png" => FormatChoice::Fixed(OutputFormat::Png),
        "avif" => FormatChoice::Fixed(OutputFormat::Avif),
        _ => return None,
    })
}

fn absolute_paths(arguments: impl Iterator<Item = OsString>) -> Vec<PathBuf> {
    arguments
        .filter_map(|argument| std::path::absolute(argument).ok())
        .collect()
}

pub fn run(command: Command) -> ExitCode {
    match command {
        Command::Help => {
            println!("{}", usage());
            ExitCode::SUCCESS
        }
        Command::Invalid(message) => {
            eprintln!("{message}\n\n{}", usage());
            ExitCode::FAILURE
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
        Command::Convert { paths, preset } => {
            // Une file par entrée du menu : deux entrées choisies coup sur coup ne se mélangent pas
            let queue_name = preset.map_or_else(|| "compress".to_owned(), |preset| preset.id());
            let paths = gathered(paths, &queue_name);
            if paths.is_empty() {
                return ExitCode::SUCCESS;
            }
            let report = convert(&storage::data_dir(), &paths, preset);
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

#[derive(Debug)]
struct Report {
    title: String,
    body: String,
    /// Une ligne par image, pour le terminal.
    lines: Vec<String>,
    success: bool,
}

impl Report {
    fn failure(title: &str, body: &str) -> Self {
        Self {
            title: title.to_owned(),
            body: body.to_owned(),
            lines: Vec::new(),
            success: false,
        }
    }
}

fn convert(data_dir: &Path, paths: &[PathBuf], preset: Option<Preset>) -> Report {
    let (jobs, ignored) = batch::collect(paths);
    let results = match preset {
        Some(preset) => {
            if jobs.is_empty() {
                return no_images(ignored);
            }
            batch::convert_all_with(&jobs, preset.format(), |job: &Job| {
                preset.options(&job.source)
            })
        }
        None => {
            let settings = Settings::load(&data_dir.join("settings.json"));
            let Some(options) = settings.options() else {
                return Report::failure(
                    "Compression impossible",
                    "La taille de redimensionnement enregistrée est invalide : corrigez-la dans Lotpix",
                );
            };
            if jobs.is_empty() {
                return no_images(ignored);
            }
            batch::convert_all(&jobs, &options)
        }
    };
    let (singular, plural_form) = match preset {
        Some(Preset::Resize(_)) => ("image redimensionnée", "images redimensionnées"),
        _ => ("image compressée", "images compressées"),
    };
    let mut report = summarize(data_dir, &jobs, results, singular, plural_form);
    report.success &= ignored == 0;
    report
}

fn no_images(ignored: usize) -> Report {
    Report {
        title: "Aucune image à traiter".to_owned(),
        body: "Formats pris en charge : JPEG, PNG, GIF, TIFF et WebP".to_owned(),
        lines: Vec::new(),
        success: ignored == 0,
    }
}

/// Bilan du lot, et ajout des conversions réussies à l'historique.
fn summarize(
    data_dir: &Path,
    jobs: &[Job],
    results: Vec<Result<ConversionResult, ConversionError>>,
    singular: &str,
    plural_form: &str,
) -> Report {
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

    let mut title = plural(converted, singular, plural_form);
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
        success: failures.is_empty(),
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

    fn create_image(path: &Path, width: u32, height: u32) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        image::RgbImage::from_pixel(width, height, image::Rgb([0x33, 0x66, 0xcc]))
            .save(path)
            .unwrap();
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
            Command::Convert { paths, preset: None } if paths.len() == 2
        ));
        assert!(matches!(
            parse(arguments(&["compress", "--format", "JPEG", "a.png"])),
            Command::Convert {
                paths,
                preset: Some(Preset::Compress(FormatChoice::Fixed(OutputFormat::Jpg))),
            } if paths.len() == 1
        ));
        assert!(matches!(
            parse(arguments(&["resize", "--size", "story", "a.png"])),
            Command::Convert {
                preset: Some(Preset::Resize(size)),
                ..
            } if size.id == "story"
        ));
        assert!(matches!(
            parse(arguments(&["compress", "--format", "bmp", "a.png"])),
            Command::Invalid(_)
        ));
        assert!(matches!(
            parse(arguments(&["resize", "a.png"])),
            Command::Invalid(_)
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

    /// Chaque commande des menus contextuels (installeur Windows, Dolphin) doit être comprise
    /// par la ligne de commande, et la liste de choix des actions Nemo doit proposer chaque
    /// format et chaque taille.
    #[test]
    fn context_menu_commands_are_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let read = |path: PathBuf| std::fs::read_to_string(path).unwrap();
        let mut sources = vec![read(root.join("Cargo.toml"))];
        for entry in std::fs::read_dir(root.join("packaging/linux/dolphin")).unwrap() {
            sources.push(read(entry.unwrap().path()));
        }

        let mut checked = 0;
        for source in &sources {
            let words: Vec<&str> = source
                .split(|c: char| c.is_whitespace() || c == '"')
                .collect();
            for pair in words.windows(2) {
                if let ["--format" | "--size", _] = pair {
                    let command = if pair[0] == "--format" {
                        "compress"
                    } else {
                        "resize"
                    };
                    let parsed = parse(arguments(&[command, pair[0], pair[1], "a.png"]));
                    assert!(
                        matches!(
                            parsed,
                            Command::Convert {
                                preset: Some(_),
                                ..
                            }
                        ),
                        "{command} {} {} : {parsed:?}",
                        pair[0],
                        pair[1]
                    );
                    checked += 1;
                }
            }
        }
        // Formats et tailles de Windows et de Dolphin
        let menu_entries = 5 + SIZES.len();
        assert_eq!(checked, 2 * menu_entries, "{checked} commandes trouvées");

        let nemo = read(root.join("packaging/linux/nemo/lotpix-menu.sh"));
        let ids = ["webp", "jpg", "png", "avif", "original"]
            .into_iter()
            .chain(SIZES.iter().map(|size| size.id));
        for id in ids {
            assert!(
                nemo.lines()
                    .any(|line| line.trim_start().starts_with(&format!("{id} \""))),
                "{id} absent de la liste Nemo"
            );
        }
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

        let report = convert(dir.path(), &[dir.path().join("a.png")], None);

        assert_eq!(report.title, "Compression impossible");
        assert!(!report.success);
    }

    #[test]
    fn reports_a_selection_without_images() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("notes.txt");
        std::fs::write(&notes, "pas une image").unwrap();

        let report = convert(dir.path(), &[notes], None);

        assert_eq!(report.title, "Aucune image à traiter");
        assert!(!report.success);
    }

    #[test]
    fn compresses_a_folder_with_the_saved_settings() {
        let dir = tempfile::tempdir().unwrap();
        let photos = dir.path().join("photos").join("vacances");
        create_image(&photos.join("a.png"), 8, 8);
        let settings = Settings {
            format: FormatChoice::Fixed(OutputFormat::Jpg),
            ..Settings::default()
        };
        settings.save(&dir.path().join("settings.json")).unwrap();

        let report = convert(dir.path(), &[dir.path().join("photos")], None);

        assert_eq!(report.title, "1 image compressée");
        assert!(report.success);
        assert!(photos.join("a.jpg").is_file());
        let history = History::load(dir.path().join("history.json"));
        assert_eq!(history.items().len(), 1);
    }

    #[test]
    fn presets_ignore_the_saved_settings() {
        let dir = tempfile::tempdir().unwrap();
        let photo = dir.path().join("a.png");
        create_image(&photo, 8, 8);
        // Réglages de l'interface qui ne doivent pas s'appliquer
        let settings = Settings {
            output_dir: Some(dir.path().join("sortie")),
            resize_enabled: true,
            resize_value: "2".to_owned(),
            ..Settings::default()
        };
        settings.save(&dir.path().join("settings.json")).unwrap();

        let preset = Preset::Compress(FormatChoice::Fixed(OutputFormat::Jpg));
        let report = convert(dir.path(), &[photo], Some(preset));

        assert!(report.success);
        let output = dir.path().join("a.jpg");
        assert_eq!(image::image_dimensions(&output).unwrap(), (8, 8));
    }

    #[test]
    fn social_sizes_are_resized_then_cropped() {
        let dir = tempfile::tempdir().unwrap();
        let photo = dir.path().join("paysage.png");
        create_image(&photo, 3000, 2000);

        let preset = Preset::Resize(presets::size("instagram-portrait").unwrap());
        let report = convert(dir.path(), &[photo], Some(preset));

        assert_eq!(report.title, "1 image redimensionnée");
        let output = dir.path().join("paysage.jpg");
        assert_eq!(image::image_dimensions(&output).unwrap(), (1080, 1350));
    }

    #[test]
    fn small_images_are_cropped_without_upscaling() {
        let dir = tempfile::tempdir().unwrap();
        let photo = dir.path().join("petite.png");
        create_image(&photo, 600, 400);

        let preset = Preset::Resize(presets::size("instagram-square").unwrap());
        let report = convert(dir.path(), &[photo], Some(preset));

        assert!(report.success);
        let output = dir.path().join("petite.jpg");
        assert_eq!(image::image_dimensions(&output).unwrap(), (400, 400));
    }
}
