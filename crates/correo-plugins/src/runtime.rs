use crate::{
    HookDispatchError, HookInvocation, HookKind, HookOutput, PluginManifest, PluginPackage,
    RuntimeLoadError,
};
use semver::Version;
use std::fmt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use wasmtime::{
    Config, Engine, ExternType, Instance, Memory, Module, Store, StoreLimits, StoreLimitsBuilder,
    Trap, UpdateDeadline, ValType,
};

const MEMORY_EXPORT: &str = "memory";
const ALLOC_EXPORT: &str = "correomqtt_alloc";
const DEALLOC_EXPORT: &str = "correomqtt_dealloc";
const WASM_PAGE_SIZE: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct WasmSandboxLimits {
    pub max_memory_bytes: usize,
    pub max_fuel: u64,
    pub max_payload_bytes: usize,
}

impl Default for WasmSandboxLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 16 * 1024 * 1024,
            max_fuel: 5_000_000,
            max_payload_bytes: 1024 * 1024,
        }
    }
}

#[derive(Clone)]
pub struct PluginCancellationToken {
    engine: Engine,
    cancelled: Arc<AtomicBool>,
}

impl PluginCancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.engine.increment_epoch();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl fmt::Debug for PluginCancellationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginCancellationToken")
            .field("cancelled", &self.is_cancelled())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct WasmtimePluginRuntime {
    engine: Engine,
    limits: WasmSandboxLimits,
}

impl WasmtimePluginRuntime {
    pub fn new(limits: WasmSandboxLimits) -> Result<Self, RuntimeLoadError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.epoch_interruption(true);
        if let Err(error) = config.cache_config_load_default() {
            eprintln!(
                "plugin: runtime: Wasmtime compilation cache disabled because cache config could not be loaded: {error}"
            );
        }
        let engine = Engine::new(&config).map_err(RuntimeLoadError::Engine)?;

        Ok(Self { engine, limits })
    }

    pub fn cancellation_token(&self) -> PluginCancellationToken {
        PluginCancellationToken {
            engine: self.engine.clone(),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn compile_package(
        &self,
        package: PluginPackage,
        correo_version: &Version,
    ) -> Result<WasmPlugin, RuntimeLoadError> {
        validate_manifest(package.manifest(), correo_version)?;
        let wasm = package.read_wasm()?;
        let module = Module::from_binary(&self.engine, &wasm).map_err(RuntimeLoadError::Compile)?;
        validate_module(package.manifest(), &module, &self.limits)?;

        Ok(WasmPlugin {
            manifest: package.manifest().clone(),
            module,
            engine: self.engine.clone(),
            limits: self.limits.clone(),
        })
    }
}

impl Default for WasmtimePluginRuntime {
    fn default() -> Self {
        Self::new(WasmSandboxLimits::default()).expect("default Wasmtime config should be valid")
    }
}

#[derive(Clone)]
pub struct WasmPlugin {
    manifest: PluginManifest,
    module: Module,
    engine: Engine,
    limits: WasmSandboxLimits,
}

impl WasmPlugin {
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn dispatch(&self, invocation: HookInvocation) -> Result<HookOutput, HookDispatchError> {
        let token = PluginCancellationToken {
            engine: self.engine.clone(),
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        self.dispatch_with_cancel(invocation, &token)
    }

    pub fn dispatch_with_cancel(
        &self,
        invocation: HookInvocation,
        token: &PluginCancellationToken,
    ) -> Result<HookOutput, HookDispatchError> {
        if token.is_cancelled() {
            return Err(HookDispatchError::Cancelled {
                hook: invocation.hook(),
            });
        }

        let hook = invocation.hook();
        let entrypoint = self
            .manifest
            .entrypoint_for(hook)
            .ok_or(HookDispatchError::HookNotDeclared { hook })?;
        let request = invocation.to_request_bytes(&self.limits)?;
        let mut store = new_store(&self.engine, &self.limits, token, hook)?;
        let instance =
            Instance::new(&mut store, &self.module, &[]).map_err(HookDispatchError::Instantiate)?;
        let memory = instance
            .get_memory(&mut store, MEMORY_EXPORT)
            .ok_or(HookDispatchError::MissingMemory)?;
        let request_ptr =
            allocate_guest_bytes(&mut store, &instance, &memory, &request, token, hook)?;

        let func = instance
            .get_typed_func::<(i32, i32), i64>(&mut store, &entrypoint.export)
            .map_err(HookDispatchError::Entrypoint)?;
        let packed = func
            .call(&mut store, (request_ptr, request.len() as i32))
            .map_err(|source| classify_call_error(source, token, hook))?;

        maybe_dealloc(
            &mut store,
            &instance,
            request_ptr,
            request.len() as i32,
            token,
            hook,
        )?;
        let response = read_packed_guest_bytes(
            &mut store,
            &instance,
            &memory,
            packed,
            &self.limits,
            token,
            hook,
        )?;
        invocation.parse_output(response, &self.limits)
    }
}

impl fmt::Debug for WasmPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WasmPlugin")
            .field("id", &self.manifest.id)
            .field("version", &self.manifest.version)
            .finish_non_exhaustive()
    }
}

