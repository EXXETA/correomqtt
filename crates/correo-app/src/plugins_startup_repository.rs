use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

use correo_core::{
    marketplace_rows_from_repository_json, ConsumerSelection, DeliveryGuarantee, DeliverySemantics,
    DetailBytesOutput, FormattedMessageDetail, MessageDetailFormat, MessageEnvelope,
    MessageTransform, NamespacedName, PayloadSyntaxKind, PayloadSyntaxSpan,
    PluginConnectionActionRequest, PluginConnectionActionResponse, PluginHookCall, PluginHookError,
    PluginHookExecution, PluginHookExecutor, PluginHookInput, PluginHookKind, PluginHookOutput,
    PluginHostAction, PluginHostActionResponse, PluginInstaller, PluginMarketplaceRow,
    PluginMarketplaceSource, PluginMessage, PluginMetricListNode, PluginMetricRow,
    PluginOpenWindow, PluginSavePayload, PluginTransportMessage, PluginUiNode, PluginValidation,
    PluginWindowCloseRequest, PluginWindowMessage, PluginWindowRenderRequest,
    PluginWindowRenderResponse, QosLevel, TransportCapabilities, TransportCapability,
    TransportMessageTransform,
};
use correo_plugins::{
    bundled_plugin_by_id, ConsumerSelectionDto, DeliveryGuaranteeDto, DetailByteTransformRequest,
    DetailFormatDto, DetailFormatterRequest, HookContextDto, HookInvocation, HookOutput,
    HostActionDto, HostSurface, IncomingMessageTransformRequest,
    IncomingTransportMessageTransformRequest, MessageDto, MessageTransformOutcomeDto,
    MessageValidatorRequest, NamespacedNameDto, OutgoingMessageTransformRequest,
    OutgoingTransportMessageTransformRequest, PluginAbi, PluginCancellationToken, PluginManifest,
    PluginPackage, PluginRegistry, QosDto, TransportHookInputDto, TransportMessageDto,
    TransportMessageTransformOutcomeDto, TransportMessageValidatorRequest, ValidationResultDto,
};
use correo_storage::current::{AppConfig, Settings};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const BUNDLED_JSON: &str = include_str!("bundled.json");
const LOCAL_REPOSITORY_FILE: &str = "local-repo.json";
// GitHub's stable newest-release asset form is /releases/latest/download/,
// matching what `cargo xtask plugin-release` publishes.
pub const DEFAULT_REPOSITORY_URL: &str =
    "https://github.com/EXXETA/correomqtt/releases/latest/download/default-repo.json";

#[derive(Debug, Default)]
pub struct StartupPlugins {
    pub repository_jsons: Vec<String>,
    pub bundled_plugin_ids: Vec<String>,
    pub installed_plugin_ids: Vec<String>,
    pub installed_plugin_paths: Vec<(String, String)>,
    pub installed_package_dirs: Vec<PathBuf>,
}

#[derive(Debug)]
struct LoadedRepository {
    id: String,
    json: String,
    base_dir: Option<PathBuf>,
    rows: Vec<PluginMarketplaceRow>,
}

#[derive(Debug, Deserialize)]
struct BundledPlugins {
    #[serde(default)]
    plugins: Vec<String>,
}

pub fn load_startup_plugins(config_root: &Path, config: &AppConfig) -> StartupPlugins {
    log_plugin_info(format!(
        "startup: loading plugin system for profile {}",
        config_root.display()
    ));
    let bundled_plugin_ids = bundled_plugin_ids();
    log_plugin_info(format!(
        "startup: bundled.json lists {} auto-install plugin(s): {}",
        bundled_plugin_ids.len(),
        joined_ids(&bundled_plugin_ids)
    ));
    let mut repositories = load_repositories(&config.settings);
    log_plugin_info(format!(
        "startup: {} plugin repository/repositories loaded",
        repositories.len()
    ));
    let (preinstall_plugin_ids, _, _) = installed_plugins(config_root);
    let installed = preinstall_plugin_ids.into_iter().collect::<BTreeSet<_>>();
    install_bundled_plugins(config_root, &repositories, &bundled_plugin_ids, &installed);
    let (installed_plugin_ids, installed_plugin_paths, installed_package_dirs) =
        installed_plugins(config_root);
    if let Some(repository) = installed_repository(&installed_package_dirs) {
        repositories.push(repository);
    }
    log_plugin_info(format!(
        "startup: plugin system ready; marketplace repositories: {}, installed plugins: {}",
        repositories.len(),
        joined_ids(&installed_plugin_ids)
    ));

    StartupPlugins {
        repository_jsons: repositories
            .into_iter()
            .map(|repository| repository.json)
            .collect(),
        bundled_plugin_ids,
        installed_plugin_ids,
        installed_plugin_paths,
        installed_package_dirs,
    }
}

