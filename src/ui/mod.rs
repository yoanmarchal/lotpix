// Iced interface: application state, messages and update
#[cfg(test)]
mod captures;
mod convert;
mod history;
mod theme;
mod widgets;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use chrono::Utc;
use iced::widget::{column, container, image};
use iced::{Element, Event, Length, Size, Subscription, Task, Theme, event, window};
use tokio::sync::Semaphore;

use crate::batch::{self, Job};
use crate::conversion::{
    self, ConversionResult, FormatChoice, INPUT_EXTENSIONS, OutputFormat, ResizeMode,
    THUMBNAIL_SIZE,
};
use crate::history::{History, HistoryItem};
use crate::i18n::{self, Language, LanguageChoice};
use crate::preferences::Preferences;
use crate::settings::{DEFAULT_RESIZE_PERCENT, DEFAULT_RESIZE_PIXELS, Settings};
use crate::storage;
use crate::t;
use crate::update::{self, Version};
use theme::Tokens;

// Each decoded image takes several tens of MB: the number of simultaneous reads is bounded
// when the user drops a whole folder
static LOAD_LIMIT: LazyLock<Semaphore> = LazyLock::new(|| {
    Semaphore::new(std::thread::available_parallelism().map_or(2, |count| count.get()))
});

/// Opens the window, with these images and folders already added.
pub fn run(paths: Vec<PathBuf>) -> iced::Result {
    iced::application(move || App::new(paths.clone()), App::update, App::view)
        .title("Lotpix")
        .subscription(App::subscription)
        .theme(App::theme)
        .font(include_bytes!("../../assets/fonts/Inter-Regular.ttf").as_slice())
        .font(include_bytes!("../../assets/fonts/Inter-Medium.ttf").as_slice())
        .font(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf").as_slice())
        .font(lucide_icons::LUCIDE_FONT_BYTES)
        .default_font(theme::FONT)
        .window(window::Settings {
            size: Size::new(1000.0, 700.0),
            min_size: Some(Size::new(800.0, 600.0)),
            icon: window::icon::from_file_data(
                include_bytes!("../../assets/icons/256x256.png"),
                None,
            )
            .ok(),
            ..window::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Convert,
    History,
}

type ImageId = u64;

#[derive(Debug, Clone)]
pub struct ImageEntry {
    id: ImageId,
    name: String,
    path: PathBuf,
    /// Subfolder recreated in the output folder (image found in an added folder).
    subdir: PathBuf,
    size: u64,
    dimensions: (u32, u32),
    thumbnail: image::Handle,
    status: Status,
}

#[derive(Debug, Clone)]
enum Status {
    Pending,
    Converting,
    Converted(ConversionResult),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    DarkModeToggled,
    LanguageSelected(LanguageChoice),
    FileHovered,
    HoverLeft,
    FileDropped(PathBuf),
    SelectFiles,
    FilesSelected(Vec<PathBuf>),
    /// Images found in the added folders.
    FoldersScanned(u64, Vec<Job>),
    ImageLoaded(u64, Option<ImageEntry>),
    RemoveImage(ImageId),
    RemoveAllImages,
    FormatSelected(FormatChoice),
    QualityChanged(u8),
    ResizeToggled(bool),
    ResizeModeSelected(ResizeMode),
    ResizeValueChanged(String),
    AllowUpscaleToggled(bool),
    SelectOutputDir,
    OutputDirSelected(Option<PathBuf>),
    OutputDirReset,
    Convert,
    Converted(ImageId, PathBuf, Result<ConversionResult, String>),
    OpenFile(PathBuf),
    ClearHistory,
    ClearHistoryConfirmed(bool),
    UpdateChecked(Option<Version>),
    UpdateCheckToggled(bool),
    DownloadUpdate,
    UpdateDismissed,
}

pub struct App {
    tab: Tab,
    images: Vec<ImageEntry>,
    next_id: ImageId,
    settings: Settings,
    settings_path: PathBuf,
    is_converting: bool,
    /// Processed images and images to process in the current batch.
    batch: (usize, usize),
    /// Conversions currently running.
    active_conversions: usize,
    /// Images being read (dimensions and thumbnail).
    loading: usize,
    /// Increases when the list is cleared: reads started before are then ignored.
    load_generation: u64,
    /// Files refused since the last addition.
    ignored: usize,
    is_hovered: bool,
    /// A file or folder picker is open: do not open a second one.
    is_dialog_open: bool,
    /// The History screen asks to confirm clearing.
    is_confirming_clear: bool,
    /// Status bar message (ignored files, file that cannot be opened…).
    notice: Option<String>,
    /// Newer version found at startup, until the banner is closed.
    available_update: Option<Version>,
    history: History,
    preferences: Preferences,
    preferences_path: PathBuf,
    tokens: Tokens,
    theme: Theme,
}

impl App {
    /// `paths`: files passed as arguments ("Open with", context menu, drop on the
    /// application icon).
    fn new(paths: Vec<PathBuf>) -> (Self, Task<Message>) {
        let mut app = Self::with_data_dir(&storage::data_dir());
        let load = app.load_images(paths);
        let check = app.check_for_update();
        (app, Task::batch([load, check]))
    }

    /// Queries GitHub in the background: startup does not wait for the answer, and an
    /// error or a missing network goes unnoticed.
    fn check_for_update(&self) -> Task<Message> {
        if !self.preferences.check_updates {
            return Task::none();
        }
        Task::perform(
            async {
                tokio::task::spawn_blocking(update::check)
                    .await
                    .ok()
                    .flatten()
            },
            Message::UpdateChecked,
        )
    }

    fn save_preferences(&mut self) {
        if self.preferences.save(&self.preferences_path).is_err() {
            self.notice = Some(t!("notice-preferences-not-saved"));
        }
    }

    fn with_data_dir(data_dir: &Path) -> Self {
        let preferences_path = data_dir.join("preferences.json");
        let preferences = Preferences::load(&preferences_path);
        let settings_path = data_dir.join("settings.json");
        Self {
            tab: Tab::Convert,
            images: Vec::new(),
            next_id: 0,
            settings: Settings::load(&settings_path),
            settings_path,
            is_converting: false,
            batch: (0, 0),
            active_conversions: 0,
            loading: 0,
            load_generation: 0,
            ignored: 0,
            is_hovered: false,
            is_dialog_open: false,
            is_confirming_clear: false,
            notice: None,
            available_update: None,
            history: History::load(data_dir.join("history.json")),
            preferences,
            preferences_path,
            tokens: theme::tokens(preferences.dark_mode),
            theme: theme::iced_theme(preferences.dark_mode),
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn pending_count(&self) -> usize {
        self.images
            .iter()
            .filter(|image| matches!(image.status, Status::Pending))
            .count()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.tab = tab;
                self.is_confirming_clear = false;
            }
            Message::DarkModeToggled => {
                self.preferences.dark_mode = !self.preferences.dark_mode;
                self.tokens = theme::tokens(self.preferences.dark_mode);
                self.theme = theme::iced_theme(self.preferences.dark_mode);
                self.save_preferences();
            }
            // The whole interface is redrawn in the new language, without restarting
            Message::LanguageSelected(choice) => {
                self.preferences.language = choice;
                i18n::set_language(choice.resolve());
                self.save_preferences();
            }

            Message::UpdateChecked(version) => {
                // The setting may have been turned off while the request was running
                if self.preferences.check_updates {
                    self.available_update = version;
                }
            }
            Message::UpdateCheckToggled(enabled) => {
                self.preferences.check_updates = enabled;
                self.save_preferences();
                if enabled {
                    return self.check_for_update();
                }
                self.available_update = None;
            }
            Message::DownloadUpdate => {
                if open::that_detached(update::DOWNLOAD_URL).is_err() {
                    self.notice = Some(t!("notice-download-page"));
                }
            }
            Message::UpdateDismissed => self.available_update = None,

            Message::FileHovered => self.is_hovered = !self.is_converting,
            Message::HoverLeft => self.is_hovered = false,
            Message::FileDropped(path) => {
                self.is_hovered = false;
                if !self.is_converting {
                    self.tab = Tab::Convert;
                    return self.load_images(vec![path]);
                }
            }
            Message::SelectFiles => {
                if !self.is_dialog_open && !self.is_converting {
                    self.is_dialog_open = true;
                    return Task::perform(pick_files(), Message::FilesSelected);
                }
            }
            Message::FilesSelected(paths) => {
                self.is_dialog_open = false;
                // The picker may have stayed open while a conversion started
                if !self.is_converting {
                    return self.load_images(paths);
                }
            }
            Message::FoldersScanned(generation, jobs) => {
                self.loading = self.loading.saturating_sub(1);
                if generation == self.load_generation {
                    return self.load_jobs(jobs);
                }
            }
            Message::ImageLoaded(generation, loaded) => {
                self.loading = self.loading.saturating_sub(1);
                match loaded {
                    // The list was cleared during the read: the image must not come back
                    _ if generation != self.load_generation => {}
                    Some(image) => {
                        // Reads finish out of order: the order of addition is kept
                        let position = self.images.partition_point(|other| other.id < image.id);
                        self.images.insert(position, image);
                    }
                    None => {
                        self.ignored += 1;
                        self.notice = Some(t!("notice-ignored-files", count = self.ignored));
                    }
                }
            }
            Message::RemoveImage(id) => self.images.retain(|image| image.id != id),
            Message::RemoveAllImages => {
                self.images.clear();
                self.load_generation += 1;
            }

            Message::FormatSelected(format) => {
                self.change_settings(|settings| settings.format = format);
            }
            Message::QualityChanged(quality) => {
                if !self.settings.is_lossless() {
                    self.change_settings(|settings| settings.quality = quality);
                }
            }
            Message::ResizeToggled(enabled) => {
                self.change_settings(|settings| settings.resize_enabled = enabled);
            }
            Message::ResizeModeSelected(mode) => self.change_settings(|settings| {
                // A value in pixels makes no sense as a percentage, and the other way round
                let was_percent = settings.resize_mode == ResizeMode::Percent;
                let is_percent = mode == ResizeMode::Percent;
                if was_percent != is_percent {
                    settings.resize_value = if is_percent {
                        DEFAULT_RESIZE_PERCENT
                    } else {
                        DEFAULT_RESIZE_PIXELS
                    }
                    .to_owned();
                }
                settings.resize_mode = mode;
            }),
            Message::ResizeValueChanged(value) => self.change_settings(|settings| {
                settings.resize_value =
                    value.chars().filter(char::is_ascii_digit).take(5).collect();
            }),
            Message::AllowUpscaleToggled(allow) => {
                self.change_settings(|settings| settings.allow_upscale = allow);
            }
            Message::SelectOutputDir => {
                if !self.is_dialog_open && !self.is_converting {
                    self.is_dialog_open = true;
                    return Task::perform(pick_folder(), Message::OutputDirSelected);
                }
            }
            Message::OutputDirSelected(dir) => {
                self.is_dialog_open = false;
                if dir.is_some() {
                    self.change_settings(|settings| settings.output_dir = dir);
                }
            }
            Message::OutputDirReset => self.change_settings(|settings| settings.output_dir = None),

            Message::Convert => {
                let pending = self.pending_count();
                // No conversion while images are loading: the batch must be complete
                let is_ready = !self.is_converting && self.loading == 0 && pending > 0;
                if is_ready && self.settings.options().is_some() {
                    self.is_converting = true;
                    self.batch = (0, pending);
                    self.notice = None;
                    return self.start_conversions();
                }
            }
            Message::Converted(id, original_path, result) => {
                self.batch.0 += 1;
                self.active_conversions = self.active_conversions.saturating_sub(1);
                let status = match result {
                    Ok(result) => {
                        let item = HistoryItem::new(original_path, &result, Utc::now());
                        if self.history.add(item).is_err() {
                            self.notice = Some(t!("notice-history-not-saved"));
                        }
                        Status::Converted(result)
                    }
                    Err(error) => Status::Failed(error),
                };
                // The image may have been removed from the list during its conversion
                if let Some(image) = self.images.iter_mut().find(|image| image.id == id) {
                    image.status = status;
                }
                return self.start_conversions();
            }

            Message::OpenFile(path) => self.open_file(&path),
            Message::ClearHistory => self.is_confirming_clear = true,
            Message::ClearHistoryConfirmed(confirmed) => {
                self.is_confirming_clear = false;
                if confirmed && self.history.clear().is_err() {
                    self.notice = Some(t!("notice-history-not-saved"));
                }
            }
        }
        Task::none()
    }

    /// Adds images; a folder brings the images of its whole tree.
    fn load_images(&mut self, paths: Vec<PathBuf>) -> Task<Message> {
        if self.loading == 0 {
            self.ignored = 0;
            self.notice = None;
        }
        let (folders, files): (Vec<_>, Vec<_>) = paths.into_iter().partition(|path| path.is_dir());
        let load = self.load_jobs(files.into_iter().map(Job::file).collect());
        if folders.is_empty() {
            return load;
        }
        // Walking a large tree must not freeze the window
        self.loading += 1;
        let generation = self.load_generation;
        let scan = Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    folders
                        .iter()
                        .flat_map(|folder| batch::find_images(folder))
                        .collect()
                })
                .await
                .unwrap_or_default()
            },
            move |jobs| Message::FoldersScanned(generation, jobs),
        );
        Task::batch([load, scan])
    }

    fn load_jobs(&mut self, jobs: Vec<Job>) -> Task<Message> {
        let generation = self.load_generation;
        Task::batch(jobs.into_iter().map(|job| {
            let id = self.next_id;
            self.next_id += 1;
            self.loading += 1;
            Task::perform(load_image(id, job), move |loaded| {
                Message::ImageLoaded(generation, loaded)
            })
        }))
    }

    /// After a settings change, the images already processed are to be converted again.
    fn change_settings(&mut self, change: impl FnOnce(&mut Settings)) {
        if self.is_converting {
            return;
        }
        change(&mut self.settings);
        if self.settings.save(&self.settings_path).is_err() {
            self.notice = Some(t!("notice-settings-not-saved"));
        }
        for image in &mut self.images {
            if matches!(image.status, Status::Converted(_) | Status::Failed(_)) {
                image.status = Status::Pending;
            }
        }
    }

    /// Starts the conversions of the pending images, within the limit of `conversion::parallel_conversions`.
    /// The batch is done when nothing is pending or running anymore.
    fn start_conversions(&mut self) -> Task<Message> {
        let Some(options) = self.settings.options() else {
            self.is_converting = false;
            return Task::none();
        };
        let slots = conversion::parallel_conversions(self.settings.format);

        let mut tasks = Vec::new();
        for image in &mut self.images {
            if self.active_conversions >= slots {
                break;
            }
            if !matches!(image.status, Status::Pending) {
                continue;
            }
            image.status = Status::Converting;
            self.active_conversions += 1;

            let id = image.id;
            let job = Job {
                source: image.path.clone(),
                subdir: image.subdir.clone(),
            };
            let original_path = image.path.clone();
            let options = options.clone();
            tasks.push(Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        batch::convert(&job, &options).map_err(|error| error.to_string())
                    })
                    .await
                    .unwrap_or_else(|_| Err(t!("conversion-unexpected-error")))
                },
                move |result| Message::Converted(id, original_path.clone(), result),
            ));
        }

        if self.active_conversions == 0 {
            self.is_converting = false;
        }
        Task::batch(tasks)
    }

    fn open_file(&mut self, path: &Path) {
        // Only images produced by the application: never an executable or a script
        let is_output = OutputFormat::ALL
            .iter()
            .any(|format| format.extension() == conversion::extension(path));
        if !path.is_absolute() || !is_output {
            return;
        }
        let name = file_name(path);
        self.notice = if !path.exists() {
            Some(t!("notice-file-missing", name = name.as_str()))
        } else if open::that_detached(path).is_err() {
            Some(t!("notice-cannot-open", name = name.as_str()))
        } else {
            None
        };
    }

    fn view(&self) -> Element<'_, Message> {
        let body = match self.tab {
            Tab::Convert => convert::view(self),
            Tab::History => history::view(self),
        };
        container(
            column![widgets::header(self)]
                .push(
                    self.available_update
                        .map(|version| widgets::update_banner(version, self.tokens)),
                )
                .push(container(body).height(Length::Fill))
                .push(widgets::status_bar(self)),
        )
        .style(move |_| theme::page(self.tokens))
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| match event {
            Event::Window(window::Event::FileHovered(_)) => Some(Message::FileHovered),
            Event::Window(window::Event::FilesHoveredLeft) => Some(Message::HoverLeft),
            Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            _ => None,
        })
    }
}

