// Time and output size of each format, to compare encoder settings.
// cargo run --release --example compare -- <output folder> <images…>
// The QUALITY environment variable sets the quality (80 by default).
use std::path::PathBuf;
use std::time::Instant;

use lotpix::conversion::{ConvertOptions, FormatChoice, OutputFormat, convert_image, image_info};
use lotpix::format::{format_dimensions, format_file_size};

fn main() {
    let mut args = std::env::args().skip(1);
    let output_dir = PathBuf::from(args.next().expect("missing output folder"));
    let quality = std::env::var("QUALITY").map_or(80, |value| value.parse().expect("QUALITY"));

    for input in args {
        // Relative paths are refused by the conversion
        let input = std::path::absolute(&input).expect("path");
        let input = input.as_path();

        let start = Instant::now();
        let info = image_info(input).expect("read");
        println!(
            "{}: {}, {} (read and thumbnail in {:?})",
            input.display(),
            format_dimensions((info.width, info.height)),
            format_file_size(info.size),
            start.elapsed()
        );

        for format in OutputFormat::ALL {
            let options = ConvertOptions {
                output_dir: Some(output_dir.clone()),
                quality,
                format: FormatChoice::Fixed(format),
                resize: None,
                frame: None,
            };
            let start = Instant::now();
            let result = convert_image(input, &options).expect("conversion");
            println!(
                "  {}: {} bytes in {:?}",
                format.extension(),
                result.new_size,
                start.elapsed()
            );
        }
    }
}
