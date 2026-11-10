use std::rc::Rc;

use base64::{engine::general_purpose::STANDARD, Engine};
use rquickjs::{prelude::Rest, Ctx, Function, Object, Value};

use crate::{client_bindings::build_client_factory, executor::HostState, ScriptLogLevel};

pub(crate) fn install_bindings<'js>(ctx: Ctx<'js>, state: Rc<HostState>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    globals.set("logger", build_logger(ctx.clone(), state.clone())?)?;

    let sleep_state = state.clone();
    globals.set(
        "sleep",
        Function::new(ctx.clone(), move |millis: i32| {
            sleep_state
                .sleep(millis)
                .map_err(|error| sleep_state.throw_host_error(error))
        })?
        .with_name("sleep")?,
    )?;

    let join_state = state.clone();
    globals.set(
        "join",
        Function::new(ctx.clone(), move || {
            join_state
                .check_cancelled()
                .map_err(|error| join_state.throw_host_error(error))
        })?
        .with_name("join")?,
    )?;

    globals.set("queue", build_queue(ctx.clone(), state.clone())?)?;
    globals.set("plugins", build_plugins(ctx.clone(), state.clone())?)?;
    globals.set("clientFactory", build_client_factory(ctx.clone(), state)?)?;
    ctx.eval::<(), _>(
        "globalThis.ClientFactory = function ClientFactory() { return globalThis.clientFactory; };",
    )?;
    Ok(())
}

fn build_plugins<'js>(ctx: Ctx<'js>, state: Rc<HostState>) -> rquickjs::Result<Object<'js>> {
    let plugins = Object::new(ctx.clone())?;
    let base64 = Object::new(ctx.clone())?;
    let decode_state = state.clone();
    base64.set(
        "decode",
        Function::new(ctx.clone(), move |payload: String| {
            let bytes = STANDARD.decode(payload).map_err(|error| {
                decode_state.throw_host_error(crate::ScriptingError::HostApi(error.to_string()))
            })?;
            String::from_utf8(bytes).map_err(|error| {
                decode_state.throw_host_error(crate::ScriptingError::HostApi(error.to_string()))
            })
        })?
        .with_name("plugins.base64.decode")?,
    )?;
    base64.set(
        "encode",
        Function::new(ctx, move |payload: String| STANDARD.encode(payload))?
            .with_name("plugins.base64.encode")?,
    )?;
    plugins.set("base64", base64)?;
    Ok(plugins)
}

fn build_logger<'js>(ctx: Ctx<'js>, state: Rc<HostState>) -> rquickjs::Result<Object<'js>> {
    let logger = Object::new(ctx.clone())?;
    let debug_state = state.clone();
    logger.set(
        "debug",
        Function::new(ctx.clone(), move |Rest(args): Rest<Value<'js>>| {
            let message = log_message(args)?;
            debug_state
                .log(ScriptLogLevel::Debug, message)
                .map_err(|error| debug_state.throw_host_error(error))
        })?
        .with_name("logger.debug")?,
    )?;

    let info_state = state.clone();
    logger.set(
        "info",
        Function::new(ctx.clone(), move |Rest(args): Rest<Value<'js>>| {
            let message = log_message(args)?;
            info_state
                .log(ScriptLogLevel::Info, message)
                .map_err(|error| info_state.throw_host_error(error))
        })?
        .with_name("logger.info")?,
    )?;

    let warn_state = state.clone();
    logger.set(
        "warn",
        Function::new(ctx.clone(), move |Rest(args): Rest<Value<'js>>| {
            let message = log_message(args)?;
            warn_state
                .log(ScriptLogLevel::Warning, message)
                .map_err(|error| warn_state.throw_host_error(error))
        })?
        .with_name("logger.warn")?,
    )?;

    let error_state = state;
    logger.set(
        "error",
        Function::new(ctx, move |Rest(args): Rest<Value<'js>>| {
            let message = log_message(args)?;
            error_state
                .log(ScriptLogLevel::Error, message)
                .map_err(|error| error_state.throw_host_error(error))
        })?
        .with_name("logger.error")?,
    )?;

    Ok(logger)
}

fn log_message(args: Vec<Value<'_>>) -> rquickjs::Result<String> {
    let Some((template, values)) = args.split_first() else {
        return Ok(String::new());
    };
    let mut message = value_string(template)?;
    for value in values {
        let replacement = value_string(value)?;
        if let Some(index) = message.find("{}") {
            message.replace_range(index..index + 2, &replacement);
        } else {
            if !message.is_empty() {
                message.push(' ');
            }
            message.push_str(&replacement);
        }
    }
    Ok(message)
}

fn value_string(value: &Value<'_>) -> rquickjs::Result<String> {
    if value.is_string() {
        value.get::<String>()
    } else if let Some(number) = value.as_number() {
        Ok(number.to_string())
    } else if let Some(boolean) = value.as_bool() {
        Ok(boolean.to_string())
    } else {
        Ok("[object Object]".to_owned())
    }
}

fn build_queue<'js>(ctx: Ctx<'js>, state: Rc<HostState>) -> rquickjs::Result<Object<'js>> {
    let queue = Object::new(ctx.clone())?;
    let process_state = state.clone();
    queue.set(
        "process",
        Function::new(ctx.clone(), move || {
            process_state
                .queue_process()
                .map_err(|error| process_state.throw_host_error(error))
        })?
        .with_name("queue.process")?,
    )?;

    let jump_out_state = state;
    queue.set(
        "jumpOut",
        Function::new(ctx, move || {
            jump_out_state
                .queue_jump_out()
                .map_err(|error| jump_out_state.throw_host_error(error))
        })?
        .with_name("queue.jumpOut")?,
    )?;
    Ok(queue)
}
