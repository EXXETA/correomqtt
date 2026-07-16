fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

fn normalize_script_path(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Script name is required.".to_owned());
    }
    if trimmed.contains('\\') {
        return Err("Use forward slashes for script folders.".to_owned());
    }
    let path = if trimmed.ends_with(".js") {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.js")
    };
    let mut components = path.split('/');
    let Some(first) = components.next() else {
        return Err("Script name is required.".to_owned());
    };
    if first == "logs" || first == "executions" {
        return Err("Script names cannot use sidecar storage folders.".to_owned());
    }
    if path
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err("Script name must be a safe relative .js path.".to_owned());
    }
    Ok(path)
}

fn default_script_source(name: &str) -> String {
    format!(
        "const client = clientFactory.getPromiseClient();\nlogger.info('starting {name}');\nqueue.process();\n"
    )
}

fn redact_script_error(mut error: ScriptExecutionError) -> ScriptExecutionError {
    error.message = redact_script_output(&error.message);
    error
}

fn redact_script_output(message: &str) -> String {
    let redacted = redact_sensitive(message);
    let lower = redacted.to_ascii_lowercase();
    if lower.contains("-----begin") && lower.contains("private key") {
        "[REDACTED KEY MATERIAL]".to_owned()
    } else if lower.contains("decrypted password map")
        || lower.contains("export password")
        || lower.contains("key material")
    {
        "[REDACTED SCRIPT OUTPUT: sensitive material]".to_owned()
    } else {
        redacted
    }
}