async fn load_image(id: ImageId, job: Job) -> Option<ImageEntry> {
    let _permit = LOAD_LIMIT.acquire().await.ok()?;
    tokio::task::spawn_blocking(move || {
        let Job {
            source: path,
            subdir,
        } = job;
        let info = conversion::image_info(&path).ok()?;
        Some(ImageEntry {
            id,
            name: file_name(&path),
            size: info.size,
            dimensions: (info.width, info.height),
            thumbnail: image::Handle::from_rgba(
                THUMBNAIL_SIZE,
                THUMBNAIL_SIZE,
                info.thumbnail.into_raw(),
            ),
            status: Status::Pending,
            path,
            subdir,
        })
    })
    .await
    .ok()
    .flatten()
}

async fn pick_files() -> Vec<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(t!("dialog-choose-images"))
        .add_filter(t!("dialog-images-filter"), &INPUT_EXTENSIONS)
        .pick_files()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|file| file.path().to_owned())
        .collect()
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title(t!("dialog-choose-output"))
        .pick_folder()
        .await
        .map(|folder| folder.path().to_owned())
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || t!("file-unknown"),
        |name| name.to_string_lossy().into_owned(),
    )
}

const FORMAT_CHOICES: [FormatChoice; 5] = [
    FormatChoice::Fixed(OutputFormat::Webp),
    FormatChoice::Fixed(OutputFormat::Jpg),
    FormatChoice::Fixed(OutputFormat::Png),
    FormatChoice::Fixed(OutputFormat::Avif),
    FormatChoice::Original,
];

