fn static_response_validator_wasm(response: &[u8], pages: u32) -> Vec<u8> {
    let mut wat = allocator_module(pages);
    wat.push_str(&format!(
        "(data (i32.const 1024) \"{}\")\n",
        wat_escape(response)
    ));
    wat.push_str(&format!(
        "(func (export \"correo_message_validator\") (param i32 i32) (result i64) i64.const {})\n)",
        pack_ptr_len(1024, response.len() as u32)
    ));
    wat::parse_str(&wat).unwrap()
}

fn allocator_module(pages: u32) -> String {
    format!(
        r#"(module
  (memory (export "memory") {pages} {pages})
  (global $heap (mut i32) (i32.const 16384))
  (func (export "correomqtt_alloc") (param $len i32) (result i32)
    (local $ptr i32)
    global.get $heap
    local.set $ptr
    global.get $heap
    local.get $len
    i32.add
    global.set $heap
    local.get $ptr)
  (func (export "correomqtt_dealloc") (param i32) (param i32))
"#
    )
}

fn expected_response_bytes(fixture: &NoopHookFixture) -> Vec<u8> {
    match fixture {
        NoopHookFixture::OutgoingMessageTransform {
            expected_response, ..
        } => serde_json::to_vec(expected_response),
        NoopHookFixture::IncomingMessageTransform {
            expected_response, ..
        } => serde_json::to_vec(expected_response),
        NoopHookFixture::MessageValidator {
            expected_response, ..
        } => serde_json::to_vec(expected_response),
        NoopHookFixture::DetailByteTransform {
            expected_response, ..
        } => serde_json::to_vec(expected_response),
        NoopHookFixture::DetailFormatter {
            expected_response, ..
        } => serde_json::to_vec(expected_response),
    }
    .unwrap()
}

fn message_validator_invocation() -> correo_plugins::HookInvocation {
    correo_plugins::HookInvocation::MessageValidator(MessageValidatorRequest::new(MessageDto::new(
        "fixture/validator",
        b"ok".to_vec(),
    )))
}

fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((ptr as u64) << 32) | len as u64
}

fn wat_escape(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("\\{byte:02x}")).collect()
}

fn export_name(hook: HookKind) -> &'static str {
    match hook {
        HookKind::OutgoingMessageTransform => "correo_outgoing_transform",
        HookKind::IncomingMessageTransform => "correo_incoming_transform",
        HookKind::MessageValidator => "correo_message_validator",
        HookKind::DetailByteTransform => "correo_detail_byte_transform",
        HookKind::DetailFormatter => "correo_detail_formatter",
        HookKind::PayloadHighlighter => "correo_payload_highlighter",
    }
}

fn hook_name(hook: HookKind) -> &'static str {
    match hook {
        HookKind::OutgoingMessageTransform => "outgoing_message_transform",
        HookKind::IncomingMessageTransform => "incoming_message_transform",
        HookKind::MessageValidator => "message_validator",
        HookKind::DetailByteTransform => "detail_byte_transform",
        HookKind::DetailFormatter => "detail_formatter",
        HookKind::PayloadHighlighter => "payload_highlighter",
    }
}

fn fixture_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/noop")
}
