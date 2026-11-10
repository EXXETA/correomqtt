use crate::{ManifestError, PluginManifest};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
pub const PLUGIN_WASM_FILE: &str = "plugin.wasm";
pub const PLUGIN_ASSETS_DIR: &str = "assets";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginAbi {
    V1,
    V2,
}

impl PluginAbi {
    pub const fn version(self) -> u16 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
        }
    }
}

fn package_abi(manifest: &toml::Value) -> Result<PluginAbi, PackageError> {
    let Some(value) = manifest.get("abi_version") else {
        return Ok(PluginAbi::V1);
    };
    let Some(version) = value.as_integer() else {
        return Err(PackageError::AbiVersionMalformed);
    };
    match version {
        1 => Ok(PluginAbi::V1),
        2 => Ok(PluginAbi::V2),
        version => Err(PackageError::AbiVersionUnsupported { version }),
    }
}

#[derive(Debug, Clone)]
pub struct PluginPackage {
    root: PathBuf,
    manifest_path: PathBuf,
    wasm_path: PathBuf,
    assets_path: Option<PathBuf>,
    manifest: PluginManifest,
    abi: PluginAbi,
}

impl PluginPackage {
    pub fn load(root: impl Into<PathBuf>) -> Result<Self, PackageError> {
        let root = root.into();
        let manifest_path = root.join(PLUGIN_MANIFEST_FILE);
        let wasm_path = root.join(PLUGIN_WASM_FILE);
        let assets_path = root.join(PLUGIN_ASSETS_DIR);

        ensure_regular_file(&manifest_path, PackageFile::Manifest)?;
        ensure_regular_file(&wasm_path, PackageFile::Wasm)?;

        let assets_path = if assets_path.exists() {
            if assets_path.is_dir() {
                Some(assets_path)
            } else {
                return Err(PackageError::InvalidAssetsPath { path: assets_path });
            }
        } else {
            None
        };

        let manifest_text =
            fs::read_to_string(&manifest_path).map_err(|source| PackageError::ReadFile {
                path: manifest_path.clone(),
                source,
            })?;
        let manifest = PluginManifest::from_toml_str(&manifest_text)?;
        let document = manifest_text
            .parse::<toml::Value>()
            .map_err(PackageError::AbiVersionToml)?;
        let abi = package_abi(&document)?;

        Ok(Self {
            root,
            manifest_path,
            wasm_path,
            assets_path,
            manifest,
            abi,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn wasm_path(&self) -> &Path {
        &self.wasm_path
    }

    pub fn assets_path(&self) -> Option<&Path> {
        self.assets_path.as_deref()
    }

    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub const fn abi(&self) -> PluginAbi {
        self.abi
    }

    pub fn read_wasm(&self) -> Result<Vec<u8>, PackageError> {
        fs::read(&self.wasm_path).map_err(|source| PackageError::ReadFile {
            path: self.wasm_path.clone(),
            source,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFile {
    Manifest,
    Wasm,
}

impl PackageFile {
    fn label(self) -> &'static str {
        match self {
            Self::Manifest => PLUGIN_MANIFEST_FILE,
            Self::Wasm => PLUGIN_WASM_FILE,
        }
    }
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("plugin package is missing required {kind} file at {path}")]
    MissingFile { kind: &'static str, path: PathBuf },
    #[error("plugin package path for {kind} is not a regular file: {path}")]
    NotAFile { kind: &'static str, path: PathBuf },
    #[error("plugin assets path exists but is not a directory: {path}")]
    InvalidAssetsPath { path: PathBuf },
    #[error("failed to read plugin package file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("plugin package abi_version is malformed")]
    AbiVersionMalformed,
    #[error("plugin package abi_version {version} is unsupported")]
    AbiVersionUnsupported { version: i64 },
    #[error("plugin package manifest could not be parsed while reading abi_version: {0}")]
    AbiVersionToml(toml::de::Error),
}

fn ensure_regular_file(path: &Path, file: PackageFile) -> Result<(), PackageError> {
    if !path.exists() {
        return Err(PackageError::MissingFile {
            kind: file.label(),
            path: path.to_path_buf(),
        });
    }

    if !path.is_file() {
        return Err(PackageError::NotAFile {
            kind: file.label(),
            path: path.to_path_buf(),
        });
    }

    Ok(())
}
