fn hook_execution(
    output: HookOutput,
    host_capabilities: &correo_plugins::HostCapabilityGrants,
    v1_transport_input: Option<PluginTransportMessage>,
) -> Result<PluginHookExecution, PluginHookError> {
    let (output, host_actions) = match output {
        HookOutput::OutgoingMessageTransform(response) => (
            v1_transport_output(
                transform_outcome(response.outcome)?,
                v1_transport_input.clone(),
            )?,
            Vec::new(),
        ),
        HookOutput::IncomingMessageTransform(response) => (
            v1_transport_output(transform_outcome(response.outcome)?, v1_transport_input)?,
            Vec::new(),
        ),
        HookOutput::OutgoingTransportMessageTransform(response) => (
            PluginHookOutput::TransportMessageTransform(transport_transform_outcome(
                response.outcome,
            )?),
            Vec::new(),
        ),
        HookOutput::IncomingTransportMessageTransform(response) => (
            PluginHookOutput::TransportMessageTransform(transport_transform_outcome(
                response.outcome,
            )?),
            Vec::new(),
        ),
        HookOutput::TransportMessageValidator(response) => (
            PluginHookOutput::Validation(match response.result {
                ValidationResultDto::Valid => PluginValidation::Valid,
                ValidationResultDto::Invalid { message } => PluginValidation::Block { message },
            }),
            Vec::new(),
        ),
        HookOutput::MessageValidator(response) => (
            PluginHookOutput::Validation(match response.result {
                ValidationResultDto::Valid => PluginValidation::Valid,
                ValidationResultDto::Invalid { message } => PluginValidation::Block { message },
            }),
            Vec::new(),
        ),
        HookOutput::DetailByteTransform(response) => (
            PluginHookOutput::DetailBytes(DetailBytesOutput {
                bytes: response.bytes,
                content_type: response.content_type,
            }),
            save_payload_actions(response.host_actions, host_capabilities)?,
        ),
        HookOutput::DetailFormatter(response) => (
            PluginHookOutput::DetailFormat(FormattedMessageDetail {
                format: detail_format(response.output.format),
                text: response.output.text,
                content_type: None,
                diagnostics: Vec::new(),
            }),
            Vec::new(),
        ),
    };
    Ok(PluginHookExecution {
        output,
        host_actions,
    })
}

