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
