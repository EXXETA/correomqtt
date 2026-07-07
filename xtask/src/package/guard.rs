use std::fs;
use std::path::{Path, PathBuf};

use crate::XtaskError;

use super::checksums::{artifact_file_name, sha256_file};
use super::PackageOutput;

const RELEASE_ARTIFACT_EXTENSIONS: &[&str] = &["zip", "dmg", "deb", "rpm", "msi"];

pub(super) fn verify(output: &Option<PackageOutput>) -> Result<(), XtaskError> {
    let Some(output) = output else {
        return Ok(());
    };

    let file_name = artifact_file_name(&output.artifact)?;
    let expected_line = format!("{}  {file_name}\n", output.checksum);
    let checksum_path = output.out_dir.join(format!("{file_name}.sha256"));
    let sums_path = output.out_dir.join("SHA256SUMS");

    ensure(
        output,
        output.artifact.exists(),
        format!("missing expected archive {}", output.artifact.display()),
    )?;
    ensure(
        output,
        checksum_path.exists(),
        format!("missing per-archive checksum {}", checksum_path.display()),
    )?;
    ensure(
        output,
        sums_path.exists(),
        format!("missing checksum summary {}", sums_path.display()),
    )?;

    let actual_checksum = sha256_file(&output.artifact)?;
    ensure(
        output,
        actual_checksum == output.checksum,
        format!(
            "archive checksum mismatch for {}: expected {}, actual {}",
            output.artifact.display(),
            output.checksum,
            actual_checksum
        ),
    )?;

    let actual_sha = fs::read_to_string(&checksum_path)?;
    ensure(
        output,
        actual_sha == expected_line,
        format!(
            "{} did not match expected `{}`",
            checksum_path.display(),
            expected_line.trim_end()
        ),
    )?;

    let actual_sums = fs::read_to_string(&sums_path)?;
    ensure(
        output,
        actual_sums.contains(&expected_line),
        format!(
            "{} did not contain the expected package checksum line",
            sums_path.display()
        ),
    )?;

    let zip_files = files_with_extension(&output.out_dir, "zip")?;
    ensure(
        output,
        zip_files == vec![output.artifact.clone()],
        format!(
            "unexpected ZIP outputs in {}: expected [{}], actual [{}]",
            output.out_dir.display(),
            output.artifact.display(),
            display_paths(&zip_files)
        ),
    )?;

    let release_artifacts = files_with_extensions(&output.out_dir, RELEASE_ARTIFACT_EXTENSIONS)?;
    let expected_sidecars = release_artifacts
        .iter()
        .map(|artifact| {
            let artifact_name = artifact_file_name(artifact)?;
            Ok(output.out_dir.join(format!("{artifact_name}.sha256")))
        })
        .collect::<Result<Vec<_>, XtaskError>>()?;
    let actual_sidecars = files_with_suffix(&output.out_dir, ".sha256")?;
    ensure(
        output,
        actual_sidecars == expected_sidecars,
        format!(
            "unexpected checksum sidecars in {}: expected [{}], actual [{}]",
            output.out_dir.display(),
            display_paths(&expected_sidecars),
            display_paths(&actual_sidecars)
        ),
    )?;
    for artifact in &release_artifacts {
        let artifact_name = artifact_file_name(artifact)?;
        let sidecar_path = output.out_dir.join(format!("{artifact_name}.sha256"));
        let artifact_checksum = sha256_file(artifact)?;
        let artifact_line = format!("{artifact_checksum}  {artifact_name}\n");
        ensure(
            output,
            sidecar_path.exists(),
            format!("missing checksum sidecar {}", sidecar_path.display()),
        )?;
        ensure(
            output,
            fs::read_to_string(&sidecar_path)? == artifact_line,
            format!(
                "checksum sidecar {} is stale or invalid",
                sidecar_path.display()
            ),
        )?;
        ensure(
            output,
            actual_sums.contains(&artifact_line),
            format!("SHA256SUMS does not contain {artifact_name}"),
        )?;
    }

    println!("package-smoke: target {}", output.target);
    println!("package-smoke: command {}", output.command);
    println!("package-smoke: artifact {}", output.artifact.display());
    println!("package-smoke: sha256 {}", output.checksum);
    println!("package-smoke: verified {}", checksum_path.display());
    println!("package-smoke: verified {}", sums_path.display());
    Ok(())
}

fn ensure(output: &PackageOutput, condition: bool, message: String) -> Result<(), XtaskError> {
    if condition {
        Ok(())
    } else {
        Err(XtaskError::PackageGuard {
            target: output.target.clone(),
            command: output.command.clone(),
            artifact: output.artifact.display().to_string(),
            message,
        })
    }
}

fn files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>, XtaskError> {
    files_with_extensions(dir, &[extension])
}

fn files_with_extensions(dir: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, XtaskError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| extensions.contains(&value))
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn files_with_suffix(dir: &Path, suffix: &str) -> Result<Vec<PathBuf>, XtaskError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.ends_with(suffix))
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
