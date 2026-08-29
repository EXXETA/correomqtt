use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{checksums, write_file, PackagePlan, APP_ID, APP_NAME, BIN_NAME, VENDOR};
use crate::XtaskError;

pub(super) fn record_extra_artifact(
    label: &str,
    path: &Path,
    plan: &PackagePlan,
) -> Result<(), XtaskError> {
    checksums::write_sidecar(path, &plan.out_dir)?;
    println!("{label}: {}", path.display());
    Ok(())
}

// Runs an external packaging tool; a missing binary downgrades to a skip so
// plain zip packaging keeps working everywhere.
fn run_or_skip(
    command: &mut Command,
    skip_message: &str,
) -> Result<Option<std::process::ExitStatus>, XtaskError> {
    match command.status() {
        Ok(status) => Ok(Some(status)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            println!("{skip_message}");
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

// DEB and RPM via nfpm (single cross-platform binary). Skipped when nfpm is
// not installed, so plain zip packaging keeps working everywhere.
pub(super) fn create_linux_installers(
    stage_dir: &Path,
    plan: &PackagePlan,
) -> Result<Vec<PathBuf>, XtaskError> {
    let Some(arch) = nfpm_arch(&plan.target) else {
        println!(
            "deb/rpm: skipped (unsupported architecture in {})",
            plan.target
        );
        return Ok(Vec::new());
    };
    let config_path = stage_dir.join("nfpm.yaml");
    write_file(&config_path, nfpm_config(stage_dir, arch).as_bytes())?;

    let targets: Vec<(&str, PathBuf)> = ["deb", "rpm"]
        .into_iter()
        .map(|packager| {
            let artifact = plan.out_dir.join(
                plan.artifact_file_name()
                    .replace(".zip", &format!(".{packager}")),
            );
            (packager, artifact)
        })
        .collect();

    // Remove every expected artifact up front: if nfpm is missing the loop
    // returns after the first packager, so a stale .rpm from an earlier run
    // must not survive and slip back into SHA256SUMS.
    for (_, artifact) in &targets {
        if artifact.exists() {
            fs::remove_file(artifact)?;
        }
    }

    let mut artifacts = Vec::new();
    for (packager, artifact) in targets {
        let Some(status) = run_or_skip(
            Command::new("nfpm")
                .arg("package")
                .arg("--config")
                .arg(&config_path)
                .arg("--packager")
                .arg(packager)
                .arg("--target")
                .arg(&artifact),
            "deb/rpm: skipped (nfpm is not installed)",
        )?
        else {
            return Ok(artifacts);
        };
        if !status.success() {
            return Err(XtaskError::CommandFailed {
                command: format!("nfpm package --packager {packager}"),
                status,
            });
        }
        artifacts.push(artifact);
    }
    Ok(artifacts)
}

fn nfpm_arch(target: &str) -> Option<&'static str> {
    match target.split('-').next() {
        Some("x86_64") => Some("amd64"),
        Some("aarch64") => Some("arm64"),
        _ => None,
    }
}

pub(super) fn nfpm_config(stage_dir: &Path, arch: &str) -> String {
    // Forward slashes: backslash paths are invalid escapes in the quoted
    // YAML strings when cross-building from a Windows host.
    let root = stage_dir
        .join(APP_NAME)
        .display()
        .to_string()
        .replace('\\', "/");
    // The app resolves its bundled plugin repository next to the executable
    // (current_exe().parent()), so the whole app tree — binary,
    // local-repo.json and plugins/ — is installed under /usr/lib/correomqtt/
    // with a /usr/bin symlink. Linux current_exe() resolves the symlink to the
    // real path, so parent() points at the plugin directory.
    format!(
        r#"name: {BIN_NAME}
arch: {arch}
platform: linux
version: {version}
maintainer: "{VENDOR}"
vendor: "{VENDOR}"
homepage: http://correomqtt.org
license: GPL-3.0-or-later
description: Native MQTT desktop client
contents:
- src: "{root}/bin/{BIN_NAME}"
  dst: /usr/lib/{BIN_NAME}/{BIN_NAME}
  file_info:
    mode: 0755
- src: "{root}/bin/local-repo.json"
  dst: /usr/lib/{BIN_NAME}/local-repo.json
- src: "{root}/bin/plugins"
  dst: /usr/lib/{BIN_NAME}/plugins
  type: tree
- src: /usr/lib/{BIN_NAME}/{BIN_NAME}
  dst: /usr/bin/{BIN_NAME}
  type: symlink
- src: "{root}/share/applications/{APP_ID}.desktop"
  dst: /usr/share/applications/{APP_ID}.desktop
- src: "{root}/share/icons/hicolor/256x256/apps/{APP_ID}.png"
  dst: /usr/share/icons/hicolor/256x256/apps/{APP_ID}.png
- src: "{root}/share/metainfo/{APP_ID}.metainfo.xml"
  dst: /usr/share/metainfo/{APP_ID}.metainfo.xml
"#,
        version = env!("CARGO_PKG_VERSION"),
    )
}

// MSI via the WiX v4+ dotnet tool (`dotnet tool install --global wix`).
// Harvests the whole staged app directory (binary, local-repo.json, plugins/)
// into the install folder plus a start-menu shortcut. Skipped when wix is not
// installed.
pub(super) fn create_msi(
    stage_dir: &Path,
    plan: &PackagePlan,
) -> Result<Option<PathBuf>, XtaskError> {
    let arch = match plan.target.split('-').next() {
        Some("x86_64") => "x64",
        Some("aarch64") => "arm64",
        _ => {
            println!("msi: skipped (unsupported architecture in {})", plan.target);
            return Ok(None);
        }
    };
    let wxs_path = stage_dir.join("correomqtt.wxs");
    write_file(&wxs_path, msi_wxs().as_bytes())?;
    let artifact = plan
        .out_dir
        .join(plan.artifact_file_name().replace(".zip", ".msi"));
    if artifact.exists() {
        fs::remove_file(&artifact)?;
    }
    let app_dir = stage_dir.join(APP_NAME);
    let Some(status) = run_or_skip(
        Command::new("wix")
            .arg("build")
            .arg("-arch")
            .arg(arch)
            .arg("-d")
            .arg(format!("Version={}", env!("CARGO_PKG_VERSION")))
            .arg("-d")
            .arg(format!("AppDir={}", app_dir.display()))
            .arg("-o")
            .arg(&artifact)
            .arg(&wxs_path),
        "msi: skipped (wix is not installed)",
    )?
    else {
        return Ok(None);
    };
    if !status.success() {
        return Err(XtaskError::CommandFailed {
            command: "wix build".to_owned(),
            status,
        });
    }
    Ok(Some(artifact))
}

pub(super) fn msi_wxs() -> String {
    format!(
        r#"<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="{APP_NAME}" Manufacturer="{VENDOR}" Version="$(var.Version)" Language="1033" Scope="perMachine" UpgradeCode="7f2a8f4e-6f5a-4d67-9c11-4c1baf1c9c5e">
    <MajorUpgrade DowngradeErrorMessage="A newer version of {APP_NAME} is already installed." />
    <MediaTemplate EmbedCab="yes" />
    <Icon Id="AppIcon" SourceFile="$(var.AppDir)\icons\Icon.ico" />
    <Property Id="ARPPRODUCTICON" Value="AppIcon" />
    <StandardDirectory Id="ProgramFiles64Folder">
      <Directory Id="INSTALLFOLDER" Name="{APP_NAME}" />
    </StandardDirectory>
    <ComponentGroup Id="AppFiles" Directory="INSTALLFOLDER">
      <Files Include="$(var.AppDir)\**" />
    </ComponentGroup>
    <StandardDirectory Id="ProgramMenuFolder">
      <Component Id="StartMenuShortcut">
        <Shortcut Id="CorreoMqttShortcut" Name="{APP_NAME}" Target="[INSTALLFOLDER]{BIN_NAME}.exe" WorkingDirectory="INSTALLFOLDER" />
        <RegistryValue Root="HKCU" Key="Software\{APP_NAME}" Name="installed" Type="integer" Value="1" KeyPath="yes" />
        <RemoveFolder Id="RemoveProgramMenu" On="uninstall" />
      </Component>
    </StandardDirectory>
    <Feature Id="MainFeature" Title="{APP_NAME}" Level="1">
      <ComponentGroupRef Id="AppFiles" />
      <ComponentRef Id="StartMenuShortcut" />
    </Feature>
  </Package>
</Wix>
"#
    )
}

// Unsigned drag-and-drop DMG. Needs hdiutil, so it only runs on macOS hosts;
// cross-builds still produce the zip.
pub(super) fn create_dmg(
    stage_dir: &Path,
    plan: &PackagePlan,
) -> Result<Option<PathBuf>, XtaskError> {
    if !cfg!(target_os = "macos") {
        println!("dmg:     skipped (requires a macOS host with hdiutil)");
        return Ok(None);
    }
    let dmg = plan
        .out_dir
        .join(plan.artifact_file_name().replace(".zip", ".dmg"));
    if dmg.exists() {
        fs::remove_file(&dmg)?;
    }
    let status = Command::new("hdiutil")
        .arg("create")
        .arg("-volname")
        .arg(APP_NAME)
        .arg("-srcfolder")
        .arg(stage_dir)
        .arg("-ov")
        .arg("-format")
        .arg("UDZO")
        .arg(&dmg)
        .status()?;
    if !status.success() {
        return Err(XtaskError::CommandFailed {
            command: "hdiutil create".to_owned(),
            status,
        });
    }
    Ok(Some(dmg))
}
