use std::fs;
use std::path::{Path, PathBuf};

use image::{Rgb, RgbImage};
use lotpix::batch::{Job, collect, convert_all, find_images};
use lotpix::conversion::{ConvertOptions, FormatChoice, OutputFormat};
use tempfile::TempDir;

fn temp_dir() -> TempDir {
    tempfile::Builder::new()
        .prefix("lotpix-test-")
        .tempdir()
        .unwrap()
}

fn create_image(path: &Path) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    RgbImage::from_pixel(8, 8, Rgb([0x33, 0x66, 0xcc]))
        .save(path)
        .unwrap();
    path.to_owned()
}

fn webp(output_dir: Option<PathBuf>) -> ConvertOptions {
    ConvertOptions {
        output_dir,
        quality: 80,
        format: FormatChoice::Fixed(OutputFormat::Webp),
        resize: None,
        frame: None,
    }
}

#[test]
fn finds_images_in_subfolders_only() {
    let dir = temp_dir();
    let root = dir.path().join("photos");
    let a = create_image(&root.join("a.png"));
    let b = create_image(&root.join("vacances").join("plage").join("b.jpg"));
    fs::write(root.join("notes.txt"), "pas une image").unwrap();

    let jobs = find_images(&root);

    assert_eq!(
        jobs,
        [
            Job::file(a),
            Job {
                source: b,
                subdir: PathBuf::from("vacances").join("plage"),
            },
        ]
    );
}

#[test]
fn collects_files_and_folders_without_duplicates() {
    let dir = temp_dir();
    let root = dir.path().join("photos");
    let a = create_image(&root.join("a.png"));
    let notes = dir.path().join("notes.txt");
    fs::write(&notes, "pas une image").unwrap();

    // L'image est sélectionnée deux fois : seule, et par son dossier
    let (jobs, ignored) = collect(&[a.clone(), root, notes]);

    assert_eq!(jobs, [Job::file(a)]);
    assert_eq!(ignored, 1);
}

#[test]
fn rebuilds_the_folder_tree_in_the_output_folder() {
    let dir = temp_dir();
    let root = dir.path().join("photos");
    create_image(&root.join("a.png"));
    create_image(&root.join("vacances").join("a.png"));
    let output = dir.path().join("sortie");
    fs::create_dir_all(&output).unwrap();

    let jobs = find_images(&root);
    let results = convert_all(&jobs, &webp(Some(output.clone())));

    let outputs: Vec<PathBuf> = results
        .into_iter()
        .map(|result| result.unwrap().output_path)
        .collect();
    assert_eq!(
        outputs,
        [
            output.join("a.webp"),
            output.join("vacances").join("a.webp"),
        ]
    );
}

#[test]
fn writes_next_to_each_original_without_output_folder() {
    let dir = temp_dir();
    let root = dir.path().join("photos");
    create_image(&root.join("vacances").join("a.png"));

    let jobs = find_images(&root);
    let results = convert_all(&jobs, &webp(None));

    assert_eq!(
        results[0].as_ref().unwrap().output_path,
        root.join("vacances").join("a.webp")
    );
}

#[test]
fn keeps_failures_in_order() {
    let dir = temp_dir();
    let good = create_image(&dir.path().join("a.png"));
    let broken = dir.path().join("b.png");
    fs::write(&broken, "pas une image").unwrap();

    let results = convert_all(&[Job::file(good), Job::file(broken)], &webp(None));

    assert!(results[0].is_ok());
    assert!(results[1].is_err());
}
