use correo_plugins::{
    PluginInstallSource, PluginManifest, PluginRepositoryDefinition, PluginRepositoryEntry,
    PLUGIN_REPOSITORY_FORMAT_VERSION,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/repository.json");

#[test]
fn repository_fixture_matches_generated_bundled_manifest_catalog() {
    let repository = expected_local_repository();
    repository.validate().unwrap();

    let mut generated = serde_json::to_string_pretty(&repository).unwrap();
    generated.push('\n');

    // Golden file: regenerate with REGEN_FIXTURE=1 after intentional
    // manifest changes, then review the diff.
    if std::env::var_os("REGEN_FIXTURE").is_some() {
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/repository.json"
            ),
            &generated,
        )
        .unwrap();
    }

    assert_eq!(generated, FIXTURE);
    assert_eq!(
        repository.repository_format_version,
        PLUGIN_REPOSITORY_FORMAT_VERSION
    );
}

fn expected_local_repository() -> PluginRepositoryDefinition {
    let mut repository = PluginRepositoryDefinition::from_bundled_plugins(
        "local-bundled-rust",
        "Bundled Rust Plugins",
    );
    let manifest = PluginManifest::from_toml_str(include_str!(
        "../../../plugins/save-manipulator/plugin.toml"
    ))
    .unwrap();
    repository
        .plugins
        .push(PluginRepositoryEntry::local_package(manifest, "plugins/save-manipulator").unwrap());
    repository
        .plugins
        .sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    repository
}

#[test]
fn repository_fixture_is_path_safe_and_contains_no_legacy_pf4j_sources() {
    let repository = serde_json::from_str::<PluginRepositoryDefinition>(FIXTURE).unwrap();

    repository.validate().unwrap();
    assert!(!repository.plugins.is_empty());

    let mut found_save_manipulator = false;
    for plugin in &repository.plugins {
        match &plugin.install_source {
            PluginInstallSource::Bundled { plugin_id } => {
                assert_eq!(plugin_id, &plugin.manifest.id);
            }
            PluginInstallSource::LocalPackage { path } => {
                assert_eq!(
                    plugin.manifest.id,
                    "org.correomqtt.plugins.save-manipulator"
                );
                assert_eq!(path, "plugins/save-manipulator");
                found_save_manipulator = true;
            }
            PluginInstallSource::Archive { .. } => {
                panic!("bundled fixture should not contain remote archive sources");
            }
        }
    }
    assert!(found_save_manipulator);

    let fixture_json = serde_json::from_str::<Value>(FIXTURE).unwrap();
    let fixture_text = fixture_json.to_string();
    assert!(!fixture_text.contains(".jar"));
    assert!(!fixture_text.contains("plugins/jars"));
}

#[test]
fn repository_rejects_absolute_or_parent_package_paths() {
    let manifest = PluginRepositoryDefinition::from_bundled_plugins("local", "Local")
        .plugins
        .into_iter()
        .next()
        .unwrap()
        .manifest;

    assert!(PluginRepositoryEntry::local_package(manifest.clone(), "/tmp/plugin").is_err());
    assert!(PluginRepositoryEntry::local_package(manifest, "../plugin").is_err());
}
