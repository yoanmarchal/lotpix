// Lots d'images venant de fichiers et de dossiers : liste des images à traiter, puis
// conversion sans interface (menu contextuel, ligne de commande)
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::conversion::{
    self, ConversionError, ConversionResult, ConvertOptions, FormatChoice, INPUT_EXTENSIONS,
};

/// Image à convertir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub source: PathBuf,
    /// Chemin du dossier de l'image relatif au dossier sélectionné, vide pour un fichier
    /// sélectionné directement : il est recréé dans le dossier de sortie.
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

/// Images d'une sélection, dans l'ordre et sans doublon, et nombre de fichiers sélectionnés
/// refusés. Dans un dossier et ses sous-dossiers, seules les images sont gardées, sans
/// compter les autres fichiers comme refusés.
///
/// La liste est établie avant toute conversion : une image écrite dans un dossier parcouru
/// n'est pas traitée une seconde fois.
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

/// Images d'un dossier et de ses sous-dossiers, triées par chemin. Les liens symboliques vers
/// des dossiers ne sont pas suivis : ils pourraient boucler.
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

/// Convertit une image ; avec un dossier de sortie, le résultat va dans le sous-dossier
/// correspondant à celui de l'image, créé au besoin.
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

/// Convertit toutes les images, plusieurs à la fois. Les résultats sont dans l'ordre des images.
pub fn convert_all(
    jobs: &[Job],
    options: &ConvertOptions,
) -> Vec<Result<ConversionResult, ConversionError>> {
    convert_all_with(jobs, options.format, |_| options.clone())
}

/// Comme `convert_all`, avec des options propres à chaque image. `format` règle le nombre de
/// conversions menées de front.
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
        .map(|result| result.expect("chaque image est traitée"))
        .collect()
}
