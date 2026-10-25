// Regroupement des fichiers reçus par des processus lancés ensemble : l'Explorateur Windows
// lance une fois l'application par fichier sélectionné dans le menu contextuel
//
// Chaque processus dépose ses chemins dans le dossier de la file, puis tente de prendre le
// verrou. Celui qui l'obtient attend que les dépôts cessent et traite tout ; les autres
// s'arrêtent sans rien faire.
use std::fs::{self, File, TryLockError};
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::storage;

const LOCK_FILE: &str = "lock";
const ENTRY_EXTENSION: &str = "json";

/// Les dépôts sont jugés terminés quand aucun n'arrive pendant ce délai.
const SETTLE_DELAY: Duration = Duration::from_millis(400);
/// Attente maximale des dépôts : au-delà, les retardataires forment un nouveau lot.
const MAX_WAIT: Duration = Duration::from_secs(5);
/// Un dépôt plus ancien vient d'un processus interrompu avant d'être traité : il est jeté
/// plutôt que de compresser des images sélectionnées il y a longtemps.
const STALE_AFTER: Duration = Duration::from_secs(60);

/// Chemins de tous les processus regroupés si celui-ci est chargé de les traiter, ou liste
/// vide si un autre processus s'en occupe.
pub fn gather(dir: &Path, paths: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    gather_with(dir, paths, SETTLE_DELAY)
}

fn gather_with(dir: &Path, paths: &[PathBuf], settle_delay: Duration) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    enqueue(dir, paths)?;

    let mut gathered = Vec::new();
    loop {
        let lock = File::create(dir.join(LOCK_FILE))?;
        match lock.try_lock() {
            Ok(()) => {}
            // Le processus qui tient le verrou lira notre dépôt
            Err(TryLockError::WouldBlock) => return Ok(gathered),
            Err(TryLockError::Error(error)) => return Err(error),
        }
        wait_until_settled(dir, settle_delay);
        gathered.extend(drain(dir)?);
        drop(lock);

        // Un dépôt arrivé entre la lecture et la libération du verrou n'a pas trouvé de
        // processus pour le traiter : on recommence plutôt que de l'abandonner
        if entries(dir)?.is_empty() {
            return Ok(gathered);
        }
    }
}

fn enqueue(dir: &Path, paths: &[PathBuf]) -> io::Result<()> {
    let name = format!("{}.{ENTRY_EXTENSION}", std::process::id());
    // Écriture puis renommage : le dépôt n'est jamais lu à moitié écrit
    storage::save(&dir.join(name), paths)
}

fn entries(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == ENTRY_EXTENSION)
        })
        .collect();
    entries.sort();
    Ok(entries)
}

fn wait_until_settled(dir: &Path, settle_delay: Duration) {
    let start = SystemTime::now();
    let mut count = entries(dir).map_or(0, |entries| entries.len());
    loop {
        thread::sleep(settle_delay);
        let new_count = entries(dir).map_or(0, |entries| entries.len());
        let waited = start.elapsed().unwrap_or_default();
        if new_count == count || waited >= MAX_WAIT {
            return;
        }
        count = new_count;
    }
}

/// Lit et supprime les dépôts, du plus ancien processus au plus récent.
fn drain(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries: Vec<(SystemTime, PathBuf)> = entries(dir)?
        .into_iter()
        .map(|path| {
            let modified = fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            (modified, path)
        })
        .collect();
    entries.sort();

    let mut paths = Vec::new();
    for (modified, entry) in entries {
        let is_stale = modified.elapsed().is_ok_and(|age| age > STALE_AFTER);
        if !is_stale {
            paths.extend(storage::read::<Vec<PathBuf>>(&entry).unwrap_or_default());
        }
        fs::remove_file(&entry)?;
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAST: Duration = Duration::from_millis(50);

    #[test]
    fn a_lone_process_gets_its_own_paths() {
        let dir = tempfile::tempdir().unwrap();
        let paths = vec![
            PathBuf::from("/images/a.png"),
            PathBuf::from("/images/b.png"),
        ];

        assert_eq!(gather_with(dir.path(), &paths, FAST).unwrap(), paths);
        assert!(entries(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn the_lock_holder_collects_the_other_deposits() {
        let dir = tempfile::tempdir().unwrap();
        // Dépôt d'un autre processus, arrivé juste avant
        storage::save(
            &dir.path().join("1.json"),
            &[PathBuf::from("/images/a.png")],
        )
        .unwrap();

        let gathered = gather_with(dir.path(), &[PathBuf::from("/images/b.png")], FAST).unwrap();

        assert_eq!(gathered.len(), 2);
        assert!(gathered.contains(&PathBuf::from("/images/a.png")));
        assert!(gathered.contains(&PathBuf::from("/images/b.png")));
    }

    #[test]
    fn leaves_its_paths_to_the_lock_holder() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        let holder = File::create(dir.path().join(LOCK_FILE)).unwrap();
        holder.lock().unwrap();

        let gathered = gather_with(dir.path(), &[PathBuf::from("/images/a.png")], FAST).unwrap();

        assert!(gathered.is_empty());
        assert_eq!(entries(dir.path()).unwrap().len(), 1);
        drop(holder);
    }
}
