// Réglages de conversion, conservés d'un lancement à l'autre : le menu contextuel compresse
// avec les derniers réglages choisis dans l'interface
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::conversion::{ConvertOptions, FormatChoice, OutputFormat, Resize, ResizeMode};
use crate::storage;

pub const DEFAULT_RESIZE_PIXELS: &str = "1920";
pub const DEFAULT_RESIZE_PERCENT: &str = "50";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub format: FormatChoice,
    pub quality: u8,
    /// Sans dossier de sortie, les fichiers sont écrits à côté des originaux.
    pub output_dir: Option<PathBuf>,
    pub resize_enabled: bool,
    pub resize_mode: ResizeMode,
    /// Saisie brute du champ : validée par `resize`.
    pub resize_value: String,
    pub allow_upscale: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: FormatChoice::Fixed(OutputFormat::Webp),
            quality: 80,
            output_dir: None,
            resize_enabled: false,
            resize_mode: ResizeMode::LongestSide,
            resize_value: DEFAULT_RESIZE_PIXELS.to_owned(),
            allow_upscale: false,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        storage::load(path)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        storage::save(path, self)
    }

    pub fn is_lossless(&self) -> bool {
        self.format == FormatChoice::Fixed(OutputFormat::Png)
    }

    /// Redimensionnement demandé, ou le message à afficher si la saisie est invalide.
    pub fn resize(&self) -> Result<Option<Resize>, String> {
        if !self.resize_enabled {
            return Ok(None);
        }
        let resize = Resize {
            mode: self.resize_mode,
            // Une saisie vide ou trop longue pour un entier donne 0, refusé par la validation
            value: self.resize_value.parse().unwrap_or(0),
            allow_upscale: self.allow_upscale,
        };
        resize.validate().map_err(|error| error.to_string())?;
        Ok(Some(resize))
    }

    pub fn options(&self) -> Option<ConvertOptions> {
        Some(ConvertOptions {
            output_dir: self.output_dir.clone(),
            quality: self.quality,
            format: self.format,
            resize: self.resize().ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reloads_saved_settings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        assert_eq!(Settings::load(&path), Settings::default());

        let saved = Settings {
            format: FormatChoice::Original,
            quality: 65,
            output_dir: Some(dir.path().join("sortie")),
            resize_enabled: true,
            resize_mode: ResizeMode::Percent,
            resize_value: "50".to_owned(),
            allow_upscale: true,
        };
        saved.save(&path).unwrap();

        assert_eq!(Settings::load(&path), saved);
    }

    #[test]
    fn missing_fields_keep_their_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"format":{"fixed":"avif"}}"#).unwrap();

        let loaded = Settings::load(&path);

        assert_eq!(loaded.format, FormatChoice::Fixed(OutputFormat::Avif));
        assert_eq!(loaded.quality, Settings::default().quality);
    }
}
