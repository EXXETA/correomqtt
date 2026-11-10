use std::rc::Rc;

use rquickjs::{prelude::Rest, Ctx, Function, Object, Value};

use crate::{
    client_args::{
        parse_async_noop_args, parse_async_publish_args, parse_async_subscribe_args,
        parse_async_unsubscribe_args, parse_publish_args, parse_unsubscribe_args, MqttOperation,
    },
    client_callbacks::{
        clear_message_callbacks, new_message_subscriptions, register_incoming_transform,
        register_message_callback, remove_message_callbacks, MessageSubscriptions,
    },
    client_results::{
        call_optional_callback, finish_async_result, finish_publish_result, promise_adapter,
        promise_connectivity_adapter, promise_subscribe_adapter,
    },
    executor::HostState,
    ScriptingError, ScriptingResult,
};

pub(crate) fn build_client_factory<'js>(
    ctx: Ctx<'js>,
    state: Rc<HostState>,
) -> rquickjs::Result<Object<'js>> {
    let factory = Object::new(ctx.clone())?;
    factory.set(
        "getBlockingClient",
        function_creating_client(
            ctx.clone(),
            "getBlockingClient",
            state.clone(),
            ClientMode::Blocking,
        )?,
    )?;
    factory.set(
        "getAsyncClient",
        function_creating_client(
            ctx.clone(),
            "getAsyncClient",
            state.clone(),
            ClientMode::Async,
        )?,
    )?;
    factory.set(
        "getPromiseClient",
        function_creating_client(ctx, "getPromiseClient", state, ClientMode::Promise)?,
    )?;
    Ok(factory)
}

#[derive(Clone, Copy)]
enum ClientMode {
    Blocking,
    Async,
    Promise,
}

fn build_client<'js>(
    ctx: Ctx<'js>,
    state: Rc<HostState>,
    mode: ClientMode,
) -> rquickjs::Result<Object<'js>> {
    let client = Object::new(ctx.clone())?;
    let subscriptions = new_message_subscriptions();
    install_connectivity(
        ctx.clone(),
        &client,
        state.clone(),
        mode,
        subscriptions.clone(),
    )?;
    install_publish(
        ctx.clone(),
        &client,
        state.clone(),
        mode,
        subscriptions.clone(),
    )?;
    install_subscribe(
        ctx.clone(),
        &client,
        state.clone(),
        mode,
        subscriptions.clone(),
    )?;
    install_incoming_message(ctx.clone(), &client, subscriptions.clone())?;
    install_conversions(ctx.clone(), &client, state.clone())?;
    install_unsubscribe(ctx, &client, state, mode, subscriptions)?;
    Ok(client)
}

fn install_conversions<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Rc<HostState>,
) -> rquickjs::Result<()> {
    let promise_state = state.clone();
    client.set(
        "toPromised",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>| {
            build_client(ctx, promise_state.clone(), ClientMode::Promise)
        })?
        .with_name("toPromised")?,
    )?;
    client.set(
        "toBlocking",
        Function::new(ctx, move |ctx: Ctx<'js>| {
            build_client(ctx, state.clone(), ClientMode::Blocking)
        })?
        .with_name("toBlocking")?,
    )
}