fn installed_repository(installed_package_dirs: &[PathBuf]) -> Option<LoadedRepository> {
    let mut plugins = Vec::new();
    for path in installed_package_dirs {
        let manifest = read_package_manifest(path).ok()?;
        plugins.push(serde_json::json!({
            "manifest": manifest,
            "install_source": {
                "kind": "local_package",
                "path": path.to_string_lossy()
            }
        }));
    }
    if plugins.is_empty() {
        return None;
    }
    let value = serde_json::json!({
        "repository_format_version": 1,
        "id": "installed-profile-plugins",
        "name": "Installed Plugins",
        "plugins": plugins
    });
    let json = serde_json::to_string(&value).ok()?;
    validate_repository("installed", json, None)
}

fn bundled_plugin_ids() -> Vec<String> {
    match serde_json::from_str::<BundledPlugins>(BUNDLED_JSON) {
        Ok(bundled) => bundled.plugins,
        Err(error) => {
            log_plugin_warning(format!("bundled.json is invalid and was ignored: {error}"));
            Vec::new()
        }
    }
}

fn load_repositories(settings: &Settings) -> Vec<LoadedRepository> {
    let mut repositories = Vec::new();
    if let Some(repository) = load_local_repository() {
        repositories.push(repository);
    }

    for (id, url) in &settings.plugin_repositories {
        log_plugin_info(format!(
            "repository {id}: configured repository source {url}"
        ));
        if let Some(repository) = load_repository_url(id, url) {
            repositories.push(repository);
        }
    }

    if settings.use_default_repo
        && !settings
            .plugin_repositories
            .values()
            .any(|url| url == DEFAULT_REPOSITORY_URL)
    {
        log_plugin_info(format!(
            "repository default: configured default repository source {DEFAULT_REPOSITORY_URL}"
        ));
        if let Some(repository) = load_repository_url("default", DEFAULT_REPOSITORY_URL) {
            repositories.push(repository);
        }
    }

    repositories
}

fn load_local_repository() -> Option<LoadedRepository> {
    let executable = std::env::current_exe().ok()?;
    let executable_dir = executable.parent()?;
    let path = executable_dir.join(LOCAL_REPOSITORY_FILE);
    log_plugin_info(format!(
        "repository local: looking for {} beside executable {}",
        LOCAL_REPOSITORY_FILE,
        executable.display()
    ));
    if !path.exists() {
        log_plugin_info(format!(
            "local plugin repository was not found at {}; packaged local plugins are unavailable",
            path.display()
        ));
        return None;
    }
    load_repository_file("local", &path)
}

fn load_repository_url(id: &str, url: &str) -> Option<LoadedRepository> {
    if url.trim().is_empty() {
        return None;
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        log_plugin_info(format!("repository {id}: fetching {url}"));
        match ureq::get(url).call() {
            Ok(response) => match response.into_string() {
                Ok(text) => validate_repository(id, text, None),
                Err(error) => {
                    log_plugin_warning(format!(
                        "plugin repository {id} at {url} was ignored: {error}"
                    ));
                    None
                }
            },
            Err(error) => {
                log_plugin_warning(format!(
                    "plugin repository {id} at {url} was ignored: {error}"
                ));
                None
            }
        }
    } else {
        log_plugin_info(format!("repository {id}: reading local file {url}"));
        load_repository_file(id, Path::new(url))
    }
}

fn load_repository_file(id: &str, path: &Path) -> Option<LoadedRepository> {
    log_plugin_info(format!(
        "repository {id}: reading JSON from {}",
        path.display()
    ));
    match fs::read_to_string(path) {
        Ok(text) => validate_repository(id, text, path.parent().map(Path::to_path_buf)),
        Err(error) => {
            log_plugin_warning(format!(
                "plugin repository {id} at {} was ignored: {error}",
                path.display()
            ));
            None
        }
    }
}

fn validate_repository(
    id: &str,
    mut json: String,
    base_dir: Option<PathBuf>,
) -> Option<LoadedRepository> {
    if let Some(base_dir) = &base_dir {
        json = resolve_local_package_paths(&json, base_dir).unwrap_or(json);
    }
    match marketplace_rows_from_repository_json(&json) {
        Ok(rows) => {
            log_plugin_info(format!(
                "repository {id}: loaded {} plugin(s): {}",
                rows.len(),
                joined_row_ids(&rows)
            ));
            Some(LoadedRepository {
                id: id.to_owned(),
                json,
                base_dir,
                rows,
            })
        }
        Err(error) => {
            log_plugin_warning(format!("plugin repository {id} was ignored: {error}"));
            None
        }
    }
}

fn resolve_local_package_paths(json: &str, base_dir: &Path) -> Option<String> {
    let mut value = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let plugins = value.get_mut("plugins")?.as_array_mut()?;
    for plugin in plugins {
        let Some(source) = plugin.get_mut("install_source") else {
            continue;
        };
        let is_local = source
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|kind| kind == "local_package");
        if !is_local {
            continue;
        }
        let Some(path_value) = source.get_mut("path") else {
            continue;
        };
        let Some(path) = path_value.as_str() else {
            continue;
        };
        let path = Path::new(path);
        if !path.is_absolute() {
            *path_value = serde_json::Value::String(base_dir.join(path).to_string_lossy().into());
        }
    }
    serde_json::to_string(&value).ok()
}

