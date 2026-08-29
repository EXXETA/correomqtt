use std::fs;
use std::path::{Path, PathBuf};

use correo_plugins::{
    PluginInstallSource, PluginManifest, PluginRepositoryDefinition, PluginRepositoryEntry,
};

use crate::{
    cargo_dynamic, package,
    plugin_specs::{PluginBuildSpec, PLUGIN_SPECS},
    XtaskError,
};

pub(crate) const LOCAL_PLUGIN_REPOSITORY_FILE: &str = "local-repo.json";
pub(crate) const RELEASE_PLUGIN_REPOSITORY_FILE: &str = "default-repo.json";
const WASM_TARGET: &str = "wasm32-unknown-unknown";
const REPOSITORY_ID: &str = "local-packaged-plugins";
const REPOSITORY_NAME: &str = "Local Packaged Plugins";
const RELEASE_REPOSITORY_ID: &str = "default-plugin-repository";
const RELEASE_REPOSITORY_NAME: &str = "Default CorreoMQTT Plugins";
const DEFAULT_RELEASE_ASSET_BASE_URL: &str =
    "https://github.com/EXXETA/correomqtt/releases/download/latest";

pub(crate) fn run(args: Vec<String>) -> Result<(), XtaskError> {
    let config = PluginRepositoryConfig::from_args(args)?;
    if config.show_help {
        print_help();
        return Ok(());
    }

    build_wasm_plugins()?;
    stage_local_plugins(&config.out_dir)?;
    println!(
        "plugin-repository: {}",
        config.out_dir.join(LOCAL_PLUGIN_REPOSITORY_FILE).display()
    );
    Ok(())
}

pub(crate) fn release(args: Vec<String>) -> Result<(), XtaskError> {
    let config = PluginReleaseConfig::from_args(args)?;
    if config.show_help {
        print_release_help();
        return Ok(());
    }

    if config.build {
        build_wasm_plugins()?;
    }
    write_release_artifacts(&config.out_dir, &config.asset_base_url)?;
    println!(
        "plugin-release: {}",
        config
            .out_dir
            .join(RELEASE_PLUGIN_REPOSITORY_FILE)
            .display()
    );
    Ok(())
}

pub(crate) fn build_wasm_plugins() -> Result<(), XtaskError> {
    let mut args = vec![
        "build".to_owned(),
        "--release".to_owned(),
        "--target".to_owned(),
        WASM_TARGET.to_owned(),
    ];
    for spec in PLUGIN_SPECS {
        args.push("-p".to_owned());
        args.push(spec.package.to_owned());
    }
    cargo_dynamic(&args)
}

pub(crate) fn stage_local_plugins(executable_dir: &Path) -> Result<(), XtaskError> {
    let plugin_root = executable_dir.join("plugins");
    if plugin_root.exists() {
        fs::remove_dir_all(&plugin_root)?;
    }
    fs::create_dir_all(&plugin_root)?;

    let mut entries = Vec::new();
    for spec in PLUGIN_SPECS {
        let manifest = read_manifest(spec.manifest_path)?;
        let relative_package_path = PathBuf::from("plugins").join(&manifest.id);
        let package_dir = executable_dir.join(&relative_package_path);
        stage_plugin_package(spec, &package_dir)?;
        entries.push(PluginRepositoryEntry::local_package(
            manifest,
            relative_package_path,
        )?);
    }

    entries.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    let repository = PluginRepositoryDefinition {
        repository_format_version: correo_plugins::PLUGIN_REPOSITORY_FORMAT_VERSION,
        id: REPOSITORY_ID.to_owned(),
        name: REPOSITORY_NAME.to_owned(),
        plugins: entries,
    };
    repository.validate()?;
    write_repository(
        &executable_dir.join(LOCAL_PLUGIN_REPOSITORY_FILE),
        &repository,
    )
}

fn write_release_artifacts(out_dir: &Path, asset_base_url: &str) -> Result<(), XtaskError> {
    write_release_artifacts_for_specs(out_dir, asset_base_url, PLUGIN_SPECS)
}

fn write_release_artifacts_for_specs(
    out_dir: &Path,
    asset_base_url: &str,
    specs: &[PluginBuildSpec],
) -> Result<(), XtaskError> {
    fs::create_dir_all(out_dir)?;
    let stage_dir = out_dir.join("stage");
    let stage_root = stage_dir.join("plugins");
    if stage_dir.exists() {
        fs::remove_dir_all(&stage_dir)?;
    }
    fs::create_dir_all(&stage_root)?;

    let mut entries = Vec::new();
    for spec in specs {
        let manifest = read_manifest(spec.manifest_path)?;
        let package_dir = stage_root.join(safe_file_component(&manifest.id));
        stage_plugin_package(spec, &package_dir)?;

        let archive_file_name = plugin_archive_file_name(&manifest);
        let archive_path = out_dir.join(&archive_file_name);
        if archive_path.exists() {
            fs::remove_file(&archive_path)?;
        }
        package::zip_dir_contents(&package_dir, &archive_path)?;
        let sha256 = package::checksums::sha256_file(&archive_path)?;
        println!("plugin-archive: {}", archive_path.display());
        println!("sha256:        {sha256}");

        entries.push(PluginRepositoryEntry {
            manifest,
            install_source: PluginInstallSource::Archive {
                url: release_asset_url(asset_base_url, &archive_file_name),
                sha256,
            },
        });
    }

    entries.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    let repository = PluginRepositoryDefinition {
        repository_format_version: correo_plugins::PLUGIN_REPOSITORY_FORMAT_VERSION,
        id: RELEASE_REPOSITORY_ID.to_owned(),
        name: RELEASE_REPOSITORY_NAME.to_owned(),
        plugins: entries,
    };
    repository.validate()?;
    write_repository(&out_dir.join(RELEASE_PLUGIN_REPOSITORY_FILE), &repository)?;
    fs::remove_dir_all(stage_dir)?;
    Ok(())
}

