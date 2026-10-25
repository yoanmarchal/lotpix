// Application language: the system one by default, or the one chosen in the preferences.
// Texts live in i18n/<language>/lotpix.ftl (Fluent) and are obtained with the `t!` macro,
// checked at compile time against English
use std::sync::LazyLock;

use i18n_embed::LanguageLoader;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Fr,
    Es,
    De,
}

impl Language {
    pub const ALL: [Language; 4] = [Self::En, Self::Fr, Self::Es, Self::De];

    /// ISO 639-1 code, also the name of the translations folder.
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::Es => "es",
            Self::De => "de",
        }
    }

    /// Name of the language in that language, for the list of choices.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Fr => "Français",
            Self::Es => "Español",
            Self::De => "Deutsch",
        }
    }

    /// Language of a locale code ("fr-CA", "de_DE.UTF-8"…), if it is translated.
    pub fn from_locale(locale: &str) -> Option<Self> {
        let primary = locale
            .split(['-', '_', '.', '@'])
            .next()?
            .to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|language| language.code() == primary)
    }
}

/// Preferences setting: follow the system or force a language.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageChoice {
    #[default]
    System,
    Fixed(Language),
}

impl LanguageChoice {
    pub fn resolve(self) -> Language {
        match self {
            Self::System => system_language(),
            Self::Fixed(language) => language,
        }
    }
}

/// First translated language among the user's preferred languages, otherwise English.
pub fn system_language() -> Language {
    sys_locale::get_locales()
        .find_map(|locale| Language::from_locale(&locale))
        .unwrap_or(Language::En)
}

/// One loader per language, prepared once: changing the language reloads nothing. A text
/// missing in a language is shown in English.
static LOADERS: LazyLock<Vec<FluentLanguageLoader>> = LazyLock::new(|| {
    Language::ALL
        .into_iter()
        .map(|language| {
            let loader = fluent_language_loader!();
            let identifier: LanguageIdentifier =
                language.code().parse().expect("valid language code");
            let fallback = loader.fallback_language().clone();
            loader
                .load_languages(&Localizations, &[identifier, fallback])
                .expect("translations embedded in the binary");
            // Without the bidirectional isolation characters around inserted values: useless for these
            // languages, and shown as boxes by some fonts
            loader.set_use_isolating(false);
            loader
        })
        .collect()
});

pub fn loader() -> &'static FluentLanguageLoader {
    let index = Language::ALL
        .iter()
        .position(|language| *language == current())
        .expect("known language");
    &LOADERS[index]
}

/// Text translated into the current language: `t!("key")` or `t!("key", count = 3)`.
#[macro_export]
macro_rules! t {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::loader(), $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::loader(), $message_id, $($args),*)
    }};
}

// The language is global to the application. In tests, each running in its own thread, it
// belongs to the thread: a test that changes the language does not disturb the others, and
// they all start in French
#[cfg(not(test))]
mod state {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::Language;

    static CURRENT: AtomicUsize = AtomicUsize::new(0);

    pub fn current() -> Language {
        Language::ALL[CURRENT.load(Ordering::Relaxed)]
    }

    pub fn set(language: Language) {
        let index = Language::ALL
            .iter()
            .position(|other| *other == language)
            .expect("known language");
        CURRENT.store(index, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod state {
    use std::cell::Cell;

    use super::Language;

    thread_local! {
        static CURRENT: Cell<Language> = const { Cell::new(Language::Fr) };
    }

    pub fn current() -> Language {
        CURRENT.get()
    }

    pub fn set(language: Language) {
        CURRENT.set(language);
    }
}

pub fn current() -> Language {
    state::current()
}

pub fn set_language(language: Language) {
    state::set(language);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn recognizes_translated_locales() {
        assert_eq!(Language::from_locale("fr-CA"), Some(Language::Fr));
        assert_eq!(Language::from_locale("de_DE.UTF-8"), Some(Language::De));
        assert_eq!(Language::from_locale("ES"), Some(Language::Es));
        assert_eq!(Language::from_locale("en-US"), Some(Language::En));
        assert_eq!(Language::from_locale("ja-JP"), None);
        assert_eq!(Language::from_locale(""), None);
    }

    #[test]
    fn translates_into_the_current_language() {
        set_language(Language::De);
        assert_eq!(t!("language"), "Sprache");

        set_language(Language::Fr);
        assert_eq!(t!("language"), "Langue");
    }

    #[test]
    fn saved_choice_round_trips() {
        for choice in [LanguageChoice::System, LanguageChoice::Fixed(Language::Es)] {
            let json = serde_json::to_string(&choice).unwrap();
            assert_eq!(
                serde_json::from_str::<LanguageChoice>(&json).unwrap(),
                choice
            );
        }
    }

    /// Message identifiers of a Fluent file, without comments, attributes or continuation
    /// lines.
    fn message_ids(source: &str) -> BTreeSet<String> {
        source
            .lines()
            .filter(|line| !line.starts_with([' ', '#', '.']) && !line.is_empty())
            .filter_map(|line| line.split_once('=').map(|(id, _)| id.trim().to_owned()))
            .collect()
    }

    /// Each language translates exactly the English texts: nothing missing, no obsolete key.
    #[test]
    fn every_language_has_every_message() {
        let source = |language: Language| {
            let file = Localizations::get(&format!("{}/lotpix.ftl", language.code()))
                .expect("translations file");
            String::from_utf8(file.data.into_owned()).unwrap()
        };
        let reference = message_ids(&source(Language::En));
        assert!(!reference.is_empty());
        for language in Language::ALL {
            assert_eq!(
                message_ids(&source(language)),
                reference,
                "{} translations missing or extra",
                language.code()
            );
        }
    }
}
