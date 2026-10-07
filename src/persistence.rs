use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static BACKUP_NUMBER: AtomicU64 = AtomicU64::new(0);

/// Write through existing symlinks and retain a unique recovery backup on failure.
/// Renaming a temporary file over the destination would replace a symlink itself.
pub fn save(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let backup = if path.try_exists()? {
        Some(backup(path)?)
    } else {
        None
    };
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.write_all(bytes)?;
        file.sync_all()
    })();
    match result {
        Ok(()) => {
            if let Some(backup) = backup {
                let _ = fs::remove_file(backup);
            }
            Ok(())
        }
        Err(error) => Err(io::Error::new(
            error.kind(),
            match backup {
                Some(backup) => format!("{error}; original preserved at {}", backup.display()),
                None => error.to_string(),
            },
        )),
    }
}

fn backup(path: &Path) -> io::Result<PathBuf> {
    let mut name = std::ffi::OsString::from(".");
    name.push(path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Output path has no filename")
    })?);
    name.push(format!(
        ".sortini.{}.{}.bak",
        std::process::id(),
        BACKUP_NUMBER.fetch_add(1, Ordering::Relaxed)
    ));
    let destination = path.with_file_name(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)?;
    let result = (|| {
        io::copy(&mut fs::File::open(path)?, &mut file)?;
        file.sync_all()
    })();
    drop(file);
    if let Err(error) = result {
        let _ = fs::remove_file(&destination);
        return Err(error);
    }
    Ok(destination)
}
