// Off-screen rendering of each screen, in light and dark. The images are written to
// target/captures for visual review; the test checks that everything draws without error
// and that the buttons send the right messages.
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use iced::widget::image;
use iced::{Settings, Size};
use iced_test::simulator::Simulator;

use super::*;

const WINDOW: Size = Size::new(1000.0, 700.0);
// Fictitious folder of the sample images: nothing is read from or written to the disk
const SAMPLE_DIR: &str = "/photos";

fn settings() -> Settings {
    Settings {
        default_font: theme::FONT,
        fonts: vec![
            include_bytes!("../../assets/fonts/Inter-Regular.ttf")
                .as_slice()
                .into(),
            include_bytes!("../../assets/fonts/Inter-Medium.ttf")
                .as_slice()
                .into(),
            include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")
                .as_slice()
                .into(),
            lucide_icons::LUCIDE_FONT_BYTES.into(),
        ],
        ..Settings::default()
    }
}

fn captures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("captures")
}

fn capture(app: &App, name: &str) {
    let mut ui = Simulator::with_size(settings(), WINDOW, app.view());
    let snapshot = ui.snapshot(&app.theme()).expect("interface rendering");
    assert!(
        snapshot
            .matches_image(captures_dir().join(name))
            .expect("capture written")
    );
}

fn messages(app: &App, label: &str) -> Vec<Message> {
    let mut ui = Simulator::with_size(settings(), WINDOW, app.view());
    ui.click(label)
        .unwrap_or_else(|_| panic!("\"{label}\" not found"));
    ui.into_messages().collect()
}

fn app(dark_mode: bool) -> (App, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    Preferences {
        dark_mode,
        ..Preferences::default()
    }
    .save(&dir.path().join("preferences.json"))
    .unwrap();
    (App::with_data_dir(dir.path()), dir)
}

// Color gradient used as a thumbnail
fn thumbnail(hue: u8) -> image::Handle {
    let pixels = (0..THUMBNAIL_SIZE * THUMBNAIL_SIZE)
        .flat_map(|index| {
            let x = (index % THUMBNAIL_SIZE) as u8;
            let y = (index / THUMBNAIL_SIZE) as u8;
            [hue.wrapping_add(x), 120u8.wrapping_add(y), 255 - hue, 255]
        })
        .collect::<Vec<u8>>();
    image::Handle::from_rgba(THUMBNAIL_SIZE, THUMBNAIL_SIZE, pixels)
}

fn result(name: &str, new_size: u64, new_dimensions: (u32, u32)) -> ConversionResult {
    let original_size = 2_818_000;
    ConversionResult {
        output_path: PathBuf::from(format!("{SAMPLE_DIR}/{name}")),
        original_size,
        new_size,
        compression_ratio: (1.0 - new_size as f64 / original_size as f64) * 100.0,
        original_dimensions: (4000, 3000),
        new_dimensions,
    }
}

fn sample_images() -> Vec<ImageEntry> {
    let entry = |id, name: &str, size, dimensions, status| ImageEntry {
        id,
        name: name.to_owned(),
        path: PathBuf::from(format!("{SAMPLE_DIR}/{name}")),
        subdir: PathBuf::new(),
        size,
        dimensions,
        thumbnail: thumbnail((id * 60) as u8),
        status,
    };
    vec![
        entry(
            0,
            "beach-holiday.jpg",
            2_818_000,
            (4000, 3000),
            Status::Converted(result("beach-holiday.webp", 212_400, (1920, 1440))),
        ),
        entry(
            1,
            "logo-transparent.png",
            56_309,
            (2000, 2000),
            Status::Converted(result("logo-transparent.webp", 3_100_000, (1920, 1920))),
        ),
        entry(
            2,
            "mountain-panorama.tiff",
            48_120_000,
            (8000, 3000),
            Status::Converting,
        ),
        entry(
            3,
            "a-really-very-long-file-name-to-check-that-it-is-truncated-in-the-list.jpg",
            1_250_000,
            (3000, 4000),
            Status::Pending,
        ),
        entry(4, "icon.png", 4_096, (512, 512), Status::Pending),
        entry(
            5,
            "damaged-scan.gif",
            820_000,
            (1200, 900),
            Status::Failed(t!("error-decode", error = "unexpected end of file")),
        ),
    ]
}

