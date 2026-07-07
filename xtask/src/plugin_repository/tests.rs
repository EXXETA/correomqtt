use super::*;

#[test]
fn release_artifacts_point_repository_entries_at_archives() {
    let root = temp_root("plugin-release");
    let source = root.join("source");
    let out_dir = root.join("out");
    let wasm_stem = unique_name("release_test_plugin");
    let wasm_path = target_dir()
        .join(WASM_TARGET)
        .join("release")
        .join(format!("{wasm_stem}.wasm"));

    write_file(&source.join("plugin.toml"), manifest_toml().as_bytes()).unwrap();
    write_file(&source.join("assets/readme.txt"), b"asset").unwrap();
    write_file(&wasm_path, b"wasm").unwrap();

    let manifest_path = leak_path(source.join("plugin.toml"));
    let wasm_stem = Box::leak(wasm_stem.into_boxed_str());
    let spec = PluginBuildSpec {
        package: "release-test-plugin",
        manifest_path,
        wasm_stem,
    };

    write_release_artifacts_for_specs(
        &out_dir,
        "https://example.invalid/releases/latest/",
        &[spec],
    )
    .unwrap();

    let repository_path = out_dir.join(RELEASE_PLUGIN_REPOSITORY_FILE);
    let repository = std::fs::read_to_string(repository_path).unwrap();
    let repository: PluginRepositoryDefinition = serde_json::from_str(&repository).unwrap();
    repository.validate().unwrap();

    assert_eq!(repository.plugins.len(), 1);
    let entry = &repository.plugins[0];
    let archive_file_name = plugin_archive_file_name(&entry.manifest).unwrap();
    let archive_path = out_dir.join(&archive_file_name);
    let expected_sha256 = package::checksums::sha256_file(&archive_path).unwrap();
    assert_eq!(
        entry.install_source,
        PluginInstallSource::Archive {
            url: format!("https://example.invalid/releases/latest/{archive_file_name}"),
            sha256: expected_sha256,
        }
    );

    let file = std::fs::File::open(archive_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut names = Vec::new();
    for index in 0..archive.len() {
        names.push(archive.by_index(index).unwrap().name().to_owned());
    }
    assert_eq!(
        names,
        vec![
            "assets/".to_owned(),
            "assets/readme.txt".to_owned(),
            "plugin.toml".to_owned(),
            "plugin.wasm".to_owned(),
        ]
    );
    assert!(!out_dir.join("stage").exists());

    std::fs::remove_file(wasm_path).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn plugin_release_config_defaults_to_latest_github_assets() {
    let config = PluginReleaseConfig::from_args(Vec::new()).unwrap();
    assert_eq!(config.out_dir, PathBuf::from("dist/plugins"));
    assert_eq!(config.asset_base_url, DEFAULT_RELEASE_ASSET_BASE_URL);
    // GitHub's stable latest-asset form is /releases/latest/download/, not the
    // literal-tag form /releases/download/latest/.
    assert!(config.asset_base_url.ends_with("/releases/latest/download"));
    assert!(config.build);
    assert!(!config.show_help);
}

#[test]
fn plugin_ids_must_be_safe_single_path_components() {
    assert!(safe_plugin_id_component("org.correomqtt.plugins.release-test").is_ok());
    assert!(safe_plugin_id_component("../outside").is_err());
    assert!(safe_plugin_id_component("nested/plugin").is_err());
    assert!(safe_plugin_id_component("C:\\outside").is_err());
    assert!(safe_plugin_id_component("").is_err());
}

fn manifest_toml() -> String {
    "manifest_version = 1\n\
     id = \"org.correomqtt.plugins.release-test\"\n\
     name = \"Release Test\"\n\
     version = \"1.2.3\"\n\
     description = \"Release artifact test plugin\"\n\
     provider = \"CorreoMQTT\"\n\
     license = \"GPL-3.0-or-later\"\n\
     compatible_correomqtt = \">=0.1.0\"\n\
     \n\
     [capabilities]\n\
     hooks = [\"message_validator\"]\n\
     \n\
     [[entrypoints]]\n\
     hook = \"message_validator\"\n\
     export = \"validate\"\n"
        .to_owned()
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(unique_name(name));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn unique_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}-{}-{timestamp}", std::process::id())
}

fn leak_path(path: PathBuf) -> &'static str {
    Box::leak(path.to_string_lossy().into_owned().into_boxed_str())
}
