use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use correo_core::{
    marketplace_rows_from_repository_json, DetailBytesOutput, FormattedMessageDetail,
    MessageDetailFormat, MessageTransform, PayloadSyntaxKind, PayloadSyntaxSpan,
    PluginConnectionActionRequest, PluginConnectionActionResponse, PluginHookCall, PluginHookError,
    PluginHookExecutor, PluginHookInput, PluginHookKind, PluginHookOutput, PluginHostAction,
    PluginHostActionResponse, PluginInstaller, PluginMarketplaceRow, PluginMarketplaceSource,
    PluginMessage, PluginMetricListNode, PluginMetricRow, PluginOpenWindow, PluginUiNode,
    PluginValidation, PluginWindowCloseRequest, PluginWindowMessage, PluginWindowRenderRequest,
    PluginWindowRenderResponse, QosLevel,
};
use correo_plugins::{
    bundled_plugin_by_id, DetailByteTransformRequest, DetailFormatDto, DetailFormatterRequest,
    HookContextDto, HookInvocation, HookOutput, IncomingMessageTransformRequest, MessageDto,
    MessageTransformOutcomeDto, MessageValidatorRequest, OutgoingMessageTransformRequest,
    PluginManifest, PluginPackage, PluginRegistry, QosDto, ValidationResultDto,
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

#[derive(Debug)]
pub struct InstalledPluginExecutor {
    registry: Mutex<PluginRegistry>,
    config_root: PathBuf,
}

impl InstalledPluginExecutor {
    pub fn load(config_root: PathBuf, package_dirs: &[PathBuf]) -> Result<Self, String> {
        log_plugin_info(format!(
            "runtime: preparing lazy plugin registry for {} installed package directory/directories",
            package_dirs.len()
        ));
        let version =
            semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| error.to_string())?;
        let registry = PluginRegistry::new(version).map_err(|error| error.to_string())?;
        log_plugin_info(
            "runtime: plugin registry ready; WASM packages will compile on first hook use"
                .to_owned(),
        );
        Ok(Self {
            registry: Mutex::new(registry),
            config_root,
        })
    }
}

impl PluginHookExecutor for InstalledPluginExecutor {
    fn execute(&self, call: PluginHookCall) -> Result<PluginHookOutput, PluginHookError> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|_| PluginHookError::failed("plugin registry lock is poisoned"))?;
        if registry.get(&call.plugin_id).is_none() {
            let package_dir = plugin_install_dir(&self.config_root, &call.plugin_id);
            log_plugin_info(format!(
                "runtime: lazy-loading plugin {} from {}",
                call.plugin_id,
                package_dir.display()
            ));
            let package = PluginPackage::load(package_dir)
                .map_err(|error| PluginHookError::failed(error.to_string()))?;
            registry
                .register_package(package)
                .map_err(|error| PluginHookError::failed(error.to_string()))?;
        }
        let plugin = registry.get(&call.plugin_id).ok_or_else(|| {
            PluginHookError::failed(format!("plugin {} is not loaded", call.plugin_id))
        })?;
        let invocation = hook_invocation(call)?;
        plugin
            .dispatch(invocation)
            .map_err(|error| PluginHookError::failed(error.to_string()))
            .and_then(hook_output)
    }

    fn highlight_payload(
        &self,
        text: &str,
        active_plugin_ids: &[String],
    ) -> Option<Vec<PayloadSyntaxSpan>> {
        let xml_active = active_plugin_ids
            .iter()
            .any(|plugin_id| plugin_id == correo_plugins::XML_FORMAT_ID);
        let json_active = active_plugin_ids
            .iter()
            .any(|plugin_id| plugin_id == correo_plugins::JSON_FORMAT_ID);

        if xml_active {
            if let Some(spans) = correo_plugins::highlight_xml(text) {
                return Some(spans.into_iter().map(payload_syntax_span).collect());
            }
        }
        if json_active {
            correo_plugins::highlight_json(text)
                .map(|spans| spans.into_iter().map(payload_syntax_span).collect())
        } else {
            None
        }
    }

    fn connection_action(
        &self,
        request: PluginConnectionActionRequest,
    ) -> Result<PluginConnectionActionResponse, PluginHookError> {
        if request.plugin_id != correo_plugins_systopic::PLUGIN_ID {
            return Err(PluginHookError::failed(format!(
                "plugin {} does not provide connection actions",
                request.plugin_id
            )));
        }
        let response = correo_plugins_systopic::connection_action_clicked(
            &request.action_id,
            &request.connection_name,
        )
        .ok_or_else(|| PluginHookError::failed("unknown plugin connection action"))?;
        Ok(PluginConnectionActionResponse {
            host_actions: response
                .host_actions
                .into_iter()
                .map(|action| sys_topic_host_action(request.connection_id, action))
                .collect(),
            open_window: Some(PluginOpenWindow {
                plugin_id: request.plugin_id,
                action_id: request.action_id,
                connection_id: request.connection_id,
                title: response.open_window.title,
                message_filter_prefix: Some(response.open_window.message_filter_prefix),
                latest_per_topic: response.open_window.latest_per_topic,
            }),
        })
    }

    fn close_window(
        &self,
        request: PluginWindowCloseRequest,
    ) -> Result<PluginHostActionResponse, PluginHookError> {
        if request.plugin_id != correo_plugins_systopic::PLUGIN_ID {
            return Ok(PluginHostActionResponse::default());
        }
        let response = correo_plugins_systopic::connection_window_closed(&request.action_id)
            .ok_or_else(|| PluginHookError::failed("unknown plugin window"))?;
        Ok(PluginHostActionResponse {
            host_actions: response
                .host_actions
                .into_iter()
                .map(|action| sys_topic_host_action(request.connection_id, action))
                .collect(),
        })
    }

    fn render_window(
        &self,
        request: PluginWindowRenderRequest,
    ) -> Result<PluginWindowRenderResponse, PluginHookError> {
        if request.plugin_id != correo_plugins_systopic::PLUGIN_ID {
            return Err(PluginHookError::failed(format!(
                "plugin {} does not provide window rendering",
                request.plugin_id
            )));
        }
        let messages = request
            .messages
            .into_iter()
            .map(sys_topic_message)
            .collect::<Vec<_>>();
        let nodes =
            correo_plugins_systopic::render_window(&request.action_id, &request.broker, &messages)
                .ok_or_else(|| PluginHookError::failed("unknown plugin window"))?
                .into_iter()
                .map(sys_topic_ui_node)
                .collect();
        Ok(PluginWindowRenderResponse { nodes })
    }
}

