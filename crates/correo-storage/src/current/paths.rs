use std::path::PathBuf;

/// Current app data root (`CORREOMQTT_CONFIG_DIR` override, then platform dirs).
/// An empty override is treated as unset.
pub fn config_root() -> PathBuf {
    if let Some(path) = std::env::var_os("CORREOMQTT_CONFIG_DIR") {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    directories::ProjectDirs::from("org", "CorreoMQTT", "CorreoMQTT")
        .map(|project_dirs| project_dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".correomqtt"))
}

pub fn current_roots() -> Vec<PathBuf> {
    vec![config_root()]
}

#[derive(Clone, Copy)]
enum LegacyPlatform {
    Windows,
    MacOs,
    Unix,
}

pub fn legacy_roots() -> Vec<PathBuf> {
    let Some(base_dirs) = directories::BaseDirs::new() else {
        return Vec::new();
    };
    vec![legacy_root_from(
        host_legacy_platform(),
        base_dirs.config_dir(),
        base_dirs.home_dir(),
    )]
}

const fn host_legacy_platform() -> LegacyPlatform {
    if cfg!(target_os = "windows") {
        LegacyPlatform::Windows
    } else if cfg!(target_os = "macos") {
        LegacyPlatform::MacOs
    } else {
        LegacyPlatform::Unix
    }
}

fn legacy_root_from(
    platform: LegacyPlatform,
    config_dir: &std::path::Path,
    home: &std::path::Path,
) -> PathBuf {
    match platform {
        LegacyPlatform::Windows => config_dir.join("CorreoMqtt"),
        LegacyPlatform::MacOs => home
            .join("Library")
            .join("Application Support")
            .join("CorreoMqtt"),
        LegacyPlatform::Unix => home.join(".correomqtt"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_roots_select_only_the_host_platform_location() {
        let config_dir = PathBuf::from("C:/Users/test/AppData/Roaming");
        let home = PathBuf::from("/Users/test");

        assert_eq!(
            legacy_root_from(LegacyPlatform::Windows, &config_dir, &home),
            PathBuf::from("C:/Users/test/AppData/Roaming/CorreoMqtt")
        );
        assert_eq!(
            legacy_root_from(LegacyPlatform::MacOs, &config_dir, &home),
            PathBuf::from("/Users/test/Library/Application Support/CorreoMqtt")
        );
        assert_eq!(
            legacy_root_from(LegacyPlatform::Unix, &config_dir, &home),
            PathBuf::from("/Users/test/.correomqtt")
        );
    }
}
