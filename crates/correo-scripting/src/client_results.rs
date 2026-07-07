use std::sync::Arc;

use rquickjs::{Ctx, Exception, Function, Promise};

use crate::{
    client_args::{MqttOperation, SubscribeInvocation},
    client_bindings::ConnectivityOperation,
    client_callbacks::{
        dispatch_message_callbacks, register_message_callback, remove_message_callbacks,
        MessageSubscriptions,
    },
    executor::HostState,
    ScriptingError, ScriptingResult,
};

pub(crate) fn finish_async_result<'js>(
    state: &Arc<HostState>,
    result: ScriptingResult<()>,
    on_success: Option<Function<'js>>,
    on_error: Option<Function<'js>>,
    publish: Option<(&MqttOperation, &MessageSubscriptions<'js>)>,
) -> rquickjs::Result<()> {
    match result {
        Ok(()) => {
            if let Some((operation, subscriptions)) = publish {
                dispatch_published_message(operation, subscriptions)?;
            }
            call_optional_callback(on_success)
        }
        Err(ScriptingError::Cancelled) => Err(state.throw_host_error(ScriptingError::Cancelled)),
        Err(_) if on_error.is_none() => Ok(()),
        Err(_) => call_optional_callback(on_error),
    }
}

pub(crate) fn finish_publish_result<'js>(
    state: &Arc<HostState>,
    result: ScriptingResult<()>,
    operation: &MqttOperation,
    subscriptions: &MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    match result {
        Ok(()) => dispatch_published_message(operation, subscriptions),
        Err(error) => Err(state.throw_host_error(error)),
    }
}

pub(crate) fn promise_adapter<'js>(
    ctx: Ctx<'js>,
    state: Arc<HostState>,
    operation: MqttOperation,
    _name: &'static str,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<Promise<'js>> {
    let result = match operation.run(&state) {
        Ok(()) => finish_mqtt_operation(&operation, &subscriptions)
            .map_err(|error| ScriptingError::JavaScriptGuest(error.to_string())),
        Err(error) => Err(error),
    };
    promise_from_result(ctx, &state, result)
}

pub(crate) fn promise_subscribe_adapter<'js>(
    ctx: Ctx<'js>,
    state: Arc<HostState>,
    subscribe: SubscribeInvocation,
    on_message: Option<Function<'js>>,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<Promise<'js>> {
    let topic_filter = subscribe.topic_filter().to_owned();
    let operation = MqttOperation::Subscribe(subscribe);
    let result = operation.run(&state);
    if result.is_ok() {
        register_message_callback(&subscriptions, topic_filter, on_message);
    }
    promise_from_result(ctx, &state, result)
}

pub(crate) fn promise_connectivity_adapter<'js>(
    ctx: Ctx<'js>,
    state: Arc<HostState>,
    operation: ConnectivityOperation,
    _name: &'static str,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<Promise<'js>> {
    let result = match operation.run(&state) {
        Ok(()) => {
            operation.finish(&subscriptions);
            Ok(())
        }
        Err(error) => Err(error),
    };
    promise_from_result(ctx, &state, result)
}

fn promise_from_result<'js>(
    ctx: Ctx<'js>,
    state: &HostState,
    result: ScriptingResult<()>,
) -> rquickjs::Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    match result {
        Ok(()) => resolve.call::<_, ()>(())?,
        Err(error) => reject.call::<_, ()>((promise_exception(ctx, state, &error)?,))?,
    }
    Ok(promise)
}

fn promise_exception<'js>(
    ctx: Ctx<'js>,
    state: &HostState,
    error: &ScriptingError,
) -> rquickjs::Result<Exception<'js>> {
    let exception = Exception::from_message(ctx.clone(), &error.to_string())?;
    state.register_promise_host_error(&ctx, &exception, error);
    Ok(exception)
}

fn finish_mqtt_operation<'js>(
    operation: &MqttOperation,
    subscriptions: &MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    match operation {
        MqttOperation::Publish(_) => dispatch_published_message(operation, subscriptions),
        MqttOperation::Unsubscribe(topic_filter) => {
            remove_message_callbacks(subscriptions, topic_filter);
            Ok(())
        }
        MqttOperation::Subscribe(_) => Ok(()),
    }
}

fn dispatch_published_message<'js>(
    operation: &MqttOperation,
    subscriptions: &MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    if let Some((topic, payload)) = operation.published_message() {
        dispatch_message_callbacks(subscriptions, topic, payload)?;
    }
    Ok(())
}

pub(crate) fn call_optional_callback(callback: Option<Function<'_>>) -> rquickjs::Result<()> {
    if let Some(callback) = callback {
        callback.call::<_, ()>(())
    } else {
        Ok(())
    }
}
