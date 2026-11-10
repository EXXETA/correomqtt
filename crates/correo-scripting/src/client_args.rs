use correo_mqtt::Qos;
use rquickjs::{Function, Object, Value};

use crate::{executor::HostState, ScriptingError, ScriptingResult};

#[derive(Clone)]
pub(crate) struct PublishInvocation {
    topic: String,
    payload: String,
    qos: Qos,
    retain: bool,
}

#[derive(Clone)]
pub(crate) struct SubscribeInvocation {
    topic_filter: String,
    qos: Qos,
}

#[derive(Clone)]
pub(crate) enum MqttOperation {
    Publish(PublishInvocation),
    Subscribe(SubscribeInvocation),
    Unsubscribe(String),
}

impl MqttOperation {
    pub(crate) fn run(&self, state: &HostState) -> ScriptingResult<()> {
        match self {
            Self::Publish(request) => state.publish(
                request.topic.clone(),
                request.payload.clone(),
                request.qos,
                request.retain,
            ),
            Self::Subscribe(request) => state.subscribe(request.topic_filter.clone(), request.qos),
            Self::Unsubscribe(topic_filter) => state.unsubscribe(topic_filter.clone()),
        }
    }

    pub(crate) fn published_message(&self) -> Option<(&str, &str)> {
        match self {
            Self::Publish(request) => Some((request.topic.as_str(), request.payload.as_str())),
            _ => None,
        }
    }
}

pub(crate) fn parse_publish_args(args: &[Value<'_>]) -> ScriptingResult<PublishInvocation> {
    if args.len() < 2 {
        return Err(ScriptingError::HostApi(
            "publish requires topic plus payload or QoS".to_owned(),
        ));
    }

    let topic = js_string(&args[0], "publish topic")?;
    if args[1].is_number() {
        parse_legacy_publish_args(topic, args)
    } else if args[1].is_string() {
        parse_modern_publish_args(topic, args)
    } else {
        Err(ScriptingError::HostApi(
            "publish second argument must be a payload string or QoS integer".to_owned(),
        ))
    }
}

fn parse_modern_publish_args(
    topic: String,
    args: &[Value<'_>],
) -> ScriptingResult<PublishInvocation> {
    let payload = js_string(&args[1], "publish payload")?;
    let (qos, retain) = match args.len() {
        2 => (Qos::AtMostOnce, false),
        3 => publish_options(Some(js_object(&args[2], "publish options")?))?,
        _ => {
            return Err(ScriptingError::HostApi(
                "publish(topic, payload) accepts only one options object".to_owned(),
            ));
        }
    };
    Ok(PublishInvocation {
        topic,
        payload,
        qos,
        retain,
    })
}

fn parse_legacy_publish_args(
    topic: String,
    args: &[Value<'_>],
) -> ScriptingResult<PublishInvocation> {
    let qos = qos_from_int(js_i32(&args[1], "publish QoS")?)?;
    let (retain, payload) = match args.len() {
        2 => (false, String::new()),
        3 if args[2].is_bool() => (js_bool(&args[2], "publish retained")?, String::new()),
        3 => (false, js_string(&args[2], "publish payload")?),
        4 => (
            js_bool(&args[2], "publish retained")?,
            js_string(&args[3], "publish payload")?,
        ),
        _ => {
            return Err(ScriptingError::HostApi(
                "legacy publish supports (topic, qos[, retained][, payload])".to_owned(),
            ));
        }
    };
    Ok(PublishInvocation {
        topic,
        payload,
        qos,
        retain,
    })
}

pub(crate) fn parse_subscribe_args(args: &[Value<'_>]) -> ScriptingResult<SubscribeInvocation> {
    if args.is_empty() {
        return Err(ScriptingError::HostApi(
            "subscribe requires a topic filter".to_owned(),
        ));
    }

    let topic_filter = js_string(&args[0], "subscribe topic filter")?;
    let (qos, callback_start) = if args.get(1).is_some_and(Value::is_number) {
        (qos_from_int(js_i32(&args[1], "subscribe QoS")?)?, 2)
    } else {
        (Qos::AtMostOnce, 1)
    };
    validate_callback_tail("subscribe", &args[callback_start..], 3)?;
    Ok(SubscribeInvocation { topic_filter, qos })
}

pub(crate) fn parse_unsubscribe_args(args: &[Value<'_>]) -> ScriptingResult<String> {
    if args.len() != 1 {
        return Err(ScriptingError::HostApi(
            "unsubscribe requires exactly one topic filter".to_owned(),
        ));
    }
    js_string(&args[0], "unsubscribe topic filter")
}

pub(crate) fn parse_async_publish_args<'js>(
    args: Vec<Value<'js>>,
) -> ScriptingResult<(
    PublishInvocation,
    Option<Function<'js>>,
    Option<Function<'js>>,
)> {
    let (args, callbacks) = split_trailing_callbacks(args, 2);
    let publish = parse_publish_args(&args)?;
    let (on_success, on_error) = callback_pair(callbacks);
    Ok((publish, on_success, on_error))
}

/// Parsed subscribe call: the subscription plus its success, error and
/// per-message callbacks.
pub(crate) type AsyncSubscribeArgs<'js> = (
    SubscribeInvocation,
    Option<Function<'js>>,
    Option<Function<'js>>,
    Option<Function<'js>>,
);

pub(crate) fn parse_async_subscribe_args(
    args: Vec<Value<'_>>,
) -> ScriptingResult<AsyncSubscribeArgs<'_>> {
    let callback_start = subscribe_callback_start(&args)?;
    validate_callback_tail("subscribe", &args[callback_start..], 3)?;
    let callbacks = args[callback_start..]
        .iter()
        .map(|value| callback_from_value(value, "subscribe callback"))
        .collect::<ScriptingResult<Vec<_>>>()?;
    let subscribe = parse_subscribe_args(&args)?;
    let (on_success, on_error, on_message) = match callbacks.len() {
        0 => (None, None, None),
        1 => (None, None, callbacks.into_iter().next()),
        2 => {
            let (on_success, on_error) = callback_pair(callbacks);
            (on_success, on_error, None)
        }
        3 => {
            let mut callbacks = callbacks.into_iter();
            (callbacks.next(), callbacks.next(), callbacks.next())
        }
        _ => unreachable!("callback tail was already validated"),
    };
    Ok((subscribe, on_success, on_error, on_message))
}

impl SubscribeInvocation {
    pub(crate) fn topic_filter(&self) -> &str {
        &self.topic_filter
    }
}

pub(crate) fn parse_async_unsubscribe_args<'js>(
    args: Vec<Value<'js>>,
) -> ScriptingResult<(String, Option<Function<'js>>, Option<Function<'js>>)> {
    let (args, callbacks) = split_trailing_callbacks(args, 2);
    let topic_filter = parse_unsubscribe_args(&args)?;
    let (on_success, on_error) = callback_pair(callbacks);
    Ok((topic_filter, on_success, on_error))
}

pub(crate) fn parse_async_noop_args<'js>(
    args: Vec<Value<'js>>,
    operation: &str,
) -> ScriptingResult<Option<Function<'js>>> {
    validate_callback_tail(operation, &args, 1)?;
    Ok(args.into_iter().next().and_then(Value::into_function))
}

