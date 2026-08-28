use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let workspace_root = workspace_root(&manifest_dir).unwrap_or(manifest_dir);
    let lock_path = workspace_root.join("Cargo.lock");
    let cargo_path = workspace_root.join("Cargo.toml");
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("out dir")).join("about_metadata.rs");

    println!("cargo:rerun-if-changed={}", lock_path.display());
    println!("cargo:rerun-if-changed={}", cargo_path.display());

    let libraries = read_libraries(&lock_path);
    let version = env::var("CARGO_PKG_VERSION").expect("package version");
    let mut generated = String::new();
    generated.push_str("pub const APP_VERSION: &str = ");
    push_quoted(&mut generated, &version);
    generated.push_str(";\n");
    generated.push_str("pub const OPEN_SOURCE_LIBRARIES: &[(&str, &str)] = &[\n");
    for (name, version) in libraries {
        generated.push_str("    (");
        push_quoted(&mut generated, &name);
        generated.push_str(", ");
        push_quoted(&mut generated, &version);
        generated.push_str("),\n");
    }
    generated.push_str("];\n");
    fs::write(out_path, generated).expect("write about metadata");
}

fn workspace_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join("Cargo.lock").is_file())
        .map(Path::to_path_buf)
}

fn read_libraries(lock_path: &Path) -> Vec<(String, String)> {
    let Ok(lock) = fs::read_to_string(lock_path) else {
        return Vec::new();
    };
    let mut libraries = Vec::new();
    let mut name = None;
    let mut version = None;
    let mut source = None;

    for line in lock.lines().chain(["[[package]]"]) {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if source.is_some() {
                if let (Some(name), Some(version)) = (name.take(), version.take()) {
                    libraries.push((name, version));
                }
            }
            name = None;
            version = None;
            source = None;
        } else if let Some(value) = trimmed.strip_prefix("name = ") {
            name = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("version = ") {
            version = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("source = ") {
            source = unquote(value);
        }
    }

    libraries.sort();
    libraries.dedup();
    libraries
}

fn unquote(value: &str) -> Option<String> {
    value
        .trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
}

fn push_quoted(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch => output.push(ch),
        }
    }
    output.push('"');
}