fn sys_topic_host_action(
    connection_id: correo_mqtt::ConnectionId,
    action: correo_plugins_systopic::SysTopicHostAction,
) -> PluginHostAction {
    match action {
        correo_plugins_systopic::SysTopicHostAction::Subscribe { topic_filter } => {
            PluginHostAction::Subscribe {
                connection_id,
                topic_filter,
                qos: QosLevel::Zero,
            }
        }
        correo_plugins_systopic::SysTopicHostAction::Unsubscribe { topic_filter } => {
            PluginHostAction::Unsubscribe {
                connection_id,
                topic_filter,
            }
        }
    }
}

fn sys_topic_message(message: PluginWindowMessage) -> correo_plugins_systopic::SysTopicMessage {
    correo_plugins_systopic::SysTopicMessage {
        topic: message.topic,
        payload: message.payload,
        qos: message.qos.label().to_owned(),
        retained: message.retained,
        timestamp: message.timestamp,
    }
}

fn sys_topic_ui_node(node: correo_plugins_systopic::SysTopicUiNode) -> PluginUiNode {
    match node {
        correo_plugins_systopic::SysTopicUiNode::Heading { text } => PluginUiNode::Heading { text },
        correo_plugins_systopic::SysTopicUiNode::Label { text } => PluginUiNode::Label { text },
        correo_plugins_systopic::SysTopicUiNode::Separator => PluginUiNode::Separator,
        correo_plugins_systopic::SysTopicUiNode::Table { columns, rows } => {
            PluginUiNode::Table { columns, rows }
        }
        correo_plugins_systopic::SysTopicUiNode::MetricList(list) => {
            PluginUiNode::MetricList(PluginMetricListNode {
                broker: list.broker,
                latest_update: list.latest_update,
                copy_text: list.copy_text,
                rows: list
                    .rows
                    .into_iter()
                    .map(|row| PluginMetricRow {
                        name: row.name,
                        description: row.description,
                        value: row.value,
                    })
                    .collect(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginFileInstaller {
    config_root: PathBuf,
    executable_dir: PathBuf,
}

impl PluginFileInstaller {
    pub fn new(config_root: PathBuf) -> Self {
        let executable_dir = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            config_root,
            executable_dir,
        }
    }
}

impl PluginInstaller for PluginFileInstaller {
    fn install(&self, plugin: &PluginMarketplaceRow) -> Result<String, String> {
        log_plugin_info(format!("marketplace: install requested for {}", plugin.id));
        let repository = LoadedRepository {
            id: "marketplace".to_owned(),
            json: String::new(),
            base_dir: Some(self.executable_dir.clone()),
            rows: Vec::new(),
        };
        install_marketplace_plugin(&self.config_root, &repository, plugin)?;
        Ok(plugin_install_dir(&self.config_root, &plugin.id)
            .to_string_lossy()
            .into_owned())
    }

    fn uninstall(&self, plugin_id: &str) -> Result<(), String> {
        let path = plugin_install_dir(&self.config_root, plugin_id);
        if path.exists() {
            log_plugin_info(format!(
                "marketplace: uninstalling {plugin_id} from {}",
                path.display()
            ));
            fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        } else {
            log_plugin_info(format!(
                "marketplace: uninstall requested for {plugin_id}, but no installed directory exists"
            ));
        }
        Ok(())
    }
}

fn hook_invocation(call: PluginHookCall) -> Result<HookInvocation, PluginHookError> {
    match (call.hook, call.input) {
        (PluginHookKind::OutgoingTransform, PluginHookInput::Message(message)) => Ok(
            HookInvocation::OutgoingMessageTransform(OutgoingMessageTransformRequest {
                abi_version: correo_plugins::ABI_VERSION,
                context: HookContextDto::default(),
                config: call.config,
                message: message_dto(message),
            }),
        ),
        (PluginHookKind::IncomingTransform, PluginHookInput::Message(message)) => Ok(
            HookInvocation::IncomingMessageTransform(IncomingMessageTransformRequest {
                abi_version: correo_plugins::ABI_VERSION,
                context: HookContextDto::default(),
                config: call.config,
                message: message_dto(message),
            }),
        ),
        (PluginHookKind::Validator, PluginHookInput::Message(message)) => {
            Ok(HookInvocation::MessageValidator(MessageValidatorRequest {
                abi_version: correo_plugins::ABI_VERSION,
                context: HookContextDto::default(),
                config: call.config,
                message: message_dto(message),
            }))
        }
        (
            PluginHookKind::DetailTransform,
            PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        ) => Ok(HookInvocation::DetailByteTransform(
            DetailByteTransformRequest {
                abi_version: correo_plugins::ABI_VERSION,
                context: HookContextDto::default(),
                config: call.config,
                bytes,
                content_type,
            },
        )),
        (
            PluginHookKind::DetailFormatter,
            PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        ) => Ok(HookInvocation::DetailFormatter(DetailFormatterRequest {
            abi_version: correo_plugins::ABI_VERSION,
            context: HookContextDto::default(),
            config: call.config,
            bytes,
            content_type,
        })),
        _ => Err(PluginHookError::failed(
            "plugin hook input did not match hook kind",
        )),
    }
}

fn hook_output(output: HookOutput) -> Result<PluginHookOutput, PluginHookError> {
    match output {
        HookOutput::OutgoingMessageTransform(response) => Ok(PluginHookOutput::MessageTransform(
            transform_outcome(response.outcome)?,
        )),
        HookOutput::IncomingMessageTransform(response) => Ok(PluginHookOutput::MessageTransform(
            transform_outcome(response.outcome)?,
        )),
        HookOutput::MessageValidator(response) => {
            Ok(PluginHookOutput::Validation(match response.result {
                ValidationResultDto::Valid => PluginValidation::Valid,
                ValidationResultDto::Invalid { message } => PluginValidation::Block { message },
            }))
        }
        HookOutput::DetailByteTransform(response) => {
            Ok(PluginHookOutput::DetailBytes(DetailBytesOutput {
                bytes: response.bytes,
                content_type: response.content_type,
            }))
        }
        HookOutput::DetailFormatter(response) => {
            Ok(PluginHookOutput::DetailFormat(FormattedMessageDetail {
                format: detail_format(response.output.format),
                text: response.output.text,
                content_type: None,
                diagnostics: Vec::new(),
            }))
        }
    }
}

fn transform_outcome(
    outcome: MessageTransformOutcomeDto,
) -> Result<MessageTransform, PluginHookError> {
    match outcome {
        MessageTransformOutcomeDto::Unchanged => Ok(MessageTransform::Unchanged),
        MessageTransformOutcomeDto::Replace { message } => {
            Ok(MessageTransform::Replace(plugin_message(message)?))
        }
        MessageTransformOutcomeDto::Drop { reason } => Ok(MessageTransform::Drop { reason }),
    }
}

fn message_dto(message: PluginMessage) -> MessageDto {
    MessageDto {
        topic: message.topic,
        payload: message.payload,
        qos: match message.qos {
            QosLevel::Zero => QosDto::AtMostOnce,
            QosLevel::One => QosDto::AtLeastOnce,
            QosLevel::Two => QosDto::ExactlyOnce,
        },
        retained: message.retained,
        properties: Default::default(),
    }
}

fn plugin_message(message: MessageDto) -> Result<PluginMessage, PluginHookError> {
    Ok(PluginMessage {
        topic: message.topic,
        payload: message.payload,
        qos: match message.qos {
            QosDto::AtMostOnce => QosLevel::Zero,
            QosDto::AtLeastOnce => QosLevel::One,
            QosDto::ExactlyOnce => QosLevel::Two,
        },
        retained: message.retained,
    })
}

fn detail_format(format: DetailFormatDto) -> MessageDetailFormat {
    match format {
        DetailFormatDto::PlainText => MessageDetailFormat::PlainText,
        DetailFormatDto::Json => MessageDetailFormat::Json,
        DetailFormatDto::Xml => MessageDetailFormat::Xml,
        DetailFormatDto::Hex => MessageDetailFormat::Hex,
    }
}

fn payload_syntax_span(span: correo_plugins::PayloadSyntaxSpan) -> PayloadSyntaxSpan {
    PayloadSyntaxSpan {
        start: span.start,
        end: span.end,
        kind: match span.kind {
            correo_plugins::PayloadSyntaxKind::Key => PayloadSyntaxKind::Key,
            correo_plugins::PayloadSyntaxKind::String => PayloadSyntaxKind::String,
            correo_plugins::PayloadSyntaxKind::Number => PayloadSyntaxKind::Number,
            correo_plugins::PayloadSyntaxKind::Keyword => PayloadSyntaxKind::Keyword,
            correo_plugins::PayloadSyntaxKind::Punctuation => PayloadSyntaxKind::Punctuation,
            correo_plugins::PayloadSyntaxKind::Tag => PayloadSyntaxKind::Tag,
            correo_plugins::PayloadSyntaxKind::Attribute => PayloadSyntaxKind::Attribute,
            correo_plugins::PayloadSyntaxKind::Comment => PayloadSyntaxKind::Comment,
        },
    }
}

fn read_package_manifest(path: &Path) -> Result<PluginManifest, String> {
    let text = fs::read_to_string(path.join("plugin.toml")).map_err(|error| error.to_string())?;
    let manifest = PluginManifest::from_toml_str(&text).map_err(|error| error.to_string())?;
    if manifest.connection_header_actions.is_empty() {
        if let Some(bundled) = bundled_plugin_by_id(&manifest.id) {
            return Ok(bundled.manifest().clone());
        }
    }
    Ok(manifest)
}

fn copy_package_dir(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.is_dir() {
        return Err(format!(
            "plugin package directory does not exist: {}",
            source.display()
        ));
    }
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            fs::create_dir_all(&to).map_err(|error| error.to_string())?;
            copy_package_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn download_archive(url: &str) -> Result<Vec<u8>, String> {
    let response = ureq::get(url).call().map_err(|error| error.to_string())?;
    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn verify_sha256(bytes: &[u8], expected: &str) -> Result<(), String> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(format!(
            "archive checksum mismatch: expected {}, got {actual}",
            expected.trim()
        ))
    }
}

fn extract_archive(bytes: &[u8], destination: &Path) -> Result<(), String> {
    let cursor = io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|error| error.to_string())?;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|error| error.to_string())?;
        let Some(path) = file.enclosed_name() else {
            return Err("archive contains an unsafe path".to_owned());
        };
        if path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_) | Component::RootDir | Component::ParentDir
            )
        }) {
            return Err("archive contains an unsafe path".to_owned());
        }
        let output = destination.join(path);
        if file.is_dir() {
            fs::create_dir_all(&output).map_err(|error| error.to_string())?;
        } else {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            let mut writer = fs::File::create(&output).map_err(|error| error.to_string())?;
            io::copy(&mut file, &mut writer).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn plugin_install_dir(config_root: &Path, plugin_id: &str) -> PathBuf {
    config_root
        .join("plugins")
        .join(safe_plugin_dir_name(plugin_id))
}

fn plugin_staging_dir(config_root: &Path, plugin_id: &str) -> PathBuf {
    config_root
        .join("plugins")
        .join(format!(".staging-{}", safe_plugin_dir_name(plugin_id)))
}

fn safe_plugin_dir_name(plugin_id: &str) -> String {
    plugin_id
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}

fn log_plugin_warning(message: String) {
    eprintln!("plugin: {message}");
    tracing::warn!(target: "correo_plugins", "{message}");
}

fn log_plugin_info(message: String) {
    eprintln!("plugin: {message}");
    tracing::info!(target: "correo_plugins", "{message}");
}

fn joined_ids(ids: &[String]) -> String {
    if ids.is_empty() {
        "none".to_owned()
    } else {
        ids.join(", ")
    }
}

fn joined_row_ids(rows: &[PluginMarketplaceRow]) -> String {
    if rows.is_empty() {
        "none".to_owned()
    } else {
        rows.iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