fn publish_options(options: Option<Object<'_>>) -> ScriptingResult<(Qos, bool)> {
    let Some(options) = options else {
        return Ok((Qos::AtMostOnce, false));
    };

    let qos = if options
        .contains_key("qos")
        .map_err(|error| ScriptingError::HostApi(error.to_string()))?
    {
        let value = options
            .get("qos")
            .map_err(|error| ScriptingError::HostApi(error.to_string()))?;
        qos_from_int(value)?
    } else {
        Qos::AtMostOnce
    };

    let retain = if options
        .contains_key("retain")
        .map_err(|error| ScriptingError::HostApi(error.to_string()))?
    {
        options
            .get("retain")
            .map_err(|error| ScriptingError::HostApi(error.to_string()))?
    } else {
        false
    };
    Ok((qos, retain))
}

fn subscribe_callback_start(args: &[Value<'_>]) -> ScriptingResult<usize> {
    if args.is_empty() {
        return Err(ScriptingError::HostApi(
            "subscribe requires a topic filter".to_owned(),
        ));
    }
    Ok(if args.get(1).is_some_and(Value::is_number) {
        2
    } else {
        1
    })
}

fn split_trailing_callbacks<'js>(
    mut args: Vec<Value<'js>>,
    max_callbacks: usize,
) -> (Vec<Value<'js>>, Vec<Function<'js>>) {
    let mut callbacks = Vec::new();
    while callbacks.len() < max_callbacks && args.last().is_some_and(Value::is_function) {
        let value = args.pop().expect("last value checked");
        callbacks.push(value.into_function().expect("value checked as function"));
    }
    callbacks.reverse();
    (args, callbacks)
}

fn callback_pair<'js>(
    callbacks: Vec<Function<'js>>,
) -> (Option<Function<'js>>, Option<Function<'js>>) {
    let mut callbacks = callbacks.into_iter();
    (callbacks.next(), callbacks.next())
}

fn validate_callback_tail(
    operation: &str,
    values: &[Value<'_>],
    max_callbacks: usize,
) -> ScriptingResult<()> {
    if values.len() > max_callbacks {
        return Err(ScriptingError::HostApi(format!(
            "{operation} received too many callback arguments"
        )));
    }
    for value in values {
        callback_from_value(value, "callback")?;
    }
    Ok(())
}

fn callback_from_value<'js>(value: &Value<'js>, label: &str) -> ScriptingResult<Function<'js>> {
    value
        .clone()
        .into_function()
        .ok_or_else(|| ScriptingError::HostApi(format!("{label} must be a JavaScript function")))
}

fn js_string(value: &Value<'_>, label: &str) -> ScriptingResult<String> {
    if !value.is_string() {
        return Err(ScriptingError::HostApi(format!("{label} must be a string")));
    }
    value
        .get::<String>()
        .map_err(|error| ScriptingError::HostApi(error.to_string()))
}

fn js_i32(value: &Value<'_>, label: &str) -> ScriptingResult<i32> {
    let number = value
        .as_number()
        .ok_or_else(|| ScriptingError::HostApi(format!("{label} must be an integer")))?;
    if number.fract() != 0.0 || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(ScriptingError::HostApi(format!(
            "{label} must be a 32-bit integer"
        )));
    }
    Ok(number as i32)
}

fn js_bool(value: &Value<'_>, label: &str) -> ScriptingResult<bool> {
    value
        .as_bool()
        .ok_or_else(|| ScriptingError::HostApi(format!("{label} must be a boolean")))
}

fn js_object<'js>(value: &Value<'js>, label: &str) -> ScriptingResult<Object<'js>> {
    if !value.is_object() || value.is_function() {
        return Err(ScriptingError::HostApi(format!(
            "{label} must be an object"
        )));
    }
    value
        .clone()
        .into_object()
        .ok_or_else(|| ScriptingError::HostApi(format!("{label} must be an object")))
}

fn qos_from_int(value: i32) -> ScriptingResult<Qos> {
    match value {
        0 => Ok(Qos::AtMostOnce),
        1 => Ok(Qos::AtLeastOnce),
        2 => Ok(Qos::ExactlyOnce),
        _ => Err(ScriptingError::HostApi(format!(
            "unsupported MQTT QoS value: {value}"
        ))),
    }
}
