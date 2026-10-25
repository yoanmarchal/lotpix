// Context menu presets: recommended quality for each format, standard sizes and social
// network formats. They do not depend on the interface settings, so that the menu always
// gives the same result
use std::path::Path;

use crate::conversion::{ConvertOptions, FormatChoice, Frame, OutputFormat, Resize, ResizeMode};
use crate::t;

/// Quality giving artifacts invisible to the eye for a small file. The AVIF scale is
/// harsher: 65 there is roughly 80 in WebP, for a smaller file.
pub fn recommended_quality(format: OutputFormat) -> u8 {
    match format {
        OutputFormat::Webp => 80,
        OutputFormat::Jpg => 82,
        // Lossless: quality is ignored
        OutputFormat::Png => 100,
        OutputFormat::Avif => 65,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Longest side, in pixels.
    LongestSide(u32),
    Percent(u32),
    /// Exact size, cropped to the center.
    Frame(Frame),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Size {
    /// Command line identifier (`--size`).
    pub id: &'static str,
    pub target: Target,
}

impl Size {
    /// Displayed name, in the current language.
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
            _ => unreachable!("size without a label: {}", self.id),
        }
    }
}

const fn frame(width: u32, height: u32) -> Target {
    Target::Frame(Frame { width, height })
}

/// Standard sizes, then social network formats.
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

/// Submenu of a context menu: "Compress with Lotpix" or "Resize with Lotpix".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Submenu {
    Compress,
    Resize,
}

impl Submenu {
    pub fn title(self) -> String {
        match self {
            Self::Compress => t!("menu-compress"),
            Self::Resize => t!("menu-resize"),
        }
    }

    /// Prompt of the list of choices, for file managers without submenus.
    pub fn prompt(self) -> String {
        match self {
            Self::Compress => t!("menu-choose-format"),
            Self::Resize => t!("menu-choose-size"),
        }
    }

    /// Command line option that each entry receives.
    pub fn option(self) -> &'static str {
        match self {
            Self::Compress => "compress --format",
            Self::Resize => "resize --size",
        }
    }

    /// Entries in menu order: option value and label in the current language.
    pub fn entries(self) -> Vec<(&'static str, String)> {
        match self {
            Self::Compress => vec![
                ("webp", t!("menu-format-webp")),
                ("jpg", t!("menu-format-jpg")),
                ("png", t!("menu-format-png")),
                ("avif", t!("menu-format-avif")),
                ("original", t!("menu-format-original")),
            ],
            Self::Resize => SIZES.iter().map(|size| (size.id, size.label())).collect(),
        }
    }
}

/// Entry chosen in the context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Compression to the chosen format, without resizing.
    Compress(FormatChoice),
    /// Resizing. Social networks recompress images and handle WebP and AVIF poorly: their
    /// formats are saved as JPEG; the standard sizes keep the original
    /// format.
    Resize(&'static Size),
}

impl Preset {
    /// Stable identifier, for example "compress-webp" or "resize-story".
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

    /// Options for an image, with the recommended quality for the format it will be saved in.
    /// The result always goes next to the original.
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

/// An image already smaller than the target keeps its size.
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
            assert_eq!(
                super::size(size.id),
                Some(size),
                "{} is duplicated",
                size.id
            );
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
