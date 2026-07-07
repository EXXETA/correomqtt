use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use correo_plugins::{
    PluginInstallSource, PluginManifest, PluginRepositoryDefinition, PluginRepositoryEntry,
};

use crate::{cargo_dynamic, package, XtaskError};

pub(crate) const LOCAL_PLUGIN_REPOSITORY_FILE: &str = "local-repo.json";
pub(crate) const RELEASE_PLUGIN_REPOSITORY_FILE: &str = "default-repo.json";
const WASM_TARGET: &str = "wasm32-unknown-unknown";
const REPOSITORY_ID: &str = "local-packaged-plugins";
const REPOSITORY_NAME: &str = "Local Packaged Plugins";
const RELEASE_REPOSITORY_ID: &str = "default-plugin-repository";
const RELEASE_REPOSITORY_NAME: &str = "Default CorreoMQTT Plugins";
// GitHub's stable "newest release" asset form is /releases/latest/download/<asset>.
// (/releases/download/latest/ would instead resolve a literal tag named "latest".)
const DEFAULT_RELEASE_ASSET_BASE_URL: &str =
    "https://github.com/EXXETA/correomqtt/releases/latest/download";

#[derive(Debug, Clone, Copy)]
pub(crate) struct PluginBuildSpec {
    pub package: &'static str,
    pub manifest_path: &'static str,
    pub wasm_stem: &'static str,
}

pub(crate) const PLUGIN_SPECS: &[PluginBuildSpec] = &[
    PluginBuildSpec {
        package: "correo-plugin-base64",
        manifest_path: "crates/correo-plugin-base64/plugin.toml",
        wasm_stem: "correo_plugin_base64",
    },
    PluginBuildSpec {
        package: "correo-plugin-zip-manipulator",
        manifest_path: "crates/correo-plugin-zip-manipulator/plugin.toml",
        wasm_stem: "correo_plugin_zip_manipulator",
    },
    PluginBuildSpec {
        package: "correo-plugins-advanced-validator",
        manifest_path: "plugins/correo-plugins-advanced-validator/plugin.toml",
        wasm_stem: "correo_plugins_advanced_validator",
    },
    PluginBuildSpec {
        package: "correo-plugins-contains-string-validator",
        manifest_path: "plugins/correo-plugins-contains-string-validator/plugin.toml",
        wasm_stem: "correo_plugins_contains_string_validator",
    },
    PluginBuildSpec {
        package: "correo-plugins-json-format",
        manifest_path: "plugins/correo-plugins-json-format/plugin.toml",
        wasm_stem: "correo_plugins_json_format",
    },
    PluginBuildSpec {
        package: "correo-plugins-systopic",
        manifest_path: "plugins/correo-plugins-systopic/plugin.toml",
        wasm_stem: "correo_plugins_systopic",
    },
    PluginBuildSpec {
        package: "correo-plugins-xml-xsd-validator",
        manifest_path: "plugins/correo-plugins-xml-xsd-validator/plugin.toml",
        wasm_stem: "correo_plugins_xml_xsd_validator",
    },
    PluginBuildSpec {
        package: "correo-plugin-xml-format",
        manifest_path: "plugins/xml-format/plugin.toml",
        wasm_stem: "correo_plugin_xml_format",
    },
    PluginBuildSpec {
        package: "correo-plugin-save-manipulator",
        manifest_path: "plugins/save-manipulator/plugin.toml",
        wasm_stem: "correo_plugin_save_manipulator",
    },
];

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
    ensure_wasm_target_available_or_install()?;

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

fn ensure_wasm_target_available_or_install() -> Result<(), XtaskError> {
    if wasm_target_available()? {
        return Ok(());
    }

    install_wasm_target()?;
    if wasm_target_available()? {
        Ok(())
    } else {
        missing_wasm_target()
    }
}

fn wasm_target_available() -> Result<bool, XtaskError> {
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = match Command::new(rustc)
        .args(["--print", "target-libdir", "--target", WASM_TARGET])
        .output()
    {
        Ok(output) => output,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };

    if !output.status.success() {
        return Ok(false);
    }

    let lib_dir = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    let mut entries = match fs::read_dir(&lib_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    Ok(entries.any(|entry| {
        entry
            .ok()
            .and_then(|entry| entry.file_name().into_string().ok())
            .is_some_and(|name| name.starts_with("libcore-") && name.ends_with(".rlib"))
    }))
}

fn install_wasm_target() -> Result<(), XtaskError> {
    println!("xtask: installing missing Rust target `{WASM_TARGET}`");
    let status = match Command::new("rustup")
        .args(["target", "add", WASM_TARGET])
        .status()
    {
        Ok(status) => status,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return missing_wasm_target(),
        Err(error) => return Err(error.into()),
    };

    if status.success() {
        Ok(())
    } else {
        missing_wasm_target()
    }
}

fn missing_wasm_target() -> Result<(), XtaskError> {
    Err(XtaskError::MissingRustTarget {
        target: WASM_TARGET.to_owned(),
        install_command: format!("rustup target add {WASM_TARGET}"),
    })
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
        let package_component = safe_plugin_id_component(&manifest.id)?;
        let relative_package_path = PathBuf::from("plugins").join(package_component);
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
        let package_dir = stage_root.join(safe_plugin_id_component(&manifest.id)?);
        stage_plugin_package(spec, &package_dir)?;

        let archive_file_name = plugin_archive_file_name(&manifest)?;
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
    let temporary = temporary_path(path);
    fs::write(&temporary, content)?;
    replace_file(&temporary, path)?;
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
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

fn plugin_archive_file_name(manifest: &PluginManifest) -> Result<String, XtaskError> {
    Ok(format!(
        "{}-{}.zip",
        safe_plugin_id_component(&manifest.id)?,
        manifest.version
    ))
}

fn release_asset_url(base_url: &str, file_name: &str) -> String {
    format!("{}/{}", base_url.trim_end_matches('/'), file_name)
}

fn safe_plugin_id_component(value: &str) -> Result<&str, XtaskError> {
    let is_safe = !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .chars()
            .all(|character| matches!(character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_'));
    if is_safe {
        Ok(value)
    } else {
        Err(XtaskError::InvalidArguments(format!(
            "plugin id `{value}` is not a safe single path component"
        )))
    }
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
