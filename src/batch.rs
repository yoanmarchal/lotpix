// Batches of images coming from files and folders: list of the images to process, then
// conversion without the interface (context menu, command line)
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::conversion::{
    self, ConversionError, ConversionResult, ConvertOptions, FormatChoice, INPUT_EXTENSIONS,
};

/// Image to convert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub source: PathBuf,
    /// Path of the image folder relative to the selected folder, empty for a file selected
    /// directly: it is recreated in the output folder.
    pub subdir: PathBuf,
}

impl Job {
    pub fn file(source: PathBuf) -> Self {
        Self {
            source,
            subdir: PathBuf::new(),
        }
    }
}

/// Images of a selection, in order and without duplicates, and the number of selected files
/// refused. In a folder and its subfolders, only the images are kept, without counting the
/// other files as refused.
///
/// The list is built before any conversion: an image written in a folder being walked is
/// not processed a second time.
pub fn collect(paths: &[PathBuf]) -> (Vec<Job>, usize) {
    let mut seen = HashSet::new();
    let mut jobs = Vec::new();
    let mut ignored = 0;
    for path in paths {
        if path.is_dir() {
            jobs.extend(
                find_images(path)
                    .into_iter()
                    .filter(|job| seen.insert(job.source.clone())),
            );
        } else if is_supported(path) {
            if seen.insert(path.clone()) {
                jobs.push(Job::file(path.clone()));
            }
        } else {
            ignored += 1;
        }
    }
    (jobs, ignored)
}

/// Images of a folder and its subfolders, sorted by path. Symbolic links to folders are
/// not followed: they could loop.
pub fn find_images(root: &Path) -> Vec<Job> {
    let mut jobs = Vec::new();
    let mut pending = vec![PathBuf::new()];
    while let Some(subdir) = pending.pop() {
        let Ok(entries) = fs::read_dir(root.join(&subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_dir() {
                pending.push(subdir.join(entry.file_name()));
            } else if is_supported(&path) && path.is_file() {
                jobs.push(Job {
                    source: path,
                    subdir: subdir.clone(),
                });
            }
        }
    }
    jobs.sort_by(|a, b| a.source.cmp(&b.source));
    jobs
}

fn is_supported(path: &Path) -> bool {
    INPUT_EXTENSIONS.contains(&conversion::extension(path).as_str())
}

/// Converts an image; with an output folder, the result goes to the subfolder matching
/// the image's one, created if needed.
pub fn convert(job: &Job, options: &ConvertOptions) -> Result<ConversionResult, ConversionError> {
    match &options.output_dir {
        Some(output_dir) if !job.subdir.as_os_str().is_empty() => {
            let output_dir = output_dir.join(&job.subdir);
            fs::create_dir_all(&output_dir)?;
            let options = ConvertOptions {
                output_dir: Some(output_dir),
                ..options.clone()
            };
            conversion::convert_image(&job.source, &options)
        }
        _ => conversion::convert_image(&job.source, options),
    }
}

/// Converts all the images, several at a time. The results are in the order of the images.
pub fn convert_all(
    jobs: &[Job],
    options: &ConvertOptions,
) -> Vec<Result<ConversionResult, ConversionError>> {
    convert_all_with(jobs, options.format, |_| options.clone())
}

/// Like `convert_all`, with options specific to each image. `format` sets the number of
/// conversions run at the same time.
pub fn convert_all_with(
    jobs: &[Job],
    format: FormatChoice,
    options_for: impl Fn(&Job) -> ConvertOptions + Sync,
) -> Vec<Result<ConversionResult, ConversionError>> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new((0..jobs.len()).map(|_| None).collect::<Vec<_>>());
    let workers = conversion::parallel_conversions(format).min(jobs.len());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(job) = jobs.get(index) else {
                        break;
                    };
                    let result = convert(job, &options_for(job));
                    results.lock().unwrap()[index] = Some(result);
                }
            });
        }
    });
    results
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|result| result.expect("every image is processed"))
        .collect()
}
