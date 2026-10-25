// Command line: compression and resizing without opening a window. The "Compress with
// Lotpix" and "Resize with Lotpix" entries of the context menu run
// `lotpix compress --format <format>` and `lotpix resize --size <size>`, with the recommended
// settings (src/presets.rs)
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::Utc;

use crate::batch::{self, Job};
use crate::conversion::{ConversionError, ConversionResult, FormatChoice, OutputFormat};
use crate::format::{format_file_size, format_percent};
use crate::history::{History, HistoryItem};
use crate::i18n;
use crate::preferences::Preferences;
use crate::presets::{self, Preset, SIZES, Submenu};
use crate::queue;
use crate::settings::Settings;
use crate::storage;
use crate::t;

fn usage() -> String {
    let files = t!("cli-files");
    let paths = t!("cli-files-or-folders");
    let format = t!("cli-format");
    let size = t!("cli-size");
    let commands = [
        (format!("lotpix [{files}]"), t!("cli-help-open")),
        (format!("lotpix open <{paths}>"), t!("cli-help-open-gather")),
        (
            format!("lotpix compress --format <{format}> <{paths}>"),
            t!("cli-help-compress-format"),
        ),
        (
            format!("lotpix resize --size <{size}> <{paths}>"),
            t!("cli-help-resize"),
        ),
        (
            format!("lotpix compress <{paths}>"),
            t!("cli-help-compress-saved"),
        ),
    ];
    let mut lines = vec![t!("cli-usage")];
    for (command, help) in commands {
        lines.push(format!("  {command}"));
        lines.push(format!("      {help}"));
    }
    lines.push(String::new());
    lines.push(t!("cli-help-folders"));
    lines.push(String::new());
    lines.push(format!(
        "{} webp, jpg, png, avif, original ({})",
        t!("cli-help-formats"),
        t!("cli-help-original-format")
    ));
    lines.push(String::new());
    lines.push(t!("cli-help-sizes"));
    lines.extend(
        SIZES
            .iter()
            .map(|size| format!("  {:<20} {}", size.id, size.label())),
    );
    lines.join("\n")
}

fn menu_choices(submenu: Submenu) -> String {
    let mut lines = vec![submenu.title(), submenu.prompt()];
    for (value, label) in submenu.entries() {
        lines.push(value.to_owned());
        lines.push(label);
    }
    lines.join("\n")
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Opens the interface with these files or folders.
    Open {
        paths: Vec<PathBuf>,
        gather: bool,
    },
    /// Converts without the interface, with a preset or, failing that, the interface settings.
    Convert {
        paths: Vec<PathBuf>,
        preset: Option<Preset>,
    },
    Help,
    /// Choices of a submenu, for the script of the Nemo actions (packaging/linux/nemo): title,
    /// prompt, then each value followed by its label, one line each.
    MenuChoices(Submenu),
    /// Arguments not understood: the message to display.
    Invalid(String),
}

/// Arguments without the program name. A relative path (run from a terminal) is resolved
/// from the current folder.
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
                        let value = value.to_string_lossy();
                        return Command::Invalid(if name == "compress" {
                            t!("cli-unknown-format", value = value.as_ref())
                        } else {
                            t!("cli-unknown-size", value = value.as_ref())
                        });
                    }
                }
            } else if name == "resize" {
                return Command::Invalid(t!("cli-missing-size"));
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
        Some("menu-choices") => {
            arguments.next();
            match arguments.next().as_ref().and_then(|kind| kind.to_str()) {
                Some("compress") => Command::MenuChoices(Submenu::Compress),
                Some("resize") => Command::MenuChoices(Submenu::Resize),
                _ => Command::Invalid("menu-choices compress|resize".to_owned()),
            }
        }
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
    // The interface, the notifications and the help follow the language chosen in the preferences
    let preferences = Preferences::load(&storage::data_dir().join("preferences.json"));
    i18n::set_language(preferences.language.resolve());
    match command {
        Command::Help => {
            println!("{}", usage());
            ExitCode::SUCCESS
        }
        Command::MenuChoices(submenu) => {
            println!("{}", menu_choices(submenu));
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
            // A launch grouped with another one has nothing to open
            if gather && paths.is_empty() {
                return ExitCode::SUCCESS;
            }
            match crate::ui::run(paths) {
                Ok(()) => ExitCode::SUCCESS,
                Err(_) => ExitCode::FAILURE,
            }
        }
        Command::Convert { paths, preset } => {
            // One queue per menu entry: two entries chosen one after the other do not get mixed
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

/// Only Windows Explorer starts one process per file: elsewhere, all the files arrive
/// together and there is nothing to wait for.
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
    /// One line per image, for the terminal.
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
                    &t!("report-impossible"),
                    &t!("report-invalid-saved-resize"),
                );
            };
            if jobs.is_empty() {
                return no_images(ignored);
            }
            batch::convert_all(&jobs, &options)
        }
    };
    let resized = matches!(preset, Some(Preset::Resize(_)));
    let mut report = summarize(data_dir, &jobs, results, resized);
    report.success &= ignored == 0;
    report
}

