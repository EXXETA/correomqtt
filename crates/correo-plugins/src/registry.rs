use crate::{
    IntoPluginDiagnostic, PluginCancellationToken, PluginDiagnostic, PluginPackage,
    RuntimeLoadError, WasmPlugin, WasmtimePluginRuntime,
};
use semver::Version;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug)]
pub struct PluginRegistry {
    correo_version: Version,
    runtime: WasmtimePluginRuntime,
    plugins: BTreeMap<String, WasmPlugin>,
}

impl PluginRegistry {
    pub fn new(correo_version: Version) -> Result<Self, RuntimeLoadError> {
        Ok(Self {
            correo_version,
            runtime: WasmtimePluginRuntime::new(Default::default())?,
            plugins: BTreeMap::new(),
        })
    }

    pub fn with_runtime(correo_version: Version, runtime: WasmtimePluginRuntime) -> Self {
        Self {
            correo_version,
            runtime,
            plugins: BTreeMap::new(),
        }
    }

    pub fn cancellation_token(&self) -> PluginCancellationToken {
        self.runtime.cancellation_token()
    }

    pub fn register_package(
        &mut self,
        package: PluginPackage,
    ) -> Result<&WasmPlugin, RegistryError> {
        let plugin_id = package.manifest().id.clone();
        if self.plugins.contains_key(&plugin_id) {
            return Err(RegistryError::DuplicatePluginId { plugin_id });
        }

        let plugin = self
            .runtime
            .compile_package(package, &self.correo_version)?;
        self.plugins.insert(plugin_id.clone(), plugin);
        Ok(self
            .plugins
            .get(&plugin_id)
            .expect("plugin was just inserted into the registry"))
    }

    pub fn get(&self, plugin_id: &str) -> Option<&WasmPlugin> {
        self.plugins.get(plugin_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &WasmPlugin> {
        self.plugins.values()
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error(transparent)]
    Runtime(#[from] RuntimeLoadError),
    #[error("plugin registry already contains plugin id {plugin_id}")]
    DuplicatePluginId { plugin_id: String },
}

impl IntoPluginDiagnostic for RegistryError {
    fn diagnostic(&self) -> PluginDiagnostic {
        match self {
            Self::Runtime(error) => error.diagnostic(),
            Self::DuplicatePluginId { .. } => PluginDiagnostic::error(self.to_string()),
        }
    }
}