fn install_connectivity<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Rc<HostState>,
    mode: ClientMode,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    for operation in [
        ConnectivityOperation::Connect,
        ConnectivityOperation::Disconnect,
        ConnectivityOperation::UnsubscribeAll,
    ] {
        install_connectivity_method(
            ctx.clone(),
            client,
            state.clone(),
            mode,
            operation,
            subscriptions.clone(),
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum ConnectivityOperation {
    Connect,
    Disconnect,
    UnsubscribeAll,
}

impl ConnectivityOperation {
    fn name(self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::Disconnect => "disconnect",
            Self::UnsubscribeAll => "unsubscribeAll",
        }
    }

    pub(crate) fn run(self, state: &HostState) -> ScriptingResult<()> {
        match self {
            Self::Connect => state.connect(),
            Self::Disconnect => state.disconnect(),
            Self::UnsubscribeAll => state.check_cancelled(),
        }
    }

    pub(crate) fn finish(self, subscriptions: &MessageSubscriptions<'_>) {
        if matches!(self, Self::UnsubscribeAll) {
            clear_message_callbacks(subscriptions);
        }
    }
}

fn install_connectivity_method<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Rc<HostState>,
    mode: ClientMode,
    operation: ConnectivityOperation,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    let name = operation.name();
    match mode {
        ClientMode::Blocking => client.set(
            name,
            Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                let on_success = parse_async_noop_args(args, name)
                    .map_err(|error| state.throw_host_error(error))?;
                operation
                    .run(&state)
                    .map_err(|error| state.throw_host_error(error))?;
                operation.finish(&subscriptions);
                call_optional_callback(on_success)
            })?
            .with_name(name)?,
        ),
        ClientMode::Async => client.set(
            name,
            Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                let on_success = parse_async_noop_args(args, name)
                    .map_err(|error| state.throw_host_error(error))?;
                finish_connectivity_async_result(
                    &state,
                    operation.run(&state),
                    on_success,
                    None,
                    operation,
                    &subscriptions,
                )
            })?
            .with_name(name)?,
        ),
        ClientMode::Promise => client.set(
            name,
            Function::new(ctx, move |ctx: Ctx<'js>, Rest(args): Rest<Value<'js>>| {
                parse_async_noop_args(args, name).map_err(|error| state.throw_host_error(error))?;
                promise_connectivity_adapter(
                    ctx,
                    state.clone(),
                    operation,
                    name,
                    subscriptions.clone(),
                )
            })?
            .with_name(name)?,
        ),
    }
}

fn function_creating_client<'js>(
    ctx: Ctx<'js>,
    name: &'static str,
    state: Rc<HostState>,
    mode: ClientMode,
) -> rquickjs::Result<Function<'js>> {
    Function::new(ctx, move |ctx: Ctx<'js>| {
        build_client(ctx, state.clone(), mode)
    })?
    .with_name(name)
}

fn install_publish<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Rc<HostState>,
    mode: ClientMode,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    match mode {
        ClientMode::Blocking => {
            let publish_state = state;
            client.set(
                "publish",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let operation = parse_publish_args(&args)
                        .map(MqttOperation::Publish)
                        .map_err(|error| publish_state.throw_host_error(error))?;
                    finish_publish_result(
                        &publish_state,
                        operation.run(&publish_state),
                        &operation,
                        &subscriptions,
                    )
                })?
                .with_name("publish")?,
            )
        }
        ClientMode::Async => {
            let publish_state = state;
            client.set(
                "publish",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let (publish, on_success, on_error) = parse_async_publish_args(args)
                        .map_err(|error| publish_state.throw_host_error(error))?;
                    let operation = MqttOperation::Publish(publish);
                    finish_async_result(
                        &publish_state,
                        operation.run(&publish_state),
                        on_success,
                        on_error,
                        Some((&operation, &subscriptions)),
                    )
                })?
                .with_name("publish")?,
            )
        }
        ClientMode::Promise => {
            let publish_state = state;
            client.set(
                "publish",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'js>, Rest(args): Rest<Value<'js>>| {
                        let operation = parse_publish_args(&args)
                            .map(MqttOperation::Publish)
                            .map_err(|error| publish_state.throw_host_error(error))?;
                        promise_adapter(
                            ctx,
                            publish_state.clone(),
                            operation,
                            "publish",
                            subscriptions.clone(),
                        )
                    },
                )?
                .with_name("publish")?,
            )
        }
    }
}