fn stage_plugin_package(spec: &PluginBuildSpec, package_dir: &Path) -> Result<(), XtaskError> {
    if package_dir.exists() {
        fs::remove_dir_all(package_dir)?;
    }
    fs::create_dir_all(package_dir)?;
    let manifest_path = Path::new(spec.manifest_path);
    copy_file(manifest_path, &package_dir.join("plugin.toml"))?;
    copy_file(
        &wasm_artifact_path(spec.wasm_stem),
        &package_dir.join("plugin.wasm"),
    )?;

    let assets_dir = manifest_path.parent().map(|parent| parent.join("assets"));
    if let Some(assets_dir) = assets_dir.filter(|path| path.exists()) {
        copy_dir_recursive(&assets_dir, &package_dir.join("assets"))?;
    }
    Ok(())
}

fn read_manifest(path: &str) -> Result<PluginManifest, XtaskError> {
    let text = fs::read_to_string(path)?;
    Ok(
        PluginManifest::from_toml_str(&text)
            .map_err(correo_plugins::PluginRepositoryError::from)?,
    )
}

fn wasm_artifact_path(wasm_stem: &str) -> PathBuf {
    target_dir()
        .join(WASM_TARGET)
        .join("release")
        .join(format!("{wasm_stem}.wasm"))
}

fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"))
}

fn write_repository(
    path: &Path,
    repository: &PluginRepositoryDefinition,
) -> Result<(), XtaskError> {
    let mut json = serde_json::to_vec_pretty(repository)?;
    json.push(b'\n');
    write_file(path, &json)
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), XtaskError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn write_file(path: &Path, content: &[u8]) -> Result<(), XtaskError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), XtaskError> {
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

fn plugin_archive_file_name(manifest: &PluginManifest) -> String {
    format!(
        "{}-{}.zip",
        safe_file_component(&manifest.id),
        manifest.version
    )
}

fn release_asset_url(base_url: &str, file_name: &str) -> String {
    format!("{}/{}", base_url.trim_end_matches('/'), file_name)
}

fn safe_file_component(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}

fn print_help() {
    println!("Usage: cargo xtask plugin-repository [--out-dir <dir>]");
    println!();
    println!("Builds and stages local plugin packages plus local-repo.json.");
    println!("Default --out-dir is target/debug, next to the dev executable.");
}

fn print_release_help() {
    println!(
        "Usage: cargo xtask plugin-release [--out-dir <dir>] [--asset-base-url <url>] [--no-build]"
    );
    println!();
    println!("Builds plugin WASM archives plus default-repo.json for GitHub release assets.");
    println!("Default --out-dir is dist/plugins.");
}

#[derive(Debug)]
struct PluginRepositoryConfig {
    out_dir: PathBuf,
    show_help: bool,
}

#[derive(Debug)]
struct PluginReleaseConfig {
    out_dir: PathBuf,
    asset_base_url: String,
    build: bool,
    show_help: bool,
}

impl PluginReleaseConfig {
    fn from_args(args: Vec<String>) -> Result<Self, XtaskError> {
        let mut out_dir = PathBuf::from("dist/plugins");
        let mut asset_base_url = DEFAULT_RELEASE_ASSET_BASE_URL.to_owned();
        let mut build = true;
        let mut show_help = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--out-dir" => {
                    let value = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--out-dir requires a value".to_owned())
                    })?;
                    out_dir = PathBuf::from(value);
                }
                "--asset-base-url" => {
                    asset_base_url = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--asset-base-url requires a value".to_owned())
                    })?;
                    if asset_base_url.trim().is_empty() {
                        return Err(XtaskError::InvalidArguments(
                            "--asset-base-url cannot be empty".to_owned(),
                        ));
                    }
                }
                "--no-build" => build = false,
                "-h" | "--help" => show_help = true,
                unknown => {
                    return Err(XtaskError::InvalidArguments(format!(
                        "unknown plugin-release option: {unknown}"
                    )));
                }
            }
        }

        Ok(Self {
            out_dir,
            asset_base_url,
            build,
            show_help,
        })
    }
}

#[cfg(test)]
mod tests;

impl PluginRepositoryConfig {
    fn from_args(args: Vec<String>) -> Result<Self, XtaskError> {
        let mut out_dir = target_dir().join("debug");
        let mut show_help = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--out-dir" => {
                    let value = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--out-dir requires a value".to_owned())
                    })?;
                    out_dir = PathBuf::from(value);
                }
                "-h" | "--help" => show_help = true,
                unknown => {
                    return Err(XtaskError::InvalidArguments(format!(
                        "unknown plugin-repository option: {unknown}"
                    )));
                }
            }
        }

        Ok(Self { out_dir, show_help })
    }
}