fn fill_history(app: &mut App) {
    let conversions = [
        (
            "beach-holiday.jpg",
            "beach-holiday.webp",
            212_400,
            (1920, 1440),
        ),
        (
            "logo-transparent.png",
            "logo-transparent.webp",
            3_100_000,
            (4000, 3000),
        ),
        ("portrait.jpeg", "portrait.avif", 96_000, (4000, 3000)),
        (
            "screenshot.png",
            "screenshot-1.png",
            1_900_000,
            (2000, 1500),
        ),
    ];
    for (index, (source, output, new_size, new_dimensions)) in conversions.into_iter().enumerate() {
        let timestamp = Utc
            .with_ymd_and_hms(2026, 10, 8 + index as u32, 9 + index as u32, 5, 0)
            .unwrap();
        let item = HistoryItem::new(
            PathBuf::from(format!("{SAMPLE_DIR}/{source}")),
            &result(output, new_size, new_dimensions),
            timestamp,
        );
        app.history.add(item).unwrap();
    }
}

#[test]
fn renders_every_screen_in_light_and_dark() {
    let _ = std::fs::remove_dir_all(captures_dir());

    for (dark_mode, suffix) in [(false, "light"), (true, "dark")] {
        let (mut app, _dir) = app(dark_mode);
        capture(&app, &format!("1-empty-{suffix}"));

        app.images = sample_images();
        app.images[2].status = Status::Pending;
        app.settings.resize_enabled = true;
        app.settings.output_dir = Some(PathBuf::from(format!("{SAMPLE_DIR}/converted")));
        app.notice = Some(t!("notice-ignored-files", count = 2));
        capture(&app, &format!("2-list-{suffix}"));

        app.settings.resize_value = String::new();
        capture(&app, &format!("3-invalid-size-{suffix}"));

        app.settings.resize_value = DEFAULT_RESIZE_PIXELS.to_owned();
        app.images = sample_images();
        app.is_converting = true;
        app.batch = (2, 5);
        app.notice = None;
        capture(&app, &format!("4-converting-{suffix}"));

        app.is_converting = false;
        app.tab = Tab::History;
        capture(&app, &format!("5-empty-history-{suffix}"));

        fill_history(&mut app);
        capture(&app, &format!("6-history-{suffix}"));

        app.is_confirming_clear = true;
        capture(&app, &format!("7-confirmation-{suffix}"));

        app.is_confirming_clear = false;
        app.tab = Tab::Convert;
        app.available_update = Version::parse("2.1.0");
        capture(&app, &format!("8-update-{suffix}"));
    }
}

/// Texts change length from one language to another: the busiest screens are rendered in
/// each language to spot a text that is cut or overflows.
#[test]
fn renders_the_busiest_screens_in_every_language() {
    for language in Language::ALL {
        i18n::set_language(language);
        let code = language.code();
        let (mut app, _dir) = app(false);
        app.images = sample_images();
        app.settings.resize_enabled = true;
        app.notice = Some(t!("notice-ignored-files", count = 2));
        capture(&app, &format!("9-language-{code}-list"));

        app.notice = None;
        app.is_converting = true;
        app.batch = (2, 5);
        app.available_update = Version::parse("2.1.0");
        capture(&app, &format!("9-language-{code}-converting"));

        app.is_converting = false;
        app.tab = Tab::History;
        fill_history(&mut app);
        app.is_confirming_clear = true;
        capture(&app, &format!("9-language-{code}-history"));
    }
}

#[test]
fn buttons_send_the_expected_messages() {
    let (mut app, _dir) = app(false);
    app.images = sample_images();

    assert!(matches!(
        messages(&app, "Historique")[..],
        [Message::TabSelected(Tab::History)]
    ));
    assert!(matches!(
        messages(&app, "Convertir les images")[..],
        [Message::Convert]
    ));
    assert!(matches!(
        messages(&app, "Tout retirer")[..],
        [Message::RemoveAllImages]
    ));
    // The button is inside the clickable drop zone: a single picker must open
    assert!(matches!(
        messages(&app, "Choisir des images")[..],
        [Message::SelectFiles]
    ));

    fill_history(&mut app);
    app.tab = Tab::History;
    assert!(matches!(
        messages(&app, "Vider l'historique")[..],
        [Message::ClearHistory]
    ));

    app.is_confirming_clear = true;
    assert!(matches!(
        messages(&app, "Vider")[..],
        [Message::ClearHistoryConfirmed(true)]
    ));
    assert!(matches!(
        messages(&app, "Annuler")[..],
        [Message::ClearHistoryConfirmed(false)]
    ));
}

#[test]
fn update_banner_offers_download_and_dismissal() {
    let (mut app, _dir) = app(false);
    assert!(messages(&app, "Vérifier les mises à jour").len() == 1);

    app.available_update = Version::parse("2.1.0");

    assert!(matches!(
        messages(&app, "Télécharger")[..],
        [Message::DownloadUpdate]
    ));
    let mut ui = Simulator::with_size(settings(), WINDOW, app.view());
    assert!(ui.find("La version 2.1.0 est disponible").is_ok());
}

#[test]
fn convert_button_is_disabled_without_pending_images() {
    let (app, _dir) = app(false);

    assert!(messages(&app, "Convertir les images").is_empty());
}