fn install_subscribe<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Rc<HostState>,
    mode: ClientMode,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    match mode {
        ClientMode::Blocking => {
            let subscribe_state = state;
            client.set(
                "subscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let (subscribe, _, _, on_message) = parse_async_subscribe_args(args)
                        .map_err(|error| subscribe_state.throw_host_error(error))?;
                    let topic_filter = subscribe.topic_filter().to_owned();
                    let result = MqttOperation::Subscribe(subscribe).run(&subscribe_state);
                    if result.is_ok() {
                        register_message_callback(&subscriptions, topic_filter, on_message);
                    }
                    result.map_err(|error| subscribe_state.throw_host_error(error))
                })?
                .with_name("subscribe")?,
            )
        }
        ClientMode::Async => {
            let subscribe_state = state;
            client.set(
                "subscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let (subscribe, on_success, on_error, on_message) =
                        parse_async_subscribe_args(args)
                            .map_err(|error| subscribe_state.throw_host_error(error))?;
                    let topic_filter = subscribe.topic_filter().to_owned();
                    let result = MqttOperation::Subscribe(subscribe).run(&subscribe_state);
                    if result.is_ok() {
                        register_message_callback(&subscriptions, topic_filter, on_message);
                    }
                    finish_async_result(&subscribe_state, result, on_success, on_error, None)
                })?
                .with_name("subscribe")?,
            )
        }
        ClientMode::Promise => {
            let subscribe_state = state;
            client.set(
                "subscribe",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'js>, Rest(args): Rest<Value<'js>>| {
                        let (subscribe, _, _, on_message) = parse_async_subscribe_args(args)
                            .map_err(|error| subscribe_state.throw_host_error(error))?;
                        promise_subscribe_adapter(
                            ctx,
                            subscribe_state.clone(),
                            subscribe,
                            on_message,
                            subscriptions.clone(),
                        )
                    },
                )?
                .with_name("subscribe")?,
            )
        }
    }
}

fn install_incoming_message<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    client.set(
        "onIncomingMessage",
        Function::new(ctx, move |topic_filter: String, callback: Function<'js>| {
            register_incoming_transform(&subscriptions, topic_filter, callback);
        })?
        .with_name("onIncomingMessage")?,
    )
}

fn install_unsubscribe<'js>(
    ctx: Ctx<'js>,
    client: &Object<'js>,
    state: Rc<HostState>,
    mode: ClientMode,
    subscriptions: MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    match mode {
        ClientMode::Blocking => {
            let unsubscribe_state = state;
            client.set(
                "unsubscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let topic_filter = parse_unsubscribe_args(&args)
                        .map_err(|error| unsubscribe_state.throw_host_error(error))?;
                    let result =
                        MqttOperation::Unsubscribe(topic_filter.clone()).run(&unsubscribe_state);
                    if result.is_ok() {
                        remove_message_callbacks(&subscriptions, &topic_filter);
                    }
                    result.map_err(|error| unsubscribe_state.throw_host_error(error))
                })?
                .with_name("unsubscribe")?,
            )
        }
        ClientMode::Async => {
            let unsubscribe_state = state;
            client.set(
                "unsubscribe",
                Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
                    let (topic_filter, on_success, on_error) =
                        parse_async_unsubscribe_args(args)
                            .map_err(|error| unsubscribe_state.throw_host_error(error))?;
                    let result =
                        MqttOperation::Unsubscribe(topic_filter.clone()).run(&unsubscribe_state);
                    if result.is_ok() {
                        remove_message_callbacks(&subscriptions, &topic_filter);
                    }
                    finish_async_result(&unsubscribe_state, result, on_success, on_error, None)
                })?
                .with_name("unsubscribe")?,
            )
        }
        ClientMode::Promise => {
            let unsubscribe_state = state;
            client.set(
                "unsubscribe",
                Function::new(
                    ctx.clone(),
                    move |ctx: Ctx<'js>, Rest(args): Rest<Value<'js>>| {
                        let operation = parse_unsubscribe_args(&args)
                            .map(MqttOperation::Unsubscribe)
                            .map_err(|error| unsubscribe_state.throw_host_error(error))?;
                        promise_adapter(
                            ctx,
                            unsubscribe_state.clone(),
                            operation,
                            "unsubscribe",
                            subscriptions.clone(),
                        )
                    },
                )?
                .with_name("unsubscribe")?,
            )
        }
    }
}

fn finish_connectivity_async_result<'js>(
    state: &Rc<HostState>,
    result: ScriptingResult<()>,
    on_success: Option<Function<'js>>,
    on_error: Option<Function<'js>>,
    operation: ConnectivityOperation,
    subscriptions: &MessageSubscriptions<'js>,
) -> rquickjs::Result<()> {
    match result {
        Ok(()) => {
            operation.finish(subscriptions);
            call_optional_callback(on_success)
        }
        Err(ScriptingError::Cancelled) => Err(state.throw_host_error(ScriptingError::Cancelled)),
        Err(_) if on_error.is_none() => Ok(()),
        Err(_) => call_optional_callback(on_error),
    }
}
