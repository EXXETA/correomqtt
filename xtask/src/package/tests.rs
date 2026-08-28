use super::checksums::{hex_digest, sha256_file, write_checksum_files};
use super::installers::{create_linux_installers, msi_wxs, nfpm_config};
use super::*;
use sha2::{Digest, Sha256};

#[test]
fn detects_supported_platforms_from_targets() {
    assert_eq!(
        Platform::from_target("x86_64-pc-windows-msvc").unwrap(),
        Platform::Windows
    );
    assert_eq!(
        Platform::from_target("aarch64-apple-darwin").unwrap(),
        Platform::Macos
    );
    assert_eq!(
        Platform::from_target("x86_64-unknown-linux-gnu").unwrap(),
        Platform::Linux
    );
}

#[test]
fn package_names_are_predictable() {
    let plan = PackagePlan::new(
        "x86_64-unknown-linux-gnu".to_owned(),
        PathBuf::from("dist/beta"),
    );
    assert_eq!(
        plan.artifact_file_name(),
        format!(
            "CorreoMQTT-{}-beta-x86_64-unknown-linux-gnu.zip",
            env!("CARGO_PKG_VERSION")
        )
    );
}

#[test]
fn sha256_hex_is_lowercase() {
    let digest = Sha256::digest(b"abc");
    assert_eq!(
        hex_digest(&digest),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn zip_entries_have_stable_metadata_and_modes() {
    let root = temp_package_root("zip-metadata");
    let source = root.join("stage");
    write_file(&source.join("bin").join(BIN_NAME), b"binary").unwrap();
    write_file(&source.join("README.txt"), b"readme").unwrap();

    let first = root.join("first.zip");
    let second = root.join("second.zip");
    zip_dir(&source, &first).unwrap();
    zip_dir(&source, &second).unwrap();

    assert_eq!(sha256_file(&first).unwrap(), sha256_file(&second).unwrap());

    let file = std::fs::File::open(&first).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let file = archive.by_index(index).unwrap();
        entries.push((
            file.name().to_owned(),
            file.last_modified().unwrap(),
            file.unix_mode().unwrap() & 0o777,
        ));
    }
    let names = entries
        .iter()
        .map(|entry| entry.0.clone())
        .collect::<Vec<_>>();
    let mut sorted_names = names.clone();
    sorted_names.sort();

    assert_eq!(names, sorted_names);
    assert_eq!(
        entries,
        vec![
            ("stage/README.txt".to_owned(), DateTime::default(), 0o644),
            ("stage/bin/".to_owned(), DateTime::default(), 0o755),
            (
                "stage/bin/correomqtt".to_owned(),
                DateTime::default(),
                0o755
            ),
        ]
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_smoke_rejects_unexpected_zip_outputs() {
    let root = temp_package_root("package-guard");
    let out_dir = root.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let plan = PackagePlan::new("x86_64-unknown-linux-gnu".to_owned(), out_dir.clone());
    let artifact = plan.artifact_path();
    write_file(&artifact, b"package").unwrap();
    let checksum = write_checksum_files(&artifact, &out_dir).unwrap();
    write_sha256sums(&out_dir).unwrap();

    write_file(&out_dir.join("stale.zip"), b"stale").unwrap();
    let output = Some(PackageOutput {
        command: "cargo xtask package-smoke --target x86_64-unknown-linux-gnu".to_owned(),
        target: plan.target,
        out_dir,
        artifact,
        checksum,
    });

    let error = guard::verify(&output).unwrap_err().to_string();
    assert!(error.contains("unexpected ZIP outputs"));
    std::fs::remove_dir_all(root).unwrap();
}

fn temp_package_root(name: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("{name}-{}-{timestamp}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn nfpm_config_lists_all_desktop_assets() {
    let config = nfpm_config(Path::new("/stage"), "amd64");
    assert!(config.contains("arch: amd64"));
    assert!(config.contains(&format!("version: {}", env!("CARGO_PKG_VERSION"))));
    assert!(config.contains("dst: /usr/share/applications/org.correomqtt.CorreoMQTT.desktop"));
    assert!(config.contains("dst: /usr/share/metainfo/org.correomqtt.CorreoMQTT.metainfo.xml"));
}

#[test]
fn nfpm_config_bundles_plugins_next_to_binary() {
    let config = nfpm_config(Path::new("/stage"), "amd64");
    // The app loads its plugin repository next to the executable, so the DEB/RPM
    // must ship local-repo.json + plugins/ alongside the binary.
    assert!(config.contains("dst: /usr/lib/correomqtt/correomqtt"));
    assert!(config.contains("dst: /usr/lib/correomqtt/local-repo.json"));
    assert!(config.contains("dst: /usr/lib/correomqtt/plugins"));
    assert!(config.contains("type: tree"));
    // /usr/bin entry is a symlink to the real binary.
    assert!(config.contains("dst: /usr/bin/correomqtt"));
    assert!(config.contains("type: symlink"));
}

#[test]
fn msi_wxs_harvests_full_app_dir_with_stable_upgrade_code() {
    let wxs = msi_wxs();
    assert!(wxs.contains("UpgradeCode=\"7f2a8f4e-6f5a-4d67-9c11-4c1baf1c9c5e\""));
    assert!(wxs.contains("Scope=\"perMachine\""));
    // Harvest the whole staged app dir so plugins/ and local-repo.json ship.
    assert!(wxs.contains("<Files Include=\"$(var.AppDir)\\**\" />"));
    assert!(wxs.contains("Target=\"[INSTALLFOLDER]correomqtt.exe\""));
}

#[test]
fn linux_installers_build_with_nfpm_when_available() {
    let nfpm_available = Command::new("nfpm")
        .arg("--version")
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if !nfpm_available {
        eprintln!("skipping: nfpm is not installed");
        return;
    }

    let root = temp_package_root("correomqtt-nfpm-test");
    let plan = PackagePlan::new("x86_64-unknown-linux-gnu".to_owned(), root.clone());
    let stage_dir = plan.stage_dir();
    let app_root = stage_dir.join(APP_NAME);
    write_file(&app_root.join("bin").join(BIN_NAME), b"#!/bin/sh\n").unwrap();
    // The plugin repository + wasm plugins that stage_local_plugins produces
    // next to the binary and that the installer must ship.
    write_file(&app_root.join("bin/local-repo.json"), b"{}").unwrap();
    write_file(
        &app_root.join("bin/plugins/org.example.demo/plugin.wasm"),
        b"\0asm",
    )
    .unwrap();
    write_file(
        &app_root
            .join("share/applications")
            .join(format!("{APP_ID}.desktop")),
        linux_desktop_entry().as_bytes(),
    )
    .unwrap();
    write_file(
        &app_root
            .join("share/icons/hicolor/256x256/apps")
            .join(format!("{APP_ID}.png")),
        b"synthetic png",
    )
    .unwrap();
    write_file(
        &app_root
            .join("share/metainfo")
            .join(format!("{APP_ID}.metainfo.xml")),
        linux_metainfo().as_bytes(),
    )
    .unwrap();

    let installers = create_linux_installers(&stage_dir, &plan).unwrap();
    assert_eq!(installers.len(), 2, "expected deb and rpm artifacts");
    for artifact in &installers {
        assert!(artifact.exists(), "missing {}", artifact.display());
    }

    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn sha256sums_covers_installer_artifacts_not_just_zip() {
    let root = temp_package_root("package-sums");
    let out_dir = root.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let plan = PackagePlan::new("x86_64-unknown-linux-gnu".to_owned(), out_dir.clone());
    let zip = plan.artifact_path();
    write_file(&zip, b"zip payload").unwrap();
    write_checksum_files(&zip, &out_dir).unwrap();

    // A DEB produced after the zip must still land in the summary.
    let deb = out_dir.join(plan.artifact_file_name().replace(".zip", ".deb"));
    write_file(&deb, b"deb payload").unwrap();

    write_sha256sums(&out_dir).unwrap();

    let summary = std::fs::read_to_string(out_dir.join("SHA256SUMS")).unwrap();
    let zip_name = zip.file_name().unwrap().to_string_lossy();
    let deb_name = deb.file_name().unwrap().to_string_lossy();
    assert!(summary.contains(&*zip_name), "zip missing from SHA256SUMS");
    assert!(
        summary.contains(&*deb_name),
        "installer artifact missing from SHA256SUMS: {summary}"
    );
    std::fs::remove_dir_all(root).unwrap();
}
