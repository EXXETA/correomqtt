use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitStatus};
use thiserror::Error;

mod package;
mod plugin_repository;
mod plugin_specs;

fn main() -> Result<(), XtaskError> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("check") => cargo(&["check", "--workspace"]),
        Some("test") => cargo(&["test", "--workspace"]),
        Some("package") => package::run(args.collect()),
        Some("package-smoke") => package::smoke(args.collect()),
        Some("plugin-release") => plugin_repository::release(args.collect()),
        Some("plugin-repository") => plugin_repository::run(args.collect()),
        Some("migrate-fixtures") => migrate_fixtures(),
        Some(command) => Err(XtaskError::UnknownCommand(command.to_owned())),
        None => {
            print_help();
            Ok(())
        }
    }
}

fn migrate_fixtures() -> Result<(), XtaskError> {
    let fixtures_dir = Path::new("crates/correo-storage/tests/fixtures");
    if fixtures_dir.exists() {
        cargo(&[
            "test",
            "-p",
            "correo-storage",
            "--test",
            "migration_fixtures",
        ])
    } else {
        println!("migrate-fixtures: no migration fixtures found yet");
        Ok(())
    }
}

pub(crate) fn cargo(args: &[&str]) -> Result<(), XtaskError> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let status = Command::new(cargo).args(args).status()?;
    ensure_success(status, args)
}

pub(crate) fn cargo_dynamic(args: &[String]) -> Result<(), XtaskError> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let status = Command::new(cargo).args(args).status()?;
    let display_args = args.iter().map(String::as_str).collect::<Vec<_>>();
    ensure_success(status, &display_args)
}

fn ensure_success(status: ExitStatus, args: &[&str]) -> Result<(), XtaskError> {
    if status.success() {
        Ok(())
    } else {
        Err(XtaskError::CommandFailed {
            command: format!("cargo {}", args.join(" ")),
            status,
        })
    }
}

fn print_help() {
    println!(
        "Usage: cargo xtask <check|test|package|package-smoke|plugin-release|plugin-repository|migrate-fixtures>"
    );
    println!();
    println!("Package options:");
    println!("  cargo xtask package [--target <triple>] [--out-dir <dir>] [--no-build]");
    println!("  cargo xtask package-smoke [--target <triple>] [--out-dir <dir>] [--no-build]");
    println!(
        "  cargo xtask plugin-release [--out-dir <dir>] [--asset-base-url <url>] [--no-build]"
    );
    println!("  cargo xtask plugin-repository [--out-dir <dir>]");
}

#[derive(Debug, Error)]
pub(crate) enum XtaskError {
    #[error("unknown xtask command: {0}")]
    UnknownCommand(String),
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("unsupported package target: {0}")]
    UnsupportedTarget(String),
    #[error("expected build artifact does not exist: {0}")]
    MissingArtifact(String),
    #[error(
        "package artifact guard failed for {target}: {message}; command: {command}; artifact: {artifact}"
    )]
    PackageGuard {
        target: String,
        command: String,
        artifact: String,
        message: String,
    },
    #[error("command failed: {command} exited with {status}")]
    CommandFailed { command: String, status: ExitStatus },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("plugin repository error: {0}")]
    PluginRepository(#[from] correo_plugins::PluginRepositoryError),
}