const RESIZE_MODES: [ResizeMode; 4] = [
    ResizeMode::LongestSide,
    ResizeMode::Width,
    ResizeMode::Height,
    ResizeMode::Percent,
];

const LANGUAGE_CHOICES: [LanguageChoice; 5] = [
    LanguageChoice::System,
    LanguageChoice::Fixed(Language::En),
    LanguageChoice::Fixed(Language::Fr),
    LanguageChoice::Fixed(Language::Es),
    LanguageChoice::Fixed(Language::De),
];

// Labels of the drop-down lists, in the current language
impl fmt::Display for FormatChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&match self {
            Self::Fixed(OutputFormat::Webp) => t!("format-webp"),
            Self::Fixed(OutputFormat::Jpg) => t!("format-jpg"),
            Self::Fixed(OutputFormat::Png) => t!("format-png"),
            Self::Fixed(OutputFormat::Avif) => t!("format-avif"),
            Self::Original => t!("format-original"),
        })
    }
}

/// Each language is shown in its own language, so that it can be recognized without reading
/// it; "System" shows the language it resolves to.
impl fmt::Display for LanguageChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => write!(
                formatter,
                "{} ({})",
                t!("language-system"),
                i18n::system_language().native_name()
            ),
            Self::Fixed(language) => formatter.write_str(language.native_name()),
        }
    }
}

