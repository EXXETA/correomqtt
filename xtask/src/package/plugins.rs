use std::fs;
use std::path::{Path, PathBuf};

use crate::XtaskError;

use super::{Platform, APP_NAME};

const LOCAL_REPOSITORY_FILE: &str = "local-repo.json";

pub(super) fn stage(
    platform: Platform,
    stage_dir: &Path,
    source_executable_dir: &Path,
) -> Result<(), XtaskError> {
    let source_plugins = source_executable_dir.join("plugins");
    let source_repository = source_executable_dir.join(LOCAL_REPOSITORY_FILE);
    if !source_plugins.is_dir() {
        return Err(XtaskError::MissingArtifact(
            source_plugins.display().to_string(),
        ));
    }
    if !source_repository.is_file() {
        return Err(XtaskError::MissingArtifact(
            source_repository.display().to_string(),
        ));
    }

    let executable_dir = executable_dir(platform, stage_dir);
    copy_dir(&source_plugins, &executable_dir.join("plugins"))?;
    fs::copy(
        source_repository,
        executable_dir.join(LOCAL_REPOSITORY_FILE),
    )?;
    Ok(())
}

fn executable_dir(platform: Platform, stage_dir: &Path) -> PathBuf {
    match platform {
        Platform::Linux => stage_dir.join(APP_NAME).join("bin"),
        Platform::Macos => stage_dir
            .join(format!("{APP_NAME}.app"))
            .join("Contents/MacOS"),
        Platform::Windows => stage_dir.join(APP_NAME),
    }
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), XtaskError> {
    fs::create_dir_all(destination)?;
    let mut entries = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let source = entry.path();
        let destination = destination.join(entry.file_name());
        if source.is_dir() {
            copy_dir(&source, &destination)?;
        } else {
            fs::copy(source, destination)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_repository_and_nested_plugins_from_application_output() {
        let root = temporary_root("stage-plugins");
        let source = root.join("source");
        fs::create_dir_all(source.join("plugins/org.example.demo/assets")).unwrap();
        fs::write(source.join(LOCAL_REPOSITORY_FILE), b"{\"plugins\":[]}").unwrap();
        fs::write(
            source.join("plugins/org.example.demo/plugin.wasm"),
            b"\0asm",
        )
        .unwrap();
        fs::write(
            source.join("plugins/org.example.demo/assets/example.txt"),
            b"asset",
        )
        .unwrap();

        for (platform, directory) in [
            (Platform::Linux, PathBuf::from("CorreoMQTT/bin")),
            (
                Platform::Macos,
                PathBuf::from("CorreoMQTT.app/Contents/MacOS"),
            ),
            (Platform::Windows, PathBuf::from("CorreoMQTT")),
        ] {
            let stage_dir = root.join(platform.binary_name());
            stage(platform, &stage_dir, &source).unwrap();

            let executable_dir = stage_dir.join(directory);
            assert_eq!(
                fs::read(executable_dir.join(LOCAL_REPOSITORY_FILE)).unwrap(),
                b"{\"plugins\":[]}"
            );
            assert_eq!(
                fs::read(executable_dir.join("plugins/org.example.demo/plugin.wasm")).unwrap(),
                b"\0asm"
            );
            assert_eq!(
                fs::read(executable_dir.join("plugins/org.example.demo/assets/example.txt"),)
                    .unwrap(),
                b"asset"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_missing_application_plugin_artifacts() {
        let root = temporary_root("missing-plugins");
        let error = stage(Platform::Linux, &root.join("stage"), &root).unwrap_err();
        assert!(matches!(error, XtaskError::MissingArtifact(_)));

        fs::create_dir(root.join("plugins")).unwrap();
        let error = stage(Platform::Linux, &root.join("stage"), &root).unwrap_err();
        assert!(matches!(error, XtaskError::MissingArtifact(_)));
        fs::remove_dir_all(root).unwrap();
    }

    fn temporary_root(name: &str) -> PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{name}-{}-{timestamp}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