fn save_payload_actions(
    actions: Vec<HostActionDto>,
    host_capabilities: &correo_plugins::HostCapabilityGrants,
) -> Result<Vec<PluginHostAction>, PluginHookError> {
    actions
        .into_iter()
        .map(|action| match action {
            HostActionDto::SavePayload(payload) => {
                if !host_capabilities.grants(HostSurface::MessageSave) {
                    return Err(PluginHookError::failed(
                        "plugin is not granted the message-save host capability",
                    ));
                }
                Ok(PluginHostAction::SavePayload(PluginSavePayload {
                    suggested_file_name: payload.suggested_file_name,
                    bytes: payload.bytes,
                    content_type: payload.content_type,
                }))
            }
        })
        .collect()
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

fn v1_transport_output(
    outcome: MessageTransform,
    input: Option<PluginTransportMessage>,
) -> Result<PluginHookOutput, PluginHookError> {
    let Some(mut input) = input else {
        return Ok(PluginHookOutput::MessageTransform(outcome));
    };
    match outcome {
        MessageTransform::Unchanged => Ok(PluginHookOutput::TransportMessageTransform(
            TransportMessageTransform::Unchanged,
        )),
        MessageTransform::Drop { reason } => Ok(PluginHookOutput::TransportMessageTransform(
            TransportMessageTransform::Drop { reason },
        )),
        MessageTransform::Replace(message) => {
            input.message.address = message.topic;
            input.message.body = message.payload;
            input.message.delivery = DeliverySemantics::new(match message.qos {
                QosLevel::Zero => DeliveryGuarantee::AtMostOnce,
                QosLevel::One => DeliveryGuarantee::AtLeastOnce,
                QosLevel::Two => DeliveryGuarantee::ExactlyOnce,
            });
            input.message.metadata.insert(
                NamespacedName::new("mqtt.qos").expect("MQTT metadata keys are namespaced"),
                serde_json::Value::String(
                    match message.qos {
                        QosLevel::Zero => "at_most_once",
                        QosLevel::One => "at_least_once",
                        QosLevel::Two => "exactly_once",
                    }
                    .to_owned(),
                ),
            );
            input.message.metadata.insert(
                NamespacedName::new("mqtt.retained").expect("MQTT metadata keys are namespaced"),
                serde_json::Value::Bool(message.retained),
            );
            Ok(PluginHookOutput::TransportMessageTransform(
                TransportMessageTransform::Replace(input),
            ))
        }
    }
}

fn legacy_message_from_transport(
    input: PluginTransportMessage,
) -> Result<PluginMessage, PluginHookError> {
    let retained = match input.message.metadata.get_value("mqtt.retained") {
        Some(serde_json::Value::Bool(value)) => *value,
        Some(_) => {
            return Err(PluginHookError::failed(
                "transport metadata mqtt.retained must be a boolean",
            ))
        }
        None => false,
    };
    Ok(PluginMessage {
        topic: input.message.address,
        payload: input.message.body,
        qos: match input.message.delivery.guarantee {
            DeliveryGuarantee::AtMostOnce => QosLevel::Zero,
            DeliveryGuarantee::AtLeastOnce => QosLevel::One,
            DeliveryGuarantee::ExactlyOnce => QosLevel::Two,
        },
        retained,
    })
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

fn transport_input_dto(
    input: PluginTransportMessage,
) -> Result<TransportHookInputDto, PluginHookError> {
    let metadata = input
        .message
        .metadata
        .iter()
        .map(|(name, value)| {
            NamespacedNameDto::new(name.as_str())
                .map(|name| (name, value.clone()))
                .map_err(PluginHookError::failed)
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let capabilities = input
        .capabilities
        .iter()
        .map(|capability| {
            NamespacedNameDto::new(capability.as_str()).map_err(PluginHookError::failed)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    Ok(TransportHookInputDto {
        message: TransportMessageDto {
            address: input.message.address,
            body: input.message.body,
            delivery: delivery_dto(input.message.delivery.guarantee),
            metadata,
        },
        consumer: input.consumer.map(consumer_dto),
        capabilities,
    })
}

fn transport_transform_outcome(
    outcome: TransportMessageTransformOutcomeDto,
) -> Result<TransportMessageTransform, PluginHookError> {
    match outcome {
        TransportMessageTransformOutcomeDto::Unchanged => Ok(TransportMessageTransform::Unchanged),
        TransportMessageTransformOutcomeDto::Replace { input } => Ok(
            TransportMessageTransform::Replace(plugin_transport_message(input)?),
        ),
        TransportMessageTransformOutcomeDto::Drop { reason } => {
            Ok(TransportMessageTransform::Drop { reason })
        }
    }
}

fn plugin_transport_message(
    input: TransportHookInputDto,
) -> Result<PluginTransportMessage, PluginHookError> {
    let mut message = MessageEnvelope::new(
        input.message.address,
        input.message.body,
        DeliverySemantics::new(plugin_delivery(input.message.delivery)),
    );
    for (name, value) in input.message.metadata {
        let name = NamespacedName::new(name.as_str())
            .map_err(|error| PluginHookError::failed(error.to_string()))?;
        message.metadata.insert(name, value);
    }
    let mut capabilities = TransportCapabilities::default();
    for capability in input.capabilities {
        let capability = TransportCapability::new(capability.as_str())
            .map_err(|error| PluginHookError::failed(error.to_string()))?;
        capabilities.insert(capability);
    }
    Ok(PluginTransportMessage {
        message,
        consumer: input.consumer.map(plugin_consumer),
        capabilities,
    })
}

fn delivery_dto(delivery: DeliveryGuarantee) -> DeliveryGuaranteeDto {
    match delivery {
        DeliveryGuarantee::AtMostOnce => DeliveryGuaranteeDto::AtMostOnce,
        DeliveryGuarantee::AtLeastOnce => DeliveryGuaranteeDto::AtLeastOnce,
        DeliveryGuarantee::ExactlyOnce => DeliveryGuaranteeDto::ExactlyOnce,
    }
}

fn plugin_delivery(delivery: DeliveryGuaranteeDto) -> DeliveryGuarantee {
    match delivery {
        DeliveryGuaranteeDto::AtMostOnce => DeliveryGuarantee::AtMostOnce,
        DeliveryGuaranteeDto::AtLeastOnce => DeliveryGuarantee::AtLeastOnce,
        DeliveryGuaranteeDto::ExactlyOnce => DeliveryGuarantee::ExactlyOnce,
    }
}

fn consumer_dto(consumer: ConsumerSelection) -> ConsumerSelectionDto {
    match consumer {
        ConsumerSelection::Address(address) => ConsumerSelectionDto::Address { address },
        ConsumerSelection::Group { address, group } => {
            ConsumerSelectionDto::Group { address, group }
        }
        ConsumerSelection::Queue(name) => ConsumerSelectionDto::Queue { name },
    }
}

fn plugin_consumer(consumer: ConsumerSelectionDto) -> ConsumerSelection {
    match consumer {
        ConsumerSelectionDto::Address { address } => ConsumerSelection::Address(address),
        ConsumerSelectionDto::Group { address, group } => {
            ConsumerSelection::Group { address, group }
        }
        ConsumerSelectionDto::Queue { name } => ConsumerSelection::Queue(name),
    }
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