fn validate_manifest(
    manifest: &PluginManifest,
    correo_version: &Version,
) -> Result<(), RuntimeLoadError> {
    if !manifest.compatible_correomqtt.matches(correo_version) {
        return Err(RuntimeLoadError::IncompatibleCorreoVersion {
            plugin_id: manifest.id.clone(),
            correo_version: correo_version.clone(),
        });
    }

    for surface in [
        crate::HostSurface::Filesystem,
        crate::HostSurface::Network,
        crate::HostSurface::Secrets,
        crate::HostSurface::Mqtt,
        crate::HostSurface::Ui,
    ] {
        if manifest.capabilities.grants_host_surface(surface) {
            return Err(RuntimeLoadError::UnsupportedHostCapability {
                plugin_id: manifest.id.clone(),
                surface,
            });
        }
    }

    Ok(())
}

fn validate_module(
    manifest: &PluginManifest,
    module: &Module,
    limits: &WasmSandboxLimits,
) -> Result<(), RuntimeLoadError> {
    if let Some(import) = module.imports().next() {
        return Err(RuntimeLoadError::HostImportDenied {
            module: import.module().to_owned(),
            name: import.name().to_owned(),
        });
    }

    let memory = export_type(module, MEMORY_EXPORT)
        .and_then(|ty| match ty {
            ExternType::Memory(memory) => Some(memory),
            _ => None,
        })
        .ok_or(RuntimeLoadError::MissingMemoryExport)?;
    if memory.is_64() || memory.is_shared() {
        return Err(RuntimeLoadError::UnsupportedMemoryExport);
    }

    let max_pages = limits.max_memory_bytes.div_ceil(WASM_PAGE_SIZE) as u64;
    if memory.minimum() > max_pages {
        return Err(RuntimeLoadError::InitialMemoryTooLarge {
            initial_pages: memory.minimum(),
            max_pages,
        });
    }

    validate_func(module, ALLOC_EXPORT, &[ValType::I32], &[ValType::I32])?;
    if export_type(module, DEALLOC_EXPORT).is_some() {
        validate_func(module, DEALLOC_EXPORT, &[ValType::I32, ValType::I32], &[])?;
    }

    for entrypoint in &manifest.entrypoints {
        let export = export_type(module, &entrypoint.export).ok_or_else(|| {
            RuntimeLoadError::MissingEntrypointExport {
                hook: entrypoint.hook,
                export: entrypoint.export.clone(),
            }
        })?;
        if !func_type_matches(export, &[ValType::I32, ValType::I32], &[ValType::I64]) {
            return Err(RuntimeLoadError::InvalidExportSignature {
                export: entrypoint.export.clone(),
            });
        }
    }

    Ok(())
}

fn validate_func(
    module: &Module,
    name: &str,
    params: &[ValType],
    results: &[ValType],
) -> Result<(), RuntimeLoadError> {
    let export = export_type(module, name).ok_or_else(|| {
        if name == ALLOC_EXPORT {
            RuntimeLoadError::MissingAllocatorExport
        } else {
            RuntimeLoadError::InvalidExportSignature {
                export: name.to_owned(),
            }
        }
    })?;
    if func_type_matches(export, params, results) {
        Ok(())
    } else {
        Err(RuntimeLoadError::InvalidExportSignature {
            export: name.to_owned(),
        })
    }
}

fn func_type_matches(export: ExternType, params: &[ValType], results: &[ValType]) -> bool {
    match export {
        ExternType::Func(func) => {
            val_types_match(func.params(), params) && val_types_match(func.results(), results)
        }
        _ => false,
    }
}

fn val_types_match<'a>(
    actual: impl ExactSizeIterator<Item = ValType> + 'a,
    expected: &'a [ValType],
) -> bool {
    actual.len() == expected.len()
        && actual
            .zip(expected.iter())
            .all(|(actual, expected)| scalar_val_type_matches(&actual, expected))
}

fn scalar_val_type_matches(actual: &ValType, expected: &ValType) -> bool {
    matches!(
        (actual, expected),
        (ValType::I32, ValType::I32)
            | (ValType::I64, ValType::I64)
            | (ValType::F32, ValType::F32)
            | (ValType::F64, ValType::F64)
            | (ValType::V128, ValType::V128)
    )
}

