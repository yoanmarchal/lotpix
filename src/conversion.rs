// Image conversion and resizing, independent of the interface (testable with cargo test)
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, BufReader, Cursor, Write};
use std::path::{Path, PathBuf};

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::gif::GifDecoder;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::metadata::LoopCount;
use image::{
    AnimationDecoder, DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageError,
    ImageFormat, ImageReader, Limits, RgbImage, RgbaImage,
};
use moxcms::{ColorProfile, Layout, TransformOptions};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::animated_webp::AnimatedWebpEncoder;
use crate::t;

// AVIF is not accepted as input: it is already optimized, and decoding it would require dav1d
pub const INPUT_EXTENSIONS: [&str; 7] = ["jpg", "jpeg", "png", "gif", "tif", "tiff", "webp"];

// Side of the square shown in the list (48 px), doubled for high-density screens
pub const THUMBNAIL_SIZE: u32 = 96;

pub const MAX_RESIZE_PIXELS: u32 = 16_384;
pub const MAX_RESIZE_PERCENT: u32 = 400;

// An upscale beyond 512 MiB of RGBA pixels is refused rather than attempted: it is also the
// limit applied when decoding
const MAX_OUTPUT_PIXELS: u64 = 512 * 1024 * 1024 / 4;

// A decoded 50-megapixel photo takes 200 MB, and as much again while it is resized
const MAX_PARALLEL_CONVERSIONS: usize = 4;

const MIN_FRAME_DELAY_MS: u32 = 10;
const DEFAULT_FRAME_DELAY_MS: u32 = 100;

/// Conversion error. The message is translated into the current language when it is displayed
/// (see `Display`).
#[derive(Debug, Error)]
pub enum ConversionError {
    InvalidPath,
    /// Extension with its dot (".bmp"), or empty for a file without an extension.
    UnsupportedExtension(String),
    SourceNotFound(PathBuf),
    NotAFile(PathBuf),
    InvalidQuality,
    InvalidResize {
        max: u32,
        unit: &'static str,
    },
    TooLarge {
        width: u32,
        height: u32,
        format: &'static str,
        max: u32,
    },
    TooManyPixels {
        width: u32,
        height: u32,
        max: u64,
    },
    EmptyOutput,
    Decode(#[from] ImageError),
    Resize(String),
    Encode(String),
    Io(#[from] io::Error),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidPath => t!("error-invalid-path"),
            Self::UnsupportedExtension(extension) if extension.is_empty() => t!(
                "error-unsupported-extension",
                extension = t!("error-no-extension")
            ),
            Self::UnsupportedExtension(extension) => {
                t!(
                    "error-unsupported-extension",
                    extension = extension.as_str()
                )
            }
            Self::SourceNotFound(path) => {
                t!("error-source-not-found", path = path.display().to_string())
            }
            Self::NotAFile(path) => t!("error-not-a-file", path = path.display().to_string()),
            Self::InvalidQuality => t!("error-invalid-quality"),
            Self::InvalidResize { max, unit } => {
                t!(
                    "error-invalid-resize",
                    max = max.to_string(),
                    unit = unit.to_string()
                )
            }
            // Numbers are passed as text: Fluent would otherwise add thousands separators, which
            // the original messages do not have
            Self::TooLarge {
                width,
                height,
                format,
                max,
            } => t!(
                "error-too-large",
                width = width.to_string(),
                height = height.to_string(),
                format = format.to_string(),
                max = max.to_string()
            ),
            Self::TooManyPixels { width, height, max } => t!(
                "error-too-many-pixels",
                width = width.to_string(),
                height = height.to_string(),
                max = max.to_string()
            ),
            Self::EmptyOutput => t!("error-empty-output"),
            Self::Decode(error) => t!("error-decode", error = error.to_string()),
            Self::Resize(error) => t!("error-resize", error = error.as_str()),
            Self::Encode(error) => t!("error-encode", error = error.as_str()),
            // Operating system message, already in the user's language
            Self::Io(error) => return fmt::Display::fmt(error, formatter),
        };
        formatter.write_str(&message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Webp,
    Jpg,
    Png,
    Avif,
}

impl OutputFormat {
    pub const ALL: [OutputFormat; 4] = [Self::Webp, Self::Jpg, Self::Png, Self::Avif];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Avif => "avif",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Webp => "WebP",
            Self::Jpg => "JPEG",
            Self::Png => "PNG",
            Self::Avif => "AVIF",
        }
    }

