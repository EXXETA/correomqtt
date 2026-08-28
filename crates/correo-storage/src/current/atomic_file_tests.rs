use super::atomic_file::{write_file_atomic, write_file_atomic_private};

fn temporary_path() -> std::path::PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("correomqtt-atomic-file-{suffix}"))
}

#[test]
fn atomic_write_replaces_existing_content() {
    let path = temporary_path();
    std::fs::write(&path, "old").unwrap();

    write_file_atomic(&path, b"new").unwrap();

    assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    std::fs::remove_file(path).unwrap();
}

#[cfg(unix)]
#[test]
fn private_atomic_write_restricts_existing_file_permissions() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    let path = temporary_path();
    std::fs::write(&path, "old").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    write_file_atomic_private(&path, b"new").unwrap();

    assert_eq!(std::fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
    std::fs::remove_file(path).unwrap();
}
