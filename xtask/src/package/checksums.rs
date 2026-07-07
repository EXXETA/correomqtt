use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::XtaskError;

pub(crate) fn write_sidecar(artifact: &Path, out_dir: &Path) -> Result<String, XtaskError> {
    let checksum = sha256_file(artifact)?;
    let file_name = artifact_file_name(artifact)?;
    write_file_atomic(
        &out_dir.join(format!("{file_name}.sha256")),
        format!("{checksum}  {file_name}\n").as_bytes(),
    )?;
    Ok(checksum)
}

// Release artifact extensions that belong in SHA256SUMS.
const ARTIFACT_EXTENSIONS: &[&str] = &["zip", "dmg", "deb", "rpm", "msi"];

pub(crate) fn write_checksum_files(artifact: &Path, out_dir: &Path) -> Result<String, XtaskError> {
    write_sidecar(artifact, out_dir)
}

/// Regenerates SHA256SUMS over every release artifact in `out_dir`. Must run
/// after all artifacts (zip + any dmg/deb/rpm/msi) have been written, so the
/// summary is complete rather than zip-only.
pub(crate) fn write_sha256sums(out_dir: &Path) -> Result<(), XtaskError> {
    let mut checksum_entries = Vec::new();
    for entry in fs::read_dir(out_dir)? {
        let path = entry?.path();
        let is_artifact = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| ARTIFACT_EXTENSIONS.contains(&extension));
        if is_artifact {
            let checksum = sha256_file(&path)?;
            let file_name = artifact_file_name(&path)?;
            checksum_entries.push((file_name, checksum));
        }
    }
    checksum_entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut summary = String::new();
    for (file_name, checksum) in checksum_entries {
        writeln!(&mut summary, "{checksum}  {file_name}").expect("writing to a String cannot fail");
    }
    write_file_atomic(&out_dir.join("SHA256SUMS"), summary.as_bytes())?;
    Ok(())
}

fn write_file_atomic(path: &Path, content: &[u8]) -> Result<(), XtaskError> {
    let temporary = temporary_path(path);
    fs::write(&temporary, content)?;
    replace_file(&temporary, path)?;
    Ok(())
}

fn temporary_path(path: &Path) -> std::path::PathBuf {
    let suffix = format!("tmp.{}", std::process::id());
    path.with_extension(suffix)
}

fn replace_file(source: &Path, destination: &Path) -> Result<(), XtaskError> {
    if cfg!(windows) && destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(source, destination)?;
    Ok(())
}

pub(crate) fn sha256_file(path: &Path) -> Result<String, XtaskError> {
    let mut file = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_digest(&hasher.finalize()))
}

pub(crate) fn hex_digest(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}

pub(crate) fn artifact_file_name(path: &Path) -> Result<String, XtaskError> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| XtaskError::InvalidArguments("artifact has no file name".to_owned()))
}