    /// Largest side the format can store.
    fn max_dimension(self) -> u32 {
        match self {
            Self::Webp => 16_383,
            Self::Jpg | Self::Avif => 65_535,
            Self::Png => u32::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FormatChoice {
    /// Keeps the format of the source file (useful to only resize).
    Original,
    Fixed(OutputFormat),
}

impl FormatChoice {
    pub fn resolve(self, path: &Path) -> OutputFormat {
        match self {
            Self::Fixed(format) => format,
            Self::Original => match extension(path).as_str() {
                "jpg" | "jpeg" => OutputFormat::Jpg,
                "webp" => OutputFormat::Webp,
                // GIF and TIFF are not output formats: lossless fallback
                _ => OutputFormat::Png,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResizeMode {
    Width,
    Height,
    LongestSide,
    Percent,
}

/// Resizing that keeps the aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resize {
    pub mode: ResizeMode,
    /// Pixels, or a percentage in `Percent` mode.
    pub value: u32,
    /// Without this option, an image smaller than the target keeps its size.
    pub allow_upscale: bool,
}

impl Resize {
    pub fn validate(&self) -> Result<(), ConversionError> {
        let (max, unit) = match self.mode {
            ResizeMode::Percent => (MAX_RESIZE_PERCENT, "%"),
            _ => (MAX_RESIZE_PIXELS, "px"),
        };
        if (1..=max).contains(&self.value) {
            Ok(())
        } else {
            Err(ConversionError::InvalidResize { max, unit })
        }
    }
}

/// Exact required size, for example 1080 × 1350 for Instagram: the image is resized to cover
/// the frame, then cropped to the center.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
}

impl Frame {
    pub fn validate(&self) -> Result<(), ConversionError> {
        let fits = |side: u32| (1..=MAX_RESIZE_PIXELS).contains(&side);
        if fits(self.width) && fits(self.height) {
            Ok(())
        } else {
            Err(ConversionError::InvalidResize {
                max: MAX_RESIZE_PIXELS,
                unit: "px",
            })
        }
    }
}

/// Output dimensions of a `width` × `height` image cropped to `frame`. An image too small for
/// the frame is not enlarged: it is only cropped to the same proportions.
pub fn frame_dimensions(width: u32, height: u32, frame: &Frame) -> (u32, u32) {
    let scale = (f64::from(frame.width) / f64::from(width))
        .max(f64::from(frame.height) / f64::from(height));
    if scale <= 1.0 {
        return (frame.width, frame.height);
    }
    let scaled = |side: u32| ((f64::from(side) / scale).round() as u32).max(1);
    (scaled(frame.width), scaled(frame.height))
}

/// Output dimensions of a `width` × `height` image (EXIF orientation already applied).
pub fn target_dimensions(width: u32, height: u32, resize: &Resize) -> (u32, u32) {
    let value = f64::from(resize.value);
    let scale = match resize.mode {
        ResizeMode::Width => value / f64::from(width),
        ResizeMode::Height => value / f64::from(height),
        ResizeMode::LongestSide => value / f64::from(width.max(height)),
        ResizeMode::Percent => value / 100.0,
    };
    let scale = if resize.allow_upscale {
        scale
    } else {
        scale.min(1.0)
    };
    let scaled = |side: u32| ((f64::from(side) * scale).round() as u32).max(1);
    (scaled(width), scaled(height))
}

/// Number of conversions run at the same time. Only the AVIF encoder already uses every core;
/// the others use one, but each decoded image takes a lot of memory.
pub fn parallel_conversions(format: FormatChoice) -> usize {
    if format == FormatChoice::Fixed(OutputFormat::Avif) {
        1
    } else {
        std::thread::available_parallelism()
            .map_or(1, |count| count.get())
            .min(MAX_PARALLEL_CONVERSIONS)
    }
}

#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Without an output folder, the file is written next to the original.
    pub output_dir: Option<PathBuf>,
    /// From 1 to 100; ignored for PNG, which is lossless.
    pub quality: u8,
    pub format: FormatChoice,
    pub resize: Option<Resize>,
    /// Exact size, with cropping; replaces `resize`.
    pub frame: Option<Frame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionResult {
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    /// Saving as a percentage; negative if the converted file is larger.
    pub compression_ratio: f64,
    pub original_dimensions: (u32, u32),
    pub new_dimensions: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub thumbnail: RgbaImage,
}

pub fn extension(path: &Path) -> String {
    path.extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

pub fn assert_image_file(path: &Path) -> Result<Metadata, ConversionError> {
    if !path.is_absolute() {
        return Err(ConversionError::InvalidPath);
    }
    let extension = extension(path);
    if !INPUT_EXTENSIONS.contains(&extension.as_str()) {
        return Err(ConversionError::UnsupportedExtension(
            if extension.is_empty() {
                String::new()
            } else {
                format!(".{extension}")
            },
        ));
    }
    let metadata = fs::metadata(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => ConversionError::SourceNotFound(path.to_owned()),
        _ => ConversionError::Io(error),
    })?;
    if !metadata.is_file() {
        return Err(ConversionError::NotAFile(path.to_owned()));
    }
    Ok(metadata)
}

/// Decodes the image and applies the EXIF orientation to the pixels: metadata is not copied,
/// so portrait photos would otherwise come out sideways.
fn load_oriented(path: &Path) -> Result<RgbaImage, ConversionError> {
    let mut decoder = ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    // An unreadable profile must not prevent the conversion
    let icc_profile = decoder.icc_profile().ok().flatten();
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);

    let mut image = image.into_rgba8();
    if let Some(icc_profile) = icc_profile {
        convert_to_srgb(&mut image, &icc_profile);
    }
    Ok(image)
}

/// The color profile is not copied to the output file: the pixels of a Display P3 or Adobe
/// RGB photo are converted to sRGB, otherwise its colors would come out dull. Non-RGB
/// (grayscale, CMYK) or invalid profiles are ignored.
fn convert_to_srgb(image: &mut RgbaImage, icc_profile: &[u8]) {
    let Ok(source) = ColorProfile::new_from_slice(icc_profile) else {
        return;
    };
    let Ok(transform) = source.create_transform_8bit(
        Layout::Rgba,
        &ColorProfile::new_srgb(),
        Layout::Rgba,
        TransformOptions::default(),
    ) else {
        return;
    };
    let mut converted = vec![0; image.as_raw().len()];
    if transform.transform(image.as_raw(), &mut converted).is_ok() {
        image.copy_from_slice(&converted);
    }
}

fn resize_with(
    image: &RgbaImage,
    width: u32,
    height: u32,
    options: &ResizeOptions,
) -> Result<RgbaImage, ConversionError> {
    let resize_error = |error: &dyn std::fmt::Display| ConversionError::Resize(error.to_string());

    let source = ImageRef::new(
        image.width(),
        image.height(),
        image.as_raw(),
        PixelType::U8x4,
    )
    .map_err(|error| resize_error(&error))?;
    let mut target = Image::new(width, height, PixelType::U8x4);
    // Alpha is premultiplied during the computation: no dark fringe around transparent areas
    Resizer::new()
        .resize(&source, &mut target, options)
        .map_err(|error| resize_error(&error))?;

    RgbaImage::from_raw(width, height, target.into_vec())
        .ok_or_else(|| resize_error(&t!("error-resize-buffer")))
}

fn lanczos() -> ResizeOptions {
    ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3))
}

/// Output dimensions, refused before any allocation if the format or the memory cannot hold
/// them.
fn planned_dimensions(
    original: (u32, u32),
    options: &ConvertOptions,
    format: OutputFormat,
) -> Result<(u32, u32), ConversionError> {
    let (width, height) = match (&options.frame, &options.resize) {
        (Some(frame), _) => frame_dimensions(original.0, original.1, frame),
        (None, Some(resize)) => target_dimensions(original.0, original.1, resize),
        (None, None) => original,
    };
    assert_fits(width, height, format)?;
    if (width, height) != original && u64::from(width) * u64::from(height) > MAX_OUTPUT_PIXELS {
        return Err(ConversionError::TooManyPixels {
            width,
            height,
            max: MAX_OUTPUT_PIXELS / 1_000_000,
        });
    }
    Ok((width, height))
}

/// Resizes to the planned dimensions; with `crop`, the edges beyond the target proportions are
/// cut around the center instead of distorting the image.
fn resize_to(
    image: RgbaImage,
    (width, height): (u32, u32),
    crop: bool,
) -> Result<RgbaImage, ConversionError> {
    if (width, height) == image.dimensions() {
        return Ok(image);
    }
    if crop {
        let options = lanczos().fit_into_destination(Some((0.5, 0.5)));
        resize_with(&image, width, height, &options)
    } else {
        resize_with(&image, width, height, &lanczos())
    }
}

/// Square thumbnail, cropped to the center.
fn create_thumbnail(image: &RgbaImage) -> Result<RgbaImage, ConversionError> {
    let options = lanczos().fit_into_destination(Some((0.5, 0.5)));
    resize_with(image, THUMBNAIL_SIZE, THUMBNAIL_SIZE, &options)
}

/// Dimensions (EXIF orientation applied), size and thumbnail of an image added to the list.
pub fn image_info(path: &Path) -> Result<ImageInfo, ConversionError> {
    let metadata = assert_image_file(path)?;
    let image = load_oriented(path)?;
    Ok(ImageInfo {
        width: image.width(),
        height: image.height(),
        size: metadata.len(),
        thumbnail: create_thumbnail(&image)?,
    })
}

// JPEG has no transparency: white background rather than black
fn flatten_on_white(image: &RgbaImage) -> RgbImage {
    RgbImage::from_fn(image.width(), image.height(), |x, y| {
        let [r, g, b, a] = image.get_pixel(x, y).0;
        let alpha = u32::from(a);
        let blend =
            |channel: u8| ((u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
        image::Rgb([blend(r), blend(g), blend(b)])
    })
}

fn is_opaque(image: &RgbaImage) -> bool {
    image.pixels().all(|pixel| pixel[3] == u8::MAX)
}

fn assert_fits(width: u32, height: u32, format: OutputFormat) -> Result<(), ConversionError> {
    let max = format.max_dimension();
    if width > max || height > max {
        return Err(ConversionError::TooLarge {
            width,
            height,
            format: format.name(),
            max,
        });
    }
    Ok(())
}

fn encode(
    image: &RgbaImage,
    format: OutputFormat,
    quality: u8,
) -> Result<Vec<u8>, ConversionError> {
    let (width, height) = image.dimensions();
    let encode_error = |error: &dyn std::fmt::Display| ConversionError::Encode(error.to_string());

    match format {
        OutputFormat::Webp => {
            let encoder = webp::Encoder::from_rgba(image, width, height);
            Ok(encoder.encode(f32::from(quality)).to_vec())
        }
        OutputFormat::Jpg => {
            // 4:2:0 subsampling and optimized Huffman tables: files about a third smaller
            let mut bytes = Vec::new();
            let mut encoder = jpeg_encoder::Encoder::new(&mut bytes, quality);
            encoder.set_sampling_factor(jpeg_encoder::SamplingFactor::F_2_2);
            encoder.set_optimized_huffman_tables(true);
            encoder
                .encode(
                    &flatten_on_white(image),
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Rgb,
                )
                .map_err(|error| encode_error(&error))?;
            Ok(bytes)
        }
        OutputFormat::Png => {
            // Lossless PNG: quality does not apply. The highest level (Best) is twice as slow for
            // files only 1 to 2% smaller
            let mut bytes = Cursor::new(Vec::new());
            let encoder = PngEncoder::new_with_quality(
                &mut bytes,
                CompressionType::Default,
                PngFilter::Adaptive,
            );
            if is_opaque(image) {
                let rgb = DynamicImage::ImageRgba8(image.clone()).into_rgb8();
                encoder.write_image(&rgb, width, height, ExtendedColorType::Rgb8)
            } else {
                encoder.write_image(image, width, height, ExtendedColorType::Rgba8)
            }
            .map_err(|error| encode_error(&error))?;
            Ok(bytes.into_inner())
        }
        OutputFormat::Avif => {
            let pixels: Vec<ravif::RGBA8> = image
                .pixels()
                .map(|pixel| ravif::RGBA8::new(pixel[0], pixel[1], pixel[2], pixel[3]))
                .collect();
            let encoded = ravif::Encoder::new()
                .with_quality(f32::from(quality))
                .with_speed(6)
                .encode_rgba(ravif::Img::new(&pixels, width as usize, height as usize))
                .map_err(|error| encode_error(&error))?;
            Ok(encoded.avif_file)
        }
    }
}

struct Encoded {
    bytes: Vec<u8>,
    original_dimensions: (u32, u32),
    new_dimensions: (u32, u32),
}

fn encode_still(
    image: RgbaImage,
    format: OutputFormat,
    options: &ConvertOptions,
) -> Result<Encoded, ConversionError> {
    let original_dimensions = image.dimensions();
    let target = planned_dimensions(original_dimensions, options, format)?;
    let image = resize_to(image, target, options.frame.is_some())?;
    Ok(Encoded {
        bytes: encode(&image, format, options.quality)?,
        original_dimensions,
        new_dimensions: image.dimensions(),
    })
}

/// Converts a GIF to an animated WebP, one frame at a time to bound memory. A single-frame
/// GIF gives a still WebP.
fn encode_animated_webp(
    mut decoder: GifDecoder<BufReader<File>>,
    options: &ConvertOptions,
) -> Result<Encoded, ConversionError> {
    let encode_error = ConversionError::Encode;
    decoder.set_limits(Limits::default())?;

    let original_dimensions = decoder.dimensions();
    let (width, height) = planned_dimensions(original_dimensions, options, OutputFormat::Webp)?;
    let loop_count = match decoder.loop_count() {
        LoopCount::Infinite => 0,
        LoopCount::Finite(count) => i32::try_from(count.get()).unwrap_or(i32::MAX),
    };
    let mut encoder =
        AnimatedWebpEncoder::new(width, height, f32::from(options.quality), loop_count)
            .map_err(encode_error)?;

    // Display time of each frame, in milliseconds since the start
    let mut timestamp: i32 = 0;
    let mut frame_count = 0;
    for frame in decoder.into_frames() {
        let frame = frame?;
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay = numerator / denominator.max(1);
        // Like browsers: a zero or near-zero delay means a tenth of a second
        let delay = if delay <= MIN_FRAME_DELAY_MS {
            DEFAULT_FRAME_DELAY_MS
        } else {
            delay
        };

        let frame = resize_to(
            frame.into_buffer(),
            (width, height),
            options.frame.is_some(),
        )?;
        encoder
            .add_frame(frame.as_raw(), timestamp)
            .map_err(encode_error)?;
        timestamp = timestamp.saturating_add(i32::try_from(delay).unwrap_or(i32::MAX));
        frame_count += 1;
    }
    if frame_count == 0 {
        return Err(ConversionError::EmptyOutput);
    }

    Ok(Encoded {
        bytes: encoder.finish(timestamp).map_err(encode_error)?,
        original_dimensions,
        new_dimensions: (width, height),
    })
}

/// Never overwrites an existing file (nor the original): photo.webp, then photo-1.webp,
/// photo-2.webp… Creation is atomic: two simultaneous conversions cannot pick the same name.
pub fn write_new_file(
    dir: &Path,
    name: &str,
    extension: &str,
    bytes: &[u8],
) -> Result<PathBuf, ConversionError> {
    for index in 0u32.. {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("-{index}")
        };
        let candidate = dir.join(format!("{name}{suffix}.{extension}"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(bytes) {
                    drop(file);
                    // Do not leave a truncated file behind
                    let _ = fs::remove_file(&candidate);
                    return Err(error.into());
                }
                return Ok(candidate);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("the loop only ends with a return")
}

/// Converts an image, resizing it if the options ask for it.
pub fn convert_image(
    path: &Path,
    options: &ConvertOptions,
) -> Result<ConversionResult, ConversionError> {
    if !(1..=100).contains(&options.quality) {
        return Err(ConversionError::InvalidQuality);
    }
    if let Some(resize) = &options.resize {
        resize.validate()?;
    }
    if let Some(frame) = &options.frame {
        frame.validate()?;
    }
    let original_size = assert_image_file(path)?.len();
    let format = options.format.resolve(path);

    // Only WebP keeps the animation of a GIF; the other formats keep the first frame. The GIF is
    // recognized by its content, as for the thumbnail, not by its extension
    let reader = ImageReader::open(path)?.with_guessed_format()?;
    let encoded = if format == OutputFormat::Webp && reader.format() == Some(ImageFormat::Gif) {
        encode_animated_webp(GifDecoder::new(reader.into_inner())?, options)?
    } else {
        encode_still(load_oriented(path)?, format, options)?
    };
    if encoded.bytes.is_empty() {
        return Err(ConversionError::EmptyOutput);
    }

    let output_dir = match &options.output_dir {
        Some(dir) => dir.as_path(),
        None => path.parent().ok_or(ConversionError::InvalidPath)?,
    };
    let name = path
        .file_stem()
        .ok_or(ConversionError::InvalidPath)?
        .to_string_lossy();
    let output_path = write_new_file(output_dir, &name, format.extension(), &encoded.bytes)?;

    let new_size = encoded.bytes.len() as u64;
    Ok(ConversionResult {
        output_path,
        original_size,
        new_size,
        compression_ratio: (1.0 - new_size as f64 / original_size as f64) * 100.0,
        original_dimensions: encoded.original_dimensions,
        new_dimensions: encoded.new_dimensions,
    })
}