fn export_type(module: &Module, name: &str) -> Option<ExternType> {
    module
        .exports()
        .find(|export| export.name() == name)
        .map(|export| export.ty())
}

struct StoreState {
    limits: StoreLimits,
}

fn new_store(
    engine: &Engine,
    limits: &WasmSandboxLimits,
    token: &PluginCancellationToken,
    hook: HookKind,
) -> Result<Store<StoreState>, HookDispatchError> {
    let store_limits = StoreLimitsBuilder::new()
        .memory_size(limits.max_memory_bytes)
        .instances(1)
        .memories(1)
        .tables(4)
        .table_elements(1024)
        .trap_on_grow_failure(true)
        .build();
    let mut store = Store::new(
        engine,
        StoreState {
            limits: store_limits,
        },
    );
    store.limiter(|state| &mut state.limits);
    store
        .set_fuel(limits.max_fuel)
        .map_err(|source| HookDispatchError::ConfigureStore { hook, source })?;

    #[cfg(target_has_atomic = "64")]
    {
        let cancelled = token.cancelled.clone();
        store.set_epoch_deadline(1);
        store.epoch_deadline_callback(move |_| {
            if cancelled.load(Ordering::SeqCst) {
                Err(wasmtime::Error::msg("plugin execution cancelled"))
            } else {
                Ok(UpdateDeadline::Continue(1))
            }
        });
    }

    Ok(store)
}

fn allocate_guest_bytes(
    store: &mut Store<StoreState>,
    instance: &Instance,
    memory: &Memory,
    bytes: &[u8],
    token: &PluginCancellationToken,
    hook: HookKind,
) -> Result<i32, HookDispatchError> {
    let len = bytes
        .len()
        .try_into()
        .map_err(|_| HookDispatchError::PayloadTooLarge {
            hook,
            actual: bytes.len(),
            limit: i32::MAX as usize,
        })?;
    let alloc = instance
        .get_typed_func::<i32, i32>(&mut *store, ALLOC_EXPORT)
        .map_err(|source| HookDispatchError::Allocate { hook, source })?;
    let ptr = alloc
        .call(&mut *store, len)
        .map_err(|source| classify_call_error(source, token, hook))?;

    if ptr < 0 {
        return Err(HookDispatchError::InvalidGuestPointer { hook, ptr });
    }

    memory
        .write(&mut *store, ptr as usize, bytes)
        .map_err(|source| HookDispatchError::MemoryWrite { hook, source })?;
    Ok(ptr)
}

fn read_packed_guest_bytes(
    store: &mut Store<StoreState>,
    instance: &Instance,
    memory: &Memory,
    packed: i64,
    limits: &WasmSandboxLimits,
    token: &PluginCancellationToken,
    hook: HookKind,
) -> Result<Vec<u8>, HookDispatchError> {
    let ptr = ((packed as u64) >> 32) as u32;
    let len = (packed as u64 & u32::MAX as u64) as u32;
    let len = len as usize;

    if ptr > i32::MAX as u32 {
        return Err(HookDispatchError::InvalidResponsePointer { hook, ptr });
    }
    if len > limits.max_payload_bytes {
        return Err(HookDispatchError::PayloadTooLarge {
            hook,
            actual: len,
            limit: limits.max_payload_bytes,
        });
    }

    let mut bytes = vec![0; len];
    memory
        .read(&*store, ptr as usize, &mut bytes)
        .map_err(|source| HookDispatchError::MemoryRead { hook, source })?;
    maybe_dealloc(store, instance, ptr as i32, len as i32, token, hook)?;
    Ok(bytes)
}

fn maybe_dealloc(
    store: &mut Store<StoreState>,
    instance: &Instance,
    ptr: i32,
    len: i32,
    token: &PluginCancellationToken,
    hook: HookKind,
) -> Result<(), HookDispatchError> {
    let Some(func) = instance.get_func(&mut *store, DEALLOC_EXPORT) else {
        return Ok(());
    };
    let dealloc = func
        .typed::<(i32, i32), ()>(&mut *store)
        .map_err(|source| HookDispatchError::Deallocate { hook, source })?;
    dealloc.call(&mut *store, (ptr, len)).map_err(|source| {
        if token.is_cancelled() {
            HookDispatchError::Cancelled { hook }
        } else {
            HookDispatchError::Deallocate { hook, source }
        }
    })
}

fn classify_call_error(
    source: wasmtime::Error,
    token: &PluginCancellationToken,
    hook: HookKind,
) -> HookDispatchError {
    if token.is_cancelled() {
        return HookDispatchError::Cancelled { hook };
    }

    if source.downcast_ref::<Trap>() == Some(&Trap::OutOfFuel) {
        HookDispatchError::FuelExhausted { hook }
    } else {
        HookDispatchError::GuestCall { hook, source }
    }
}
