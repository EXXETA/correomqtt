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
        crate_path: "unused-in-release-test",
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
    let repository = std::fs::read_to_string(&repository_path).unwrap();
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

    let file = std::fs::File::open(&archive_path).unwrap();
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

    let download_dir = root.join("release-download");
    copy_file(
        &repository_path,
        &download_dir.join(RELEASE_PLUGIN_REPOSITORY_FILE),
    )
    .unwrap();
    copy_file(&archive_path, &download_dir.join(&archive_file_name)).unwrap();
    let profile_dir = root.join("clean-profile");
    smoke_release_artifacts(
        &download_dir,
        &profile_dir,
        "https://example.invalid/releases/latest/",
    )
    .unwrap();

    let installed_package = correo_plugins::PluginPackage::load(
        profile_dir
            .join("plugins")
            .join(safe_plugin_id_component(&entry.manifest.id).unwrap()),
    )
    .unwrap();
    assert_eq!(installed_package.manifest().id, entry.manifest.id);
    assert_eq!(installed_package.read_wasm().unwrap(), b"wasm");
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
fn plugin_release_smoke_help_does_not_require_directories() {
    let config = PluginReleaseSmokeConfig::from_args(vec!["--help".to_owned()]).unwrap();
    assert!(config.show_help);
}

#[test]
fn tag_release_workflow_publishes_generated_repository_assets() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let workflow = fs::read_to_string(workspace_root.join(".github/workflows/rust-build.yml"))
        .unwrap()
        .replace("\r\n", "\n");
    let plugin_release = workflow.split("  plugin-release:\n").nth(1).unwrap();

    assert_eq!(workflow.matches("contents: write").count(), 1);
    assert!(plugin_release.contains(
        "needs: [check, package, keyring-integration, mqtt-integration]"
    ));
    assert!(plugin_release.contains("permissions:\n      contents: write\n      actions: read"));
    assert!(plugin_release.contains(
        "RELEASE_ASSET_BASE_URL: ${{ github.server_url }}/${{ github.repository }}/releases/download/${{ github.ref_name }}"
    ));
    assert!(plugin_release.contains("gh release view \"$GITHUB_REF_NAME\""));
    assert!(plugin_release
        .contains("gh release view \"$GITHUB_REF_NAME\" --json isDraft --jq '.isDraft'"));
    assert!(plugin_release.contains("refusing to replace assets on published release"));
    assert!(plugin_release.contains(
        "gh release create \"$GITHUB_REF_NAME\" --verify-tag --title \"$GITHUB_REF_NAME\" --draft --generate-notes"
    ));
    let publish = "gh release edit \"$GITHUB_REF_NAME\" --title \"$GITHUB_REF_NAME\" --draft=false";
    let upload = "gh release upload \"$GITHUB_REF_NAME\" \\\n            dist/plugins/*.zip \\\n            dist/plugins/default-repo.json \\\n            dist/packages/*/*.{zip,zip.sha256,dmg,dmg.sha256,deb,deb.sha256,rpm,rpm.sha256,msi,msi.sha256}";
    let smoke = "cargo xtask plugin-release-smoke --download-dir \"$release_dir\" --profile-dir \"$profile_dir\" --asset-base-url \"$RELEASE_ASSET_BASE_URL\"";
    assert!(plugin_release.contains(publish));
    assert!(plugin_release.contains(upload));
    assert!(plugin_release.contains(
        "cargo xtask plugin-release --out-dir dist/plugins --asset-base-url \"$RELEASE_ASSET_BASE_URL\""
    ));
    assert!(plugin_release.contains("gh run download \"$GITHUB_RUN_ID\" --dir dist/packages"));
    assert!(plugin_release.contains("gh release delete-asset \"$GITHUB_REF_NAME\" \"$asset\" --yes"));
    assert!(plugin_release.contains(
        "gh release download \"$GITHUB_REF_NAME\" --dir \"$release_dir\" --pattern '*.zip' --pattern default-repo.json"
    ));
    assert!(plugin_release.contains(smoke));
    assert!(plugin_release.find(upload).unwrap() < plugin_release.find(smoke).unwrap());
    assert!(plugin_release.find(smoke).unwrap() < plugin_release.find(publish).unwrap());
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
