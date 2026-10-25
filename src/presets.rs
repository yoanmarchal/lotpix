// Préréglages du menu contextuel : qualité recommandée pour chaque format, tailles standard et
// formats des réseaux sociaux. Ils ne dépendent pas des réglages de l'interface, pour que le
// menu donne toujours le même résultat
use std::path::Path;

use crate::conversion::{ConvertOptions, FormatChoice, Frame, OutputFormat, Resize, ResizeMode};
use crate::t;

/// Qualité donnant des artefacts invisibles à l'œil pour un fichier léger. L'échelle de l'AVIF
/// est plus sévère : 65 y vaut à peu près 80 en WebP, pour un fichier plus léger.
pub fn recommended_quality(format: OutputFormat) -> u8 {
    match format {
        OutputFormat::Webp => 80,
        OutputFormat::Jpg => 82,
        // Sans perte : la qualité est ignorée
        OutputFormat::Png => 100,
        OutputFormat::Avif => 65,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Côté le plus long, en pixels.
    LongestSide(u32),
    Percent(u32),
    /// Taille exacte, avec recadrage au centre.
    Frame(Frame),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Size {
    /// Identifiant de la ligne de commande (`--size`).
    pub id: &'static str,
    pub target: Target,
}

impl Size {
    /// Nom affiché, dans la langue courante.
    pub fn label(&self) -> String {
        match self.id {
            "4k" => t!("size-4k"),
            "full-hd" => t!("size-full-hd"),
            "hd" => t!("size-hd"),
            "web" => t!("size-web"),
            "thumbnail" => t!("size-thumbnail"),
            "half" => t!("size-half"),
            "instagram-square" => t!("size-instagram-square"),
            "instagram-portrait" => t!("size-instagram-portrait"),
            "story" => t!("size-story"),
            "facebook" => t!("size-facebook"),
            "x" => t!("size-x"),
            "youtube" => t!("size-youtube"),
            "pinterest" => t!("size-pinterest"),
            _ => unreachable!("taille sans libellé : {}", self.id),
        }
    }
}

const fn frame(width: u32, height: u32) -> Target {
    Target::Frame(Frame { width, height })
}

/// Tailles standard, puis formats des réseaux sociaux.
pub const SIZES: [Size; 13] = [
    Size {
        id: "4k",
        target: Target::LongestSide(3840),
    },
    Size {
        id: "full-hd",
        target: Target::LongestSide(1920),
    },
    Size {
        id: "hd",
        target: Target::LongestSide(1280),
    },
    Size {
        id: "web",
        target: Target::LongestSide(1024),
    },
    Size {
        id: "thumbnail",
        target: Target::LongestSide(640),
    },
    Size {
        id: "half",
        target: Target::Percent(50),
    },
    Size {
        id: "instagram-square",
        target: frame(1080, 1080),
    },
    Size {
        id: "instagram-portrait",
        target: frame(1080, 1350),
    },
    Size {
        id: "story",
        target: frame(1080, 1920),
    },
    Size {
        id: "facebook",
        target: frame(1200, 630),
    },
    Size {
        id: "x",
        target: frame(1600, 900),
    },
    Size {
        id: "youtube",
        target: frame(1280, 720),
    },
    Size {
        id: "pinterest",
        target: frame(1000, 1500),
    },
];

pub fn size(id: &str) -> Option<&'static Size> {
    SIZES.iter().find(|size| size.id == id)
}

/// Entrée choisie dans le menu contextuel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Compression au format choisi, sans redimensionnement.
    Compress(FormatChoice),
    /// Redimensionnement. Les réseaux sociaux recompressent les images et acceptent mal le
    /// WebP et l'AVIF : leurs formats sont écrits en JPEG ; les tailles standard gardent le
    /// format d'origine.
    Resize(&'static Size),
}

impl Preset {
    /// Identifiant stable, par exemple « compress-webp » ou « resize-story ».
    pub fn id(&self) -> String {
        match self {
            Self::Compress(FormatChoice::Original) => "compress-original".to_owned(),
            Self::Compress(FormatChoice::Fixed(format)) => {
                format!("compress-{}", format.extension())
            }
            Self::Resize(size) => format!("resize-{}", size.id),
        }
    }

    pub fn format(&self) -> FormatChoice {
        match self {
            Self::Compress(format) => *format,
            Self::Resize(Size {
                target: Target::Frame(_),
                ..
            }) => FormatChoice::Fixed(OutputFormat::Jpg),
            Self::Resize(_) => FormatChoice::Original,
        }
    }

    /// Options pour une image, avec la qualité recommandée pour le format dans lequel elle
    /// sera écrite. Le résultat va toujours à côté de l'original.
    pub fn options(&self, path: &Path) -> ConvertOptions {
        let format = self.format().resolve(path);
        let (resize, frame) = match self {
            Self::Compress(_) => (None, None),
            Self::Resize(size) => match size.target {
                Target::LongestSide(value) => (Some(shrink(ResizeMode::LongestSide, value)), None),
                Target::Percent(value) => (Some(shrink(ResizeMode::Percent, value)), None),
                Target::Frame(frame) => (None, Some(frame)),
            },
        };
        ConvertOptions {
            output_dir: None,
            quality: recommended_quality(format),
            format: FormatChoice::Fixed(format),
            resize,
            frame,
        }
    }
}

/// Une image déjà plus petite que la cible garde sa taille.
fn shrink(mode: ResizeMode, value: u32) -> Resize {
    Resize {
        mode,
        value,
        allow_upscale: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_ids_are_unique_and_valid() {
        for (index, size) in SIZES.iter().enumerate() {
            assert_eq!(super::size(size.id), Some(size), "{} en double", size.id);
            match size.target {
                Target::Frame(frame) => frame.validate().unwrap(),
                Target::LongestSide(value) => {
                    shrink(ResizeMode::LongestSide, value).validate().unwrap()
                }
                Target::Percent(value) => shrink(ResizeMode::Percent, value).validate().unwrap(),
            }
            assert!(SIZES[..index].iter().all(|other| other.id != size.id));
        }
    }

    #[test]
    fn original_format_uses_the_quality_of_the_source_format() {
        let preset = Preset::Compress(FormatChoice::Original);

        let jpeg = preset.options(Path::new("/images/a.jpg"));
        assert_eq!(jpeg.format, FormatChoice::Fixed(OutputFormat::Jpg));
        assert_eq!(jpeg.quality, 82);

        let webp = preset.options(Path::new("/images/a.webp"));
        assert_eq!(webp.quality, 80);
    }

    #[test]
    fn social_sizes_are_cropped_jpegs() {
        let preset = Preset::Resize(size("instagram-portrait").unwrap());

        let options = preset.options(Path::new("/images/a.png"));

        assert_eq!(options.format, FormatChoice::Fixed(OutputFormat::Jpg));
        assert_eq!(
            options.frame,
            Some(Frame {
                width: 1080,
                height: 1350
            })
        );
        assert_eq!(options.resize, None);
        assert_eq!(preset.id(), "resize-instagram-portrait");
    }

    #[test]
    fn standard_sizes_keep_the_source_format() {
        let preset = Preset::Resize(size("full-hd").unwrap());

        let options = preset.options(Path::new("/images/a.png"));

        assert_eq!(options.format, FormatChoice::Fixed(OutputFormat::Png));
        assert_eq!(options.resize, Some(shrink(ResizeMode::LongestSide, 1920)));
        assert_eq!(options.output_dir, None);
    }
}