fn install_bundled_plugins(
    config_root: &Path,
    repositories: &[LoadedRepository],
    bundled_plugin_ids: &[String],
    installed: &BTreeSet<String>,
) {
    if repositories.is_empty() && !bundled_plugin_ids.is_empty() {
        log_plugin_warning(
            "bundled plugins were not installed because no plugin repository was loaded".to_owned(),
        );
        return;
    }
    log_plugin_info(format!(
        "bundled install: checking {} bundled plugin(s)",
        bundled_plugin_ids.len()
    ));
    for plugin_id in bundled_plugin_ids {
        if installed.contains(plugin_id) {
            log_plugin_info(format!(
                "bundled install: {plugin_id} already installed; skipping"
            ));
            continue;
        }
        let Some((repository, row)) = repositories.iter().find_map(|repository| {
            repository
                .rows
                .iter()
                .find(|row| &row.id == plugin_id)
                .map(|row| (repository, row))
        }) else {
            log_plugin_warning(format!(
                "bundled plugin {plugin_id} was not installed because no repository entry was found"
            ));
            continue;
        };
        log_plugin_info(format!(
            "bundled install: installing {plugin_id} from repository {}",
            repository.id
        ));
        if let Err(error) = install_marketplace_plugin(config_root, repository, row) {
            log_plugin_warning(format!(
                "bundled plugin {plugin_id} install failed: {error}"
            ));
        } else {
            log_plugin_info(format!("bundled install: installed {plugin_id}"));
        }
    }
}

fn install_marketplace_plugin(
    config_root: &Path,
    repository: &LoadedRepository,
    row: &PluginMarketplaceRow,
) -> Result<(), String> {
    let destination = plugin_install_dir(config_root, &row.id);
    if destination.exists() {
        log_plugin_info(format!(
            "install: {} already exists at {}; skipping copy/extract",
            row.id,
            destination.display()
        ));
        return Ok(());
    }
    let staging = plugin_staging_dir(config_root, &row.id);
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;

    match &row.install_source {
        PluginMarketplaceSource::LocalPackage { path } => {
            let base = repository.base_dir.as_ref().ok_or_else(|| {
                "local package source has no repository base directory".to_owned()
            })?;
            log_plugin_info(format!(
                "install: copying {} from local package {}",
                row.id,
                base.join(path).display()
            ));
            copy_package_dir(&base.join(path), &staging)?;
        }
        PluginMarketplaceSource::Archive { url, sha256 } => {
            log_plugin_info(format!("install: downloading {} from {url}", row.id));
            let bytes = download_archive(url)?;
            log_plugin_info(format!(
                "install: verifying SHA-256 for {} ({} bytes)",
                row.id,
                bytes.len()
            ));
            verify_sha256(&bytes, sha256)?;
            log_plugin_info(format!("install: extracting archive for {}", row.id));
            extract_archive(&bytes, &staging)?;
        }
        PluginMarketplaceSource::Bundled { .. } | PluginMarketplaceSource::Unknown => {
            return Err("repository entry has no installable package source".to_owned());
        }
    }

    PluginPackage::load(&staging).map_err(|error| error.to_string())?;
    fs::rename(&staging, &destination).map_err(|error| error.to_string())?;
    log_plugin_info(format!(
        "install: {} installed into {}",
        row.id,
        destination.display()
    ));
    Ok(())
}

pub fn installed_plugins(config_root: &Path) -> (Vec<String>, Vec<(String, String)>, Vec<PathBuf>) {
    let root = config_root.join("plugins");
    log_plugin_info(format!(
        "installed scan: checking plugin directory {}",
        root.display()
    ));
    let mut ids = Vec::new();
    let mut id_paths = Vec::new();
    let mut dirs = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        log_plugin_info("installed scan: plugin directory does not exist yet".to_owned());
        return (ids, id_paths, dirs);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        match read_package_manifest(&path) {
            Ok(manifest) => {
                log_plugin_info(format!(
                    "installed scan: found {} at {}",
                    manifest.id,
                    path.display()
                ));
                id_paths.push((manifest.id.clone(), path.to_string_lossy().into_owned()));
                ids.push(manifest.id);
                dirs.push(path);
            }
            Err(error) => log_plugin_warning(format!(
                "installed plugin at {} was ignored: {error}",
                path.display()
            )),
        }
    }
    ids.sort();
    id_paths.sort_by(|left, right| left.0.cmp(&right.0));
    dirs.sort();
    log_plugin_info(format!(
        "installed scan: {} installed plugin(s): {}",
        ids.len(),
        joined_ids(&ids)
    ));
    (ids, id_paths, dirs)
}