impl fmt::Display for ResizeMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&match self {
            Self::LongestSide => t!("resize-longest-side"),
            Self::Width => t!("resize-width"),
            Self::Height => t!("resize-height"),
            Self::Percent => t!("resize-percent"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::Resize;

    fn app() -> (App, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        (App::with_data_dir(dir.path()), dir)
    }

    fn image(id: ImageId, status: Status) -> ImageEntry {
        ImageEntry {
            id,
            name: format!("photo-{id}.png"),
            path: PathBuf::from(format!("/images/photo-{id}.png")),
            subdir: PathBuf::new(),
            size: 1000,
            dimensions: (4000, 3000),
            thumbnail: image::Handle::from_rgba(1, 1, vec![0; 4]),
            status,
        }
    }

    fn result() -> ConversionResult {
        ConversionResult {
            output_path: PathBuf::from("/images/photo-1.webp"),
            original_size: 1000,
            new_size: 250,
            compression_ratio: 75.0,
            original_dimensions: (4000, 3000),
            new_dimensions: (4000, 3000),
        }
    }

    #[test]
    fn changing_a_setting_resets_processed_images_to_pending() {
        let (mut app, _dir) = app();
        app.images = vec![
            image(0, Status::Converted(result())),
            image(1, Status::Failed("error".to_owned())),
        ];

        let _ = app.update(Message::QualityChanged(60));

        assert_eq!(app.settings.quality, 60);
        assert_eq!(app.pending_count(), 2);
    }

    #[test]
    fn quality_does_not_change_in_png() {
        let (mut app, _dir) = app();
        let _ = app.update(Message::FormatSelected(FormatChoice::Fixed(
            OutputFormat::Png,
        )));
        app.images = vec![image(0, Status::Converted(result()))];

        let _ = app.update(Message::QualityChanged(10));

        assert_eq!(app.settings.quality, 80);
        assert_eq!(app.pending_count(), 0);
    }

    #[test]
    fn settings_are_locked_while_converting() {
        let (mut app, _dir) = app();
        app.is_converting = true;

        let _ = app.update(Message::QualityChanged(10));
        let _ = app.update(Message::ResizeToggled(true));

        assert_eq!(app.settings.quality, 80);
        assert!(!app.settings.resize_enabled);
    }

    #[test]
    fn resize_input_keeps_only_digits() {
        let (mut app, _dir) = app();
        let _ = app.update(Message::ResizeToggled(true));

        let _ = app.update(Message::ResizeValueChanged("12a 80px".to_owned()));

        assert_eq!(app.settings.resize_value, "1280");
        assert_eq!(
            app.settings.resize(),
            Ok(Some(Resize {
                mode: ResizeMode::LongestSide,
                value: 1280,
                allow_upscale: false,
            }))
        );
    }

    #[test]
    fn invalid_size_blocks_conversion() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Pending)];
        let _ = app.update(Message::ResizeToggled(true));
        let _ = app.update(Message::ResizeValueChanged(String::new()));

        let _ = app.update(Message::Convert);

        assert!(app.settings.resize().is_err());
        assert!(!app.is_converting);
    }

    #[test]
    fn switching_unit_restores_a_default_value() {
        let (mut app, _dir) = app();
        let _ = app.update(Message::ResizeValueChanged("800".to_owned()));

        let _ = app.update(Message::ResizeModeSelected(ResizeMode::Width));
        assert_eq!(app.settings.resize_value, "800");

        let _ = app.update(Message::ResizeModeSelected(ResizeMode::Percent));
        assert_eq!(app.settings.resize_value, DEFAULT_RESIZE_PERCENT);

        let _ = app.update(Message::ResizeModeSelected(ResizeMode::Height));
        assert_eq!(app.settings.resize_value, DEFAULT_RESIZE_PIXELS);
    }

    #[test]
    fn successful_conversion_is_added_to_history() {
        let (mut app, dir) = app();
        app.images = vec![image(0, Status::Converting)];
        app.is_converting = true;
        app.batch = (0, 1);

        let _ = app.update(Message::Converted(
            0,
            PathBuf::from("/images/photo-0.png"),
            Ok(result()),
        ));

        assert!(matches!(app.images[0].status, Status::Converted(_)));
        assert!(!app.is_converting);
        assert_eq!(app.batch, (1, 1));
        let saved = History::load(dir.path().join("history.json"));
        assert_eq!(saved.items().len(), 1);
        assert_eq!(saved.items()[0].output_path, result().output_path);
    }

    #[test]
    fn failure_does_not_stop_the_batch_or_reach_history() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Converting)];
        app.is_converting = true;

        let _ = app.update(Message::Converted(
            0,
            PathBuf::from("/images/photo-0.png"),
            Err("Unreadable image".to_owned()),
        ));

        assert!(
            matches!(&app.images[0].status, Status::Failed(error) if error == "Unreadable image")
        );
        assert!(app.history.items().is_empty());
    }

    #[test]
    fn images_keep_insertion_order() {
        let (mut app, _dir) = app();
        app.loading = 3;

        for id in [2, 0, 1] {
            let _ = app.update(Message::ImageLoaded(0, Some(image(id, Status::Pending))));
        }

        let ids: Vec<ImageId> = app.images.iter().map(|image| image.id).collect();
        assert_eq!(ids, [0, 1, 2]);
        assert_eq!(app.loading, 0);
    }

    #[test]
    fn images_loaded_after_the_list_was_cleared_are_dropped() {
        let (mut app, _dir) = app();
        app.loading = 2;
        let _ = app.update(Message::ImageLoaded(0, Some(image(0, Status::Pending))));

        let _ = app.update(Message::RemoveAllImages);
        let _ = app.update(Message::ImageLoaded(0, Some(image(1, Status::Pending))));

        assert!(app.images.is_empty());
        assert_eq!(app.loading, 0);
    }

    #[test]
    fn conversion_waits_for_images_to_finish_loading() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Pending)];
        app.loading = 1;

        let _ = app.update(Message::Convert);

        assert!(!app.is_converting);
    }

    #[test]
    fn files_picked_during_a_conversion_are_not_added() {
        let (mut app, _dir) = app();
        app.is_converting = true;
        app.is_dialog_open = true;

        let _ = app.update(Message::FilesSelected(vec![PathBuf::from("/images/a.png")]));

        assert_eq!(app.loading, 0);
        assert!(!app.is_dialog_open);
    }

    #[test]
    fn converts_several_images_at_once_but_one_at_a_time_in_avif() {
        let (mut app, _dir) = app();
        app.images = (0..6).map(|id| image(id, Status::Pending)).collect();

        let _ = app.update(Message::Convert);

        let slots = conversion::parallel_conversions(app.settings.format);
        assert!(slots >= 1);
        assert_eq!(app.active_conversions, slots);
        assert_eq!(app.batch, (0, 6));
        assert_eq!(app.pending_count(), 6 - slots);

        let (mut app, _dir) = self::app();
        app.images = (0..6).map(|id| image(id, Status::Pending)).collect();
        let _ = app.update(Message::FormatSelected(FormatChoice::Fixed(
            OutputFormat::Avif,
        )));

        let _ = app.update(Message::Convert);

        assert_eq!(app.active_conversions, 1);
    }

    #[test]
    fn batch_ends_when_the_last_conversion_returns() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Converting), image(1, Status::Converting)];
        app.is_converting = true;
        app.active_conversions = 2;
        app.batch = (0, 2);

        let failure = || Err("Unreadable image".to_owned());
        let _ = app.update(Message::Converted(
            0,
            PathBuf::from("/images/a.png"),
            failure(),
        ));
        assert!(app.is_converting);

        let _ = app.update(Message::Converted(
            1,
            PathBuf::from("/images/b.png"),
            failure(),
        ));
        assert!(!app.is_converting);
        assert_eq!(app.batch, (2, 2));
    }

    #[test]
    fn clearing_history_needs_a_confirmation() {
        let (mut app, _dir) = app();
        let item = HistoryItem::new(PathBuf::from("/images/a.png"), &result(), Utc::now());
        app.history.add(item).unwrap();

        let _ = app.update(Message::ClearHistory);
        assert!(app.is_confirming_clear);
        assert_eq!(app.history.items().len(), 1);

        let _ = app.update(Message::ClearHistoryConfirmed(false));
        assert!(!app.is_confirming_clear);
        assert_eq!(app.history.items().len(), 1);

        let _ = app.update(Message::ClearHistory);
        let _ = app.update(Message::ClearHistoryConfirmed(true));
        assert!(app.history.items().is_empty());
    }

    #[test]
    fn reports_ignored_files() {
        let (mut app, _dir) = app();
        app.loading = 2;

        let _ = app.update(Message::ImageLoaded(0, None));
        let _ = app.update(Message::ImageLoaded(0, None));

        assert_eq!(
            app.notice.as_deref(),
            Some("2 fichiers ignorés : format non pris en charge ou fichier illisible")
        );
    }

    #[test]
    fn shows_an_available_update_until_dismissed() {
        let (mut app, _dir) = app();
        let version = Version::parse("2.1.0");

        let _ = app.update(Message::UpdateChecked(version));
        assert_eq!(app.available_update, version);

        let _ = app.update(Message::UpdateDismissed);
        assert_eq!(app.available_update, None);
    }

    #[test]
    fn turning_update_checks_off_is_persisted_and_hides_the_banner() {
        let (mut app, dir) = app();
        let _ = app.update(Message::UpdateChecked(Version::parse("2.1.0")));

        let _ = app.update(Message::UpdateCheckToggled(false));

        assert_eq!(app.available_update, None);
        assert!(!App::with_data_dir(dir.path()).preferences.check_updates);

        // An answer arriving afterwards does not reopen the banner
        let _ = app.update(Message::UpdateChecked(Version::parse("2.1.0")));
        assert_eq!(app.available_update, None);
    }

    #[test]
    fn settings_are_persisted_for_the_context_menu() {
        let (mut app, dir) = app();

        let _ = app.update(Message::QualityChanged(55));
        let _ = app.update(Message::ResizeToggled(true));

        let reloaded = App::with_data_dir(dir.path()).settings;
        assert_eq!(reloaded.quality, 55);
        assert!(reloaded.resize_enabled);
    }

    #[test]
    fn language_choice_applies_at_once_and_is_persisted() {
        let (mut app, dir) = app();

        let _ = app.update(Message::LanguageSelected(LanguageChoice::Fixed(
            Language::De,
        )));

        assert_eq!(i18n::current(), Language::De);
        assert_eq!(
            FormatChoice::Original.to_string(),
            "Originalformat - Nur zum Skalieren"
        );
        assert_eq!(
            App::with_data_dir(dir.path()).preferences.language,
            LanguageChoice::Fixed(Language::De)
        );
    }

    #[test]
    fn dark_mode_is_persisted() {
        let (mut app, dir) = app();

        let _ = app.update(Message::DarkModeToggled);

        assert!(App::with_data_dir(dir.path()).preferences.dark_mode);
    }

    #[test]
    fn opens_only_converted_images() {
        let (mut app, _dir) = app();

        app.open_file(&std::env::temp_dir().join("script.exe"));
        assert_eq!(app.notice, None);

        let missing = std::env::temp_dir().join("lotpix-absent.webp");
        app.open_file(&missing);
        assert_eq!(
            app.notice.as_deref(),
            Some("Le fichier n'existe plus : lotpix-absent.webp")
        );
    }
}
