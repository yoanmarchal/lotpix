// Langue de l'application : celle du système par défaut, ou celle choisie dans les
// préférences. Les textes sont dans i18n/<langue>/lotpix.ftl (Fluent) et s'obtiennent avec
// la macro `t!`, vérifiée à la compilation contre l'anglais
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

    /// Code ISO 639-1, aussi nom du dossier de traductions.
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::Es => "es",
            Self::De => "de",
        }
    }

    /// Nom de la langue dans cette langue, pour la liste de choix.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Fr => "Français",
            Self::Es => "Español",
            Self::De => "Deutsch",
        }
    }

    /// Langue d'un code de locale (« fr-CA », « de_DE.UTF-8 »…), si elle est traduite.
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

/// Réglage des préférences : suivre le système ou imposer une langue.
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

/// Première langue traduite parmi les langues préférées de l'utilisateur, sinon l'anglais.
pub fn system_language() -> Language {
    sys_locale::get_locales()
        .find_map(|locale| Language::from_locale(&locale))
        .unwrap_or(Language::En)
}

/// Un chargeur par langue, préparé une fois : changer de langue ne recharge rien. Un texte
/// manquant dans une langue s'affiche en anglais.
static LOADERS: LazyLock<Vec<FluentLanguageLoader>> = LazyLock::new(|| {
    Language::ALL
        .into_iter()
        .map(|language| {
            let loader = fluent_language_loader!();
            let identifier: LanguageIdentifier =
                language.code().parse().expect("code de langue valide");
            let fallback = loader.fallback_language().clone();
            loader
                .load_languages(&Localizations, &[identifier, fallback])
                .expect("traductions intégrées au binaire");
            // Sans les caractères d'isolation bidirectionnelle autour des valeurs insérées :
            // inutiles pour ces langues, et affichés comme des carrés par certaines polices
            loader.set_use_isolating(false);
            loader
        })
        .collect()
});

pub fn loader() -> &'static FluentLanguageLoader {
    let index = Language::ALL
        .iter()
        .position(|language| *language == current())
        .expect("langue connue");
    &LOADERS[index]
}

/// Texte traduit dans la langue courante : `t!("cle")` ou `t!("cle", nombre = 3)`.
#[macro_export]
macro_rules! t {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::loader(), $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::loader(), $message_id, $($args),*)
    }};
}

// La langue est globale à l'application. Dans les tests, chacun dans son propre thread, elle
// est propre au thread : un test qui change de langue ne perturbe pas les autres, et tous
// partent du français
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
            .expect("langue connue");
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

    /// Identifiants des messages d'un fichier Fluent, sans les commentaires, les attributs ni
    /// les lignes de continuation.
    fn message_ids(source: &str) -> BTreeSet<String> {
        source
            .lines()
            .filter(|line| !line.starts_with([' ', '#', '.']) && !line.is_empty())
            .filter_map(|line| line.split_once('=').map(|(id, _)| id.trim().to_owned()))
            .collect()
    }

    /// Chaque langue traduit exactement les textes de l'anglais : ni oubli, ni clé obsolète.
    #[test]
    fn every_language_has_every_message() {
        let source = |language: Language| {
            let file = Localizations::get(&format!("{}/lotpix.ftl", language.code()))
                .expect("fichier de traductions");
            String::from_utf8(file.data.into_owned()).unwrap()
        };
        let reference = message_ids(&source(Language::En));
        assert!(!reference.is_empty());
        for language in Language::ALL {
            assert_eq!(
                message_ids(&source(language)),
                reference,
                "traductions {} incomplètes ou en trop",
                language.code()
            );
        }
    }
}
