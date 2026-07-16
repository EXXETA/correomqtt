use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[path = "../../xtask/src/plugin_specs.rs"]
mod plugin_specs;

use plugin_specs::{PluginBuildSpec, PLUGIN_SPECS};

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const LOCAL_REPOSITORY_FILE: &str = "local-repo.json";
const REPOSITORY_ID: &str = "local-packaged-plugins";
const REPOSITORY_NAME: &str = "Local Packaged Plugins";
const PLUGIN_REPOSITORY_FORMAT_VERSION: u16 = 1;

fn main() {
    let workspace_root = workspace_root().expect("resolve workspace root");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
    println!("cargo:rerun-if-env-changed=CARGO");
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("Cargo.lock").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("xtask/src/plugin_specs.rs").display()
    );
    for spec in PLUGIN_SPECS {
        println!(
            "cargo:rerun-if-changed={}",
            workspace_root.join(spec.crate_path).display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            workspace_root.join(spec.manifest_path).display()
        );
    }

    if let Err(error) = build_and_stage_plugins() {
        panic!("failed to prepare bundled plugins for correo-app: {error}");
    }
}

fn build_and_stage_plugins() -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let target_root = target_root(&workspace_root);
    let plugin_target_root = target_root.join("correo-app-plugins");
    let profile = env::var("PROFILE").map_err(|error| error.to_string())?;

    build_wasm_plugins(&plugin_target_root, &profile)?;

    for executable_dir in executable_dirs(&target_root, &profile)? {
        stage_local_plugins(
            &workspace_root,
            &plugin_target_root,
            &profile,
            &executable_dir,
        )?;
    }

    Ok(())
}

fn build_wasm_plugins(plugin_target_root: &Path, profile: &str) -> Result<(), String> {
    let mut command = Command::new(env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")));
    command.env("CARGO_TARGET_DIR", plugin_target_root);
    command.args(["build", "--target", WASM_TARGET]);
    if profile == "release" {
        command.arg("--release");
    }
    for spec in PLUGIN_SPECS {
        command.args(["-p", spec.package]);
    }

    let status = command.status().map_err(|error| error.to_string())?;
    ensure_success(status, "cargo build --target wasm32-unknown-unknown")
}

fn stage_local_plugins(
    workspace_root: &Path,
    target_root: &Path,
    profile: &str,
    executable_dir: &Path,
) -> Result<(), String> {
    let plugin_root = executable_dir.join("plugins");
    if plugin_root.exists() {
        fs::remove_dir_all(&plugin_root).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&plugin_root).map_err(|error| error.to_string())?;

    let mut entries = Vec::new();
    for spec in PLUGIN_SPECS {
        let manifest_path = workspace_root.join(spec.manifest_path);
        let manifest = read_manifest(&manifest_path)?;
        let plugin_id = manifest_id(&manifest)?;
        let relative_package_path = PathBuf::from("plugins").join(plugin_id);
        let package_dir = executable_dir.join(&relative_package_path);
        stage_plugin_package(workspace_root, target_root, profile, spec, &package_dir)?;
        entries.push(serde_json::json!({
            "manifest": manifest,
            "install_source": {
                "kind": "local_package",
                "path": slash_path(&relative_package_path),
            }
        }));
    }

    entries.sort_by_key(manifest_entry_id);
    let repository = serde_json::json!({
        "repository_format_version": PLUGIN_REPOSITORY_FORMAT_VERSION,
        "id": REPOSITORY_ID,
        "name": REPOSITORY_NAME,
        "plugins": entries,
    });

    let mut json = serde_json::to_vec_pretty(&repository).map_err(|error| error.to_string())?;
    json.push(b'\n');
    fs::write(executable_dir.join(LOCAL_REPOSITORY_FILE), json).map_err(|error| error.to_string())
}

fn stage_plugin_package(
    workspace_root: &Path,
    target_root: &Path,
    profile: &str,
    spec: &PluginBuildSpec,
    package_dir: &Path,
) -> Result<(), String> {
    if package_dir.exists() {
        fs::remove_dir_all(package_dir).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(package_dir).map_err(|error| error.to_string())?;

    let manifest_path = workspace_root.join(spec.manifest_path);
    copy_file(&manifest_path, &package_dir.join("plugin.toml"))?;
    copy_file(
        &wasm_artifact_path(target_root, profile, spec.wasm_stem),
        &package_dir.join("plugin.wasm"),
    )?;

    let assets_dir = manifest_path.parent().map(|parent| parent.join("assets"));
    if let Some(assets_dir) = assets_dir.filter(|path| path.exists()) {
        copy_dir_recursive(&assets_dir, &package_dir.join("assets"))?;
    }
    Ok(())
}

fn read_manifest(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let value = toml::from_str::<toml::Value>(&text).map_err(|error| error.to_string())?;
    serde_json::to_value(value).map_err(|error| error.to_string())
}

fn manifest_id(manifest: &serde_json::Value) -> Result<&str, String> {
    manifest
        .get("id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.trim().is_empty() && !id.contains('/') && !id.contains('\\'))
        .ok_or_else(|| format!("plugin manifest has invalid id: {manifest}"))
}

fn manifest_entry_id(entry: &serde_json::Value) -> String {
    entry
        .get("manifest")
        .and_then(|manifest| manifest.get("id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn executable_dirs(target_root: &Path, profile: &str) -> Result<Vec<PathBuf>, String> {
    let target = env::var("TARGET").map_err(|error| error.to_string())?;
    let host = env::var("HOST").map_err(|error| error.to_string())?;
    let default_dir = target_root.join(profile);
    let target_dir = target_root.join(&target).join(profile);

    if target == host {
        Ok(vec![default_dir, target_dir])
    } else {
        Ok(vec![target_dir])
    }
}

fn wasm_artifact_path(target_root: &Path, profile: &str, wasm_stem: &str) -> PathBuf {
    target_root
        .join(WASM_TARGET)
        .join(profile)
        .join(format!("{wasm_stem}.wasm"))
}

fn target_root(workspace_root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"))
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing CARGO_MANIFEST_DIR")?);
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "could not resolve workspace root from {}",
                manifest_dir.display()
            )
        })
}

fn ensure_success(status: ExitStatus, command: &str) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{command} exited with {status}; ensure the {WASM_TARGET} target is installed"
        ))
    }
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::copy(source, destination).map_err(|error| error.to_string())?;
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let mut entries = fs::read_dir(source)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
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

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
