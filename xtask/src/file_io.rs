use std::fs;
use std::io::Write as _;
use std::path::Path;

use atomic_write_file::AtomicWriteFile;

use crate::XtaskError;

pub(crate) fn write_file(path: &Path, content: &[u8]) -> Result<(), XtaskError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = AtomicWriteFile::open(path)?;
    file.write_all(content)?;
    file.commit()?;
    Ok(())
}

pub(crate) fn copy_file(source: &Path, destination: &Path) -> Result<(), XtaskError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

pub(crate) fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), XtaskError> {
    fs::create_dir_all(destination)?;
    let mut entries = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            copy_file(&from, &to)?;
        }
    }
    Ok(())
}