fn no_images(ignored: usize) -> Report {
    Report {
        title: t!("report-no-images"),
        body: t!("report-supported-formats"),
        lines: Vec::new(),
        success: ignored == 0,
    }
}

/// Summary of the batch, and successful conversions added to the history.
fn summarize(
    data_dir: &Path,
    jobs: &[Job],
    results: Vec<Result<ConversionResult, ConversionError>>,
    resized: bool,
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
                let failure = t!(
                    "report-failure",
                    name = name.as_str(),
                    error = error.to_string()
                );
                lines.push(failure.clone());
                failures.push(failure);
            }
        }
    }

    let mut title = if resized {
        t!("report-resized", count = converted)
    } else {
        t!("report-compressed", count = converted)
    };
    if !failures.is_empty() {
        title.push_str(&t!("report-failures", count = failures.len()));
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

/// A notification that fails (no notification service) prevents nothing: the images are
/// already converted.
fn notify(title: &str, body: &str) {
    let mut notification = notify_rust::Notification::new();
    notification.summary(title).body(body).appname("Lotpix");
    #[cfg(target_os = "linux")]
    notification.icon("lotpix");
    // Windows only shows the notification under the Lotpix name if the application is installed:
    // the identifier is the one of the Start menu shortcut created by the installer
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
            panic!("expected opening: {command:?}");
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

    /// Every entry of the context submenus, from which the Windows, Dolphin and Nemo
    /// menus are generated (src/context_menu.rs), must be understood by the command
    /// line.
    #[test]
    fn context_menu_entries_are_valid_commands() {
        for submenu in [Submenu::Compress, Submenu::Resize] {
            let entries = submenu.entries();
            assert!(!entries.is_empty());
            for (value, _) in entries {
                let mut command: Vec<&str> = submenu.option().split(' ').collect();
                command.extend([value, "a.png"]);
                let parsed = parse(arguments(&command));
                assert!(
                    matches!(
                        parsed,
                        Command::Convert {
                            preset: Some(_),
                            ..
                        }
                    ),
                    "{command:?}: {parsed:?}"
                );
            }
        }
        assert_eq!(Submenu::Resize.entries().len(), SIZES.len());
    }

    #[test]
    fn lists_menu_choices_for_the_nemo_script() {
        i18n::set_language(i18n::Language::En);

        let choices = menu_choices(Submenu::Compress);

        let lines: Vec<&str> = choices.lines().collect();
        assert_eq!(
            lines[..4],
            [
                "Compress with Lotpix",
                "Format, with the recommended quality:",
                "webp",
                "WebP"
            ]
        );
        assert_eq!(lines.len(), 2 + 2 * 5);
        assert_eq!(
            parse(arguments(&["menu-choices", "resize"])),
            Command::MenuChoices(Submenu::Resize)
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

        let report = convert(dir.path(), &[dir.path().join("a.png")], None);

        assert_eq!(report.title, "Compression impossible");
        assert!(!report.success);
    }

    #[test]
    fn reports_a_selection_without_images() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("notes.txt");
        std::fs::write(&notes, "not an image").unwrap();

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
        // Interface settings that must not apply
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

    #[test]
    fn reports_and_help_follow_the_current_language() {
        i18n::set_language(i18n::Language::De);
        let dir = tempfile::tempdir().unwrap();
        let photo = dir.path().join("a.png");
        create_image(&photo, 8, 8);
        let broken = dir.path().join("b.png");
        std::fs::write(&broken, "not an image").unwrap();

        let preset = Preset::Compress(FormatChoice::Fixed(OutputFormat::Jpg));
        let report = convert(dir.path(), &[photo, broken], Some(preset));

        assert_eq!(report.title, "1 Bild komprimiert, 1 Fehler");
        assert!(
            report.body.contains("b.png: Bild unlesbar"),
            "{}",
            report.body
        );

        i18n::set_language(i18n::Language::En);
        let help = usage();
        assert!(help.starts_with("Usage:"));
        assert!(help.contains("instagram-square     Instagram square (1080 × 1080)"));
        assert_eq!(
            parse(arguments(&["resize", "--size", "giant", "a.png"])),
            Command::Invalid("Unknown size: giant".to_owned())
        );
    }

    #[test]
    fn conversion_errors_follow_the_current_language() {
        use crate::conversion::ConversionError;

        let error = ConversionError::UnsupportedExtension(String::new());
        assert_eq!(error.to_string(), "Extension non supportée : (aucune)");

        i18n::set_language(i18n::Language::Es);
        assert_eq!(error.to_string(), "Extensión no compatible: (ninguna)");
        let error = ConversionError::TooLarge {
            width: 16_384,
            height: 1,
            format: "WebP",
            max: 16_383,
        };
        // Dimensions stay written without a thousands separator
        assert_eq!(
            error.to_string(),
            "La imagen mediría 16384 × 1 px: el formato WebP está limitado a 16383 px por lado"
        );
    }
}
