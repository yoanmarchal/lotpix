// Préférences conservées d'un lancement à l'autre
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::i18n::LanguageChoice;
use crate::storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub dark_mode: bool,
    /// Interroger GitHub au démarrage pour signaler une nouvelle version.
    pub check_updates: bool,
    /// Langue de l'interface : celle du système par défaut.
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

        // Fichier écrit par une version qui ne connaissait pas encore ce réglage
        std::fs::write(&path, r#"{"darkMode":true}"#).unwrap();

        let loaded = Preferences::load(&path);
        assert!(loaded.dark_mode);
        assert!(loaded.check_updates);
        assert_eq!(loaded.language, LanguageChoice::System);
    }
}
