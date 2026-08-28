use crate::abi::{
    DetailByteTransformRequest, DetailByteTransformResponse, DetailFormatterRequest,
    DetailFormatterResponse, IncomingMessageTransformRequest, IncomingMessageTransformResponse,
    MessageValidatorRequest, MessageValidatorResponse, OutgoingMessageTransformRequest,
    OutgoingMessageTransformResponse,
};
use crate::capabilities::HookKind;
use crate::{HookInvocation, HookOutput};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct WasmFixtureHarness {
    root: PathBuf,
}

impl WasmFixtureHarness {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn load_noop_fixture(&self, hook: HookKind) -> Result<NoopHookFixture, FixtureError> {
        let path = self.noop_fixture_path(hook);
        let text = fs::read_to_string(&path).map_err(|source| FixtureError::Io {
            path: path.clone(),
            source,
        })?;
        let fixture = serde_json::from_str::<NoopHookFixture>(&text).map_err(|source| {
            FixtureError::Json {
                path: path.clone(),
                source,
            }
        })?;

        if fixture.hook() == hook {
            Ok(fixture)
        } else {
            Err(FixtureError::HookMismatch {
                path,
                expected: hook,
                found: fixture.hook(),
            })
        }
    }

    pub fn load_all_noop_fixtures(&self) -> Result<Vec<NoopHookFixture>, FixtureError> {
        HookKind::WASM_DISPATCHED
            .into_iter()
            .map(|hook| self.load_noop_fixture(hook))
            .collect()
    }

    pub fn noop_fixture_path(&self, hook: HookKind) -> PathBuf {
        self.root.join(hook.fixture_file_name())
    }
}

impl From<&Path> for WasmFixtureHarness {
    fn from(root: &Path) -> Self {
        Self::new(root)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "hook", rename_all = "snake_case")]
pub enum NoopHookFixture {
    OutgoingMessageTransform {
        request: OutgoingMessageTransformRequest,
        expected_response: OutgoingMessageTransformResponse,
    },
    IncomingMessageTransform {
        request: IncomingMessageTransformRequest,
        expected_response: IncomingMessageTransformResponse,
    },
    MessageValidator {
        request: MessageValidatorRequest,
        expected_response: MessageValidatorResponse,
    },
    DetailByteTransform {
        request: DetailByteTransformRequest,
        expected_response: DetailByteTransformResponse,
    },
    DetailFormatter {
        request: DetailFormatterRequest,
        expected_response: DetailFormatterResponse,
    },
}

impl NoopHookFixture {
    pub fn hook(&self) -> HookKind {
        match self {
            Self::OutgoingMessageTransform { .. } => HookKind::OutgoingMessageTransform,
            Self::IncomingMessageTransform { .. } => HookKind::IncomingMessageTransform,
            Self::MessageValidator { .. } => HookKind::MessageValidator,
            Self::DetailByteTransform { .. } => HookKind::DetailByteTransform,
            Self::DetailFormatter { .. } => HookKind::DetailFormatter,
        }
    }

    pub fn invocation(&self) -> HookInvocation {
        match self {
            Self::OutgoingMessageTransform { request, .. } => {
                HookInvocation::OutgoingMessageTransform(request.clone())
            }
            Self::IncomingMessageTransform { request, .. } => {
                HookInvocation::IncomingMessageTransform(request.clone())
            }
            Self::MessageValidator { request, .. } => {
                HookInvocation::MessageValidator(request.clone())
            }
            Self::DetailByteTransform { request, .. } => {
                HookInvocation::DetailByteTransform(request.clone())
            }
            Self::DetailFormatter { request, .. } => {
                HookInvocation::DetailFormatter(request.clone())
            }
        }
    }

    pub fn expected_output(&self) -> HookOutput {
        match self {
            Self::OutgoingMessageTransform {
                expected_response, ..
            } => HookOutput::OutgoingMessageTransform(expected_response.clone()),
            Self::IncomingMessageTransform {
                expected_response, ..
            } => HookOutput::IncomingMessageTransform(expected_response.clone()),
            Self::MessageValidator {
                expected_response, ..
            } => HookOutput::MessageValidator(expected_response.clone()),
            Self::DetailByteTransform {
                expected_response, ..
            } => HookOutput::DetailByteTransform(expected_response.clone()),
            Self::DetailFormatter {
                expected_response, ..
            } => HookOutput::DetailFormatter(expected_response.clone()),
        }
    }
}

#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("failed to read WASM fixture {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse WASM fixture {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("WASM fixture {path} was for {found:?}, expected {expected:?}")]
    HookMismatch {
        path: PathBuf,
        expected: HookKind,
        found: HookKind,
    },
}
