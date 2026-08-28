use crate::{Result, StorageError};
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub(super) fn collect_script_paths(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|source| StorageError::Read {
        path: dir.to_path_buf(),
        source,
    })? {
        let path = entry
            .map_err(|source| StorageError::Read {
                path: dir.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            if is_sidecar_dir(root, &path) {
                continue;
            }
            collect_script_paths(root, &path, out)?;
        } else if path.extension() == Some(OsStr::new("js")) {
            out.push(relative_script_path(root, &path));
        }
    }
    Ok(())
}

fn relative_script_path(root: &Path, path: &Path) -> PathBuf {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
        .into()
}

pub fn redact_script_log_text(input: &str) -> String {
    input
        .lines()
        .map(redact_script_log_line)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn normalize_script_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.extension() != Some(OsStr::new("js"))
        || path.to_string_lossy().contains('\\')
    {
        return Err(StorageError::InvalidScriptFileName(display_path(path)));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(StorageError::InvalidScriptFileName(display_path(path))),
        }
    }
    Ok(path.to_path_buf())
}

pub(super) fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    Ok(())
}

pub(super) fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| StorageError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

pub(super) fn rename_path(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to).map_err(|source| StorageError::Rename {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        source,
    })
}

pub(super) fn remove_dir_if_exists(path: PathBuf) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(&path).map_err(|source| StorageError::Delete { path, source })?;
    }
    Ok(())
}

pub(super) fn remove_file_if_exists(path: PathBuf) -> Result<()> {
    if path.exists() {
        fs::remove_file(&path).map_err(|source| StorageError::Delete { path, source })?;
    }
    Ok(())
}

pub(super) fn write_text(path: &Path, text: &str) -> Result<()> {
    let mut file = fs::File::create(path).map_err(|source| StorageError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| StorageError::Write {
            path: path.to_path_buf(),
            source,
        })
}

pub(super) fn sync_parent_dir(path: &Path) -> Result<()> {
    #[cfg(not(windows))]
    {
        let parent = path.parent().unwrap_or(path);
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|source| StorageError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
    }
    Ok(())
}

pub(super) fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn is_sidecar_dir(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    matches!(
        relative.components().next(),
        Some(Component::Normal(name)) if name == "logs" || name == "executions"
    )
}

fn redact_script_log_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if lower.contains("-----begin") && lower.contains("private key") {
        return "[REDACTED KEY MATERIAL]".to_owned();
    }
    if !contains_sensitive_term(&lower) {
        return line.to_owned();
    }
    if let Some(index) = line.find(['=', ':']) {
        let (label, _) = line.split_at(index + 1);
        format!("{label} [REDACTED]")
    } else {
        "[REDACTED SCRIPT LOG: sensitive material]".to_owned()
    }
}

fn contains_sensitive_term(lowercase_line: &str) -> bool {
    [
        "password",
        "passphrase",
        "private key",
        "key material",
        "key_material",
        "keymaterial",
        "decrypted password map",
        "export password",
    ]
    .iter()
    .any(|term| lowercase_line.contains(term))
}
