// Preferences kept from one launch to the next
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::i18n::LanguageChoice;
use crate::storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub dark_mode: bool,
    /// Query GitHub at startup to announce a new version.
    pub check_updates: bool,
    /// Interface language: the system one by default.
    pub language: LanguageChoice,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark_mode: false,
            check_updates: true,
            language: LanguageChoice::System,
        }
    }
}

impl Preferences {
    pub fn load(path: &Path) -> Self {
        storage::load(path)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        storage::save(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reloads_saved_preferences() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences.json");

        assert_eq!(Preferences::load(&path), Preferences::default());

        let saved = Preferences {
            dark_mode: true,
            check_updates: false,
            language: LanguageChoice::Fixed(crate::i18n::Language::De),
        };
        saved.save(&path).unwrap();

        assert_eq!(Preferences::load(&path), saved);
    }

    #[test]
    fn checks_updates_unless_turned_off() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences.json");
        assert!(Preferences::default().check_updates);

        // File written by a version that did not know this setting yet
        std::fs::write(&path, r#"{"darkMode":true}"#).unwrap();

        let loaded = Preferences::load(&path);
        assert!(loaded.dark_mode);
        assert!(loaded.check_updates);
        assert_eq!(loaded.language, LanguageChoice::System);
    }
}
