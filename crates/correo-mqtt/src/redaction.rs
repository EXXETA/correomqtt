const REDACTED: &str = "[REDACTED]";
const KEY_MATERIAL_REDACTED: &str = "[REDACTED KEY MATERIAL]";
const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "passwd",
    "passphrase",
    "token",
    "credential",
    "credentials",
    "secret",
    "private_key",
    "private key",
    "key_material",
    "decrypted secret material",
];

pub(crate) fn redact_sensitive(input: &str) -> String {
    let without_keys = redact_pem_key_material(input);
    let without_values = redact_key_values(&without_keys);
    redact_url_passwords(&without_values)
}

fn redact_pem_key_material(input: &str) -> String {
    let mut output = String::new();
    let mut redacting_key = false;
    let mut lines = input.split_inclusive('\n').peekable();

    while let Some(line) = lines.next() {
        let line_without_newline = line.trim_end_matches('\n');
        let upper = line_without_newline.to_ascii_uppercase();
        let starts_private_key = upper.contains("-----BEGIN") && upper.contains("PRIVATE KEY-----");

        if starts_private_key {
            output.push_str(KEY_MATERIAL_REDACTED);
            if line.ends_with('\n') {
                output.push('\n');
            }
            redacting_key = true;
            continue;
        }

        if redacting_key {
            if upper.contains("-----END") && upper.contains("PRIVATE KEY-----") {
                redacting_key = false;
            }
            if !line.ends_with('\n') && lines.peek().is_none() {
                break;
            }
            continue;
        }

        output.push_str(line);
    }

    output
}

fn redact_key_values(input: &str) -> String {
    SENSITIVE_KEYS
        .iter()
        .fold(input.to_owned(), |value, key| redact_key(&value, key))
}

fn redact_key(input: &str, key: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0;

    while let Some(relative_start) = find_ascii_case_insensitive(&input[cursor..], key) {
        let key_start = cursor + relative_start;
        let key_end = key_start + key.len();
        let bytes = input.as_bytes();
        let mut delimiter = key_end;

        while delimiter < input.len() && bytes[delimiter].is_ascii_whitespace() {
            delimiter += 1;
        }

        if delimiter >= input.len() || !matches!(bytes[delimiter], b':' | b'=') {
            output.push_str(&input[cursor..key_end]);
            cursor = key_end;
            continue;
        }

        let mut value_start = delimiter + 1;
        while value_start < input.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }

        let value_end = input[value_start..]
            .find([',', ';', '\n', '\r'])
            .map(|offset| value_start + offset)
            .unwrap_or(input.len());

        output.push_str(&input[cursor..value_start]);
        output.push_str(REDACTED);
        cursor = value_end;
    }

    output.push_str(&input[cursor..]);
    output
}

fn redact_url_passwords(input: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0;

    while let Some(scheme_offset) = input[cursor..].find("://") {
        let authority_start = cursor + scheme_offset + 3;
        let authority_end = input[authority_start..]
            .find(['/', '?', '#', ' ', '\n', '\r'])
            .map(|offset| authority_start + offset)
            .unwrap_or(input.len());
        let authority = &input[authority_start..authority_end];

        let Some(at_offset) = authority.rfind('@') else {
            output.push_str(&input[cursor..authority_end]);
            cursor = authority_end;
            continue;
        };
        let Some(colon_offset) = authority[..at_offset].find(':') else {
            output.push_str(&input[cursor..authority_end]);
            cursor = authority_end;
            continue;
        };

        let password_start = authority_start + colon_offset + 1;
        let password_end = authority_start + at_offset;
        output.push_str(&input[cursor..password_start]);
        output.push_str(REDACTED);
        output.push_str(&input[password_end..authority_end]);
        cursor = authority_end;
    }

    output.push_str(&input[cursor..]);
    output
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
