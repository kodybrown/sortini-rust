use sortini::persistence::save;
use std::fs;

#[test]
fn success_removes_backup_and_preserves_unrelated_backup() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("settings.ini");
    let unrelated = temp.path().join(".settings.ini.sortini.bak");
    fs::write(&path, b"original").unwrap();
    fs::write(&unrelated, b"user owned").unwrap();
    save(&path, b"new").unwrap();
    assert_eq!(fs::read(path).unwrap(), b"new");
    assert_eq!(fs::read(unrelated).unwrap(), b"user owned");
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
}

#[cfg(windows)]
#[test]
fn failed_write_retains_recovery_backup() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("readonly.ini");
    fs::write(&path, b"original").unwrap();
    let original_permissions = fs::metadata(&path).unwrap().permissions();
    let mut permissions = original_permissions.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions.clone()).unwrap();
    let result = save(&path, b"new");
    fs::set_permissions(&path, original_permissions).unwrap();
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("original preserved at")
    );
    assert_eq!(fs::read(&path).unwrap(), b"original");
    let backup = fs::read_dir(temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p != &path)
        .unwrap();
    assert_eq!(fs::read(backup).unwrap(), b"original");
}

#[test]
fn existing_symlink_is_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("target.ini");
    let link = temp.path().join("link.ini");
    fs::write(&target, b"original").unwrap();
    #[cfg(windows)]
    let result = std::os::windows::fs::symlink_file(&target, &link);
    #[cfg(unix)]
    let result = std::os::unix::fs::symlink(&target, &link);
    if let Err(error) = result {
        assert!(
            error.kind() == std::io::ErrorKind::PermissionDenied
                || (cfg!(windows) && error.raw_os_error() == Some(1314)),
            "unexpected symlink error: {error}"
        );
        eprintln!("Symlink check skipped: OS permission required ({error})");
        return;
    }
    save(&link, b"new").unwrap();
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(target).unwrap(), b"new");
}

#[test]
fn existing_hardlink_is_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("target.ini");
    let link = temp.path().join("link.ini");
    fs::write(&target, b"original").unwrap();
    fs::hard_link(&target, &link).unwrap();
    save(&link, b"new").unwrap();
    assert_eq!(fs::read(target).unwrap(), b"new");
}
