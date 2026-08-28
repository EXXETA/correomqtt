use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;

use zip::{write::SimpleFileOptions, DateTime};

use crate::{cargo_dynamic, XtaskError};

pub(crate) mod checksums;
mod guard;
mod plugins;

use self::checksums::write_checksum_files;

const APP_NAME: &str = "CorreoMQTT";
const APP_ID: &str = "org.correomqtt.CorreoMQTT";
const BIN_NAME: &str = "correomqtt";
const VENDOR: &str = "EXXETA AG";
const PACKAGE_ASSET_DIR: &str = "assets/package";

pub(crate) fn run(args: Vec<String>) -> Result<(), XtaskError> {
    package("cargo xtask package", args).map(|_| ())
}

pub(crate) fn smoke(args: Vec<String>) -> Result<(), XtaskError> {
    let output = package("cargo xtask package-smoke", args)?;
    guard::verify(&output)
}

fn package(command_base: &str, args: Vec<String>) -> Result<Option<PackageOutput>, XtaskError> {
    let command = command_display(command_base, &args);
    let config = PackageConfig::from_args(args)?;
    if config.show_help {
        print_package_help();
        return Ok(None);
    }

    let target = match &config.target {
        Some(target) => target.clone(),
        None => host_triple()?,
    };
    let platform = Platform::from_target(&target)?;

    if config.build {
        build_app(config.target.as_deref())?;
        crate::plugin_repository::build_wasm_plugins()?;
    }

    let binary = release_binary_path(config.target.as_deref(), platform);
    if !binary.exists() {
        return Err(XtaskError::MissingArtifact(binary.display().to_string()));
    }

    let plan = PackagePlan::new(target, config.out_dir);
    let stage_dir = plan.stage_dir();
    if stage_dir.exists() {
        fs::remove_dir_all(&stage_dir)?;
    }
    fs::create_dir_all(&stage_dir)?;

    match platform {
        Platform::Linux => stage_linux(&binary, &stage_dir)?,
        Platform::Macos => stage_macos(&binary, &stage_dir)?,
        Platform::Windows => stage_windows(&binary, &stage_dir)?,
    }
    let source_executable_dir = binary
        .parent()
        .ok_or_else(|| XtaskError::MissingArtifact(binary.display().to_string()))?;
    plugins::stage(platform, &stage_dir, source_executable_dir)?;

    fs::create_dir_all(&plan.out_dir)?;
    let artifact = plan.artifact_path();
    if artifact.exists() {
        fs::remove_file(&artifact)?;
    }
    zip_dir(&stage_dir, &artifact)?;
    let checksum = write_checksum_files(&artifact, &plan.out_dir)?;

    println!("package: {}", artifact.display());
    println!("sha256:  {checksum}");
    Ok(Some(PackageOutput {
        command,
        target: plan.target,
        out_dir: plan.out_dir,
        artifact,
        checksum,
    }))
}

fn build_app(target: Option<&str>) -> Result<(), XtaskError> {
    let mut args = vec![
        "build".to_owned(),
        "--release".to_owned(),
        "-p".to_owned(),
        "correo-app".to_owned(),
        "--bin".to_owned(),
        BIN_NAME.to_owned(),
    ];
    if let Some(target) = target {
        args.push("--target".to_owned());
        args.push(target.to_owned());
    }
    cargo_dynamic(&args)
}

fn stage_linux(binary: &Path, stage_dir: &Path) -> Result<(), XtaskError> {
    let root = stage_dir.join(APP_NAME);
    copy_file(binary, &root.join("bin").join(BIN_NAME))?;
    copy_file(
        Path::new(PACKAGE_ASSET_DIR).join("Icon.png").as_path(),
        &root
            .join("share/icons/hicolor/256x256/apps")
            .join(format!("{APP_ID}.png")),
    )?;
    write_file(
        &root
            .join("share/applications")
            .join(format!("{APP_ID}.desktop")),
        linux_desktop_entry().as_bytes(),
    )?;
    write_file(
        &root
            .join("share/metainfo")
            .join(format!("{APP_ID}.metainfo.xml")),
        linux_metainfo().as_bytes(),
    )?;
    write_file(&root.join("README.txt"), package_readme().as_bytes())
}

fn stage_macos(binary: &Path, stage_dir: &Path) -> Result<(), XtaskError> {
    let contents = stage_dir.join(format!("{APP_NAME}.app")).join("Contents");
    copy_file(binary, &contents.join("MacOS").join(BIN_NAME))?;
    copy_file(
        Path::new(PACKAGE_ASSET_DIR).join("Icon.icns").as_path(),
        &contents.join("Resources/Icon.icns"),
    )?;
    write_file(&contents.join("Info.plist"), macos_info_plist().as_bytes())?;
    write_file(&contents.join("PkgInfo"), b"APPL????")?;
    write_file(&stage_dir.join("README.txt"), package_readme().as_bytes())
}

fn stage_windows(binary: &Path, stage_dir: &Path) -> Result<(), XtaskError> {
    let root = stage_dir.join(APP_NAME);
    copy_file(binary, &root.join(format!("{BIN_NAME}.exe")))?;
    copy_file(
        Path::new(PACKAGE_ASSET_DIR).join("Icon.ico").as_path(),
        &root.join("icons/Icon.ico"),
    )?;
    write_file(
        &root.join("metadata/app.json"),
        windows_metadata().as_bytes(),
    )?;
    write_file(&root.join("README.txt"), package_readme().as_bytes())
}

fn release_binary_path(target: Option<&str>, platform: Platform) -> PathBuf {
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"));
    let profile_dir = match target {
        Some(target) => target_dir.join(target).join("release"),
        None => target_dir.join("release"),
    };
    profile_dir.join(platform.binary_name())
}

fn host_triple() -> Result<String, XtaskError> {
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc).arg("-vV").output()?;
    if !output.status.success() {
        return Err(XtaskError::InvalidArguments(format!(
            "rustc -vV exited with {}",
            output.status
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(host) = line.strip_prefix("host: ") {
            return Ok(host.to_owned());
        }
    }
    Err(XtaskError::InvalidArguments(
        "rustc -vV did not report a host triple".to_owned(),
    ))
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), XtaskError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn write_file(path: &Path, content: &[u8]) -> Result<(), XtaskError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn zip_dir(source_dir: &Path, destination: &Path) -> Result<(), XtaskError> {
    let file = File::create(destination)?;
    let mut writer = zip::ZipWriter::new(BufWriter::new(file));
    let root = source_dir
        .parent()
        .ok_or_else(|| XtaskError::InvalidArguments("stage directory has no parent".to_owned()))?;
    add_zip_entries(root, source_dir, &mut writer)?;
    writer.finish()?;
    Ok(())
}

pub(crate) fn zip_dir_contents(source_dir: &Path, destination: &Path) -> Result<(), XtaskError> {
    let file = File::create(destination)?;
    let mut writer = zip::ZipWriter::new(BufWriter::new(file));
    add_zip_entries(source_dir, source_dir, &mut writer)?;
    writer.finish()?;
    Ok(())
}

fn add_zip_entries(
    root: &Path,
    path: &Path,
    writer: &mut zip::ZipWriter<BufWriter<File>>,
) -> Result<(), XtaskError> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| XtaskError::InvalidArguments(error.to_string()))?;
        let archive_name = zip_path(relative);
        if path.is_dir() {
            writer.add_directory(format!("{archive_name}/"), zip_options(0o755))?;
            add_zip_entries(root, &path, writer)?;
        } else {
            let mode = zip_mode(&path);
            writer.start_file(archive_name, zip_options(mode))?;
            let mut file = BufReader::new(File::open(&path)?);
            io::copy(&mut file, writer)?;
        }
    }
    Ok(())
}

fn zip_options(mode: u32) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(DateTime::default())
        .unix_permissions(mode)
}

fn zip_mode(path: &Path) -> u32 {
    let file_name = path.file_name().and_then(OsStr::to_str);
    let executable = path
        .components()
        .any(|component| component.as_os_str() == OsStr::new("MacOS"))
        || file_name == Some(BIN_NAME)
        || file_name == Some("correomqtt.exe");
    if executable {
        0o755
    } else {
        0o644
    }
}

fn zip_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn linux_desktop_entry() -> String {
    format!(
        "[Desktop Entry]\n\
         Name={APP_NAME}\n\
         Comment=Native MQTT desktop client\n\
         Exec={BIN_NAME}\n\
         Icon={APP_ID}\n\
         StartupWMClass={APP_ID}\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Development;Network;\n"
    )
}

fn linux_metainfo() -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <component type=\"desktop-application\">\n\
           <id>{APP_ID}</id>\n\
           <name>{APP_NAME}</name>\n\
           <summary>Native MQTT desktop client</summary>\n\
           <metadata_license>CC0-1.0</metadata_license>\n\
           <project_license>GPL-3.0-or-later</project_license>\n\
         </component>\n"
    )
}

fn macos_info_plist() -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
           <key>CFBundleDisplayName</key><string>{APP_NAME}</string>\n\
           <key>CFBundleExecutable</key><string>{BIN_NAME}</string>\n\
           <key>CFBundleIconFile</key><string>Icon.icns</string>\n\
           <key>CFBundleIdentifier</key><string>{APP_ID}</string>\n\
           <key>CFBundleName</key><string>{APP_NAME}</string>\n\
           <key>CFBundlePackageType</key><string>APPL</string>\n\
           <key>CFBundleShortVersionString</key><string>{}</string>\n\
           <key>CFBundleVersion</key><string>{}</string>\n\
           <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>\n\
         </dict>\n\
         </plist>\n",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION")
    )
}

fn windows_metadata() -> String {
    format!(
        "{{\n  \"name\": \"{APP_NAME}\",\n  \"identifier\": \"{APP_ID}\",\n  \
         \"version\": \"{}\",\n  \"vendor\": \"{VENDOR}\",\n  \"binary\": \
         \"{BIN_NAME}.exe\",\n  \"icon\": \"icons/Icon.ico\",\n  \"signed\": false\n}}\n",
        env!("CARGO_PKG_VERSION")
    )
}

fn package_readme() -> String {
    format!(
        "{APP_NAME} unsigned beta package\n\n\
         Version: {}\n\
         Vendor: {VENDOR}\n\
         App ID: {APP_ID}\n\n\
         This package is intentionally unsigned. Signing, notarization, \
         auto-update, paid services, and external release commitments are \
         outside this automation scope.\n\n\
         Runtime data:\n\
         Set CORREOMQTT_CONFIG_DIR to use a specific config/history/log root.\n\
         Without it, the Rust beta uses the OS project data directory for \
         org/CorreoMQTT/CorreoMQTT and also checks legacy Java roots during startup.\n\
         Current config and histories live under that root. Script execution \
         metadata/logs live under scripts/executions/ and scripts/logs/ when \
         scripting persistence writes them. Rust plugin packages and \
         local-repo.json are included next to the executable. \
         App diagnostics currently go to stdout/stderr.\n",
        env!("CARGO_PKG_VERSION")
    )
}

fn print_package_help() {
    println!("Usage: cargo xtask package [--target <triple>] [--out-dir <dir>] [--no-build]");
    println!();
    println!("Builds correo-app release binary and writes an unsigned beta archive.");
    println!("Use `cargo xtask package-smoke` to build and validate artifact guardrails.");
}

fn command_display(base: &str, args: &[String]) -> String {
    if args.is_empty() {
        base.to_owned()
    } else {
        format!("{base} {}", args.join(" "))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Platform {
    Linux,
    Macos,
    Windows,
}

impl Platform {
    fn from_target(target: &str) -> Result<Self, XtaskError> {
        if target.contains("windows") {
            Ok(Self::Windows)
        } else if target.contains("apple-darwin") {
            Ok(Self::Macos)
        } else if target.contains("linux") {
            Ok(Self::Linux)
        } else {
            Err(XtaskError::UnsupportedTarget(target.to_owned()))
        }
    }

    fn binary_name(self) -> &'static str {
        match self {
            Self::Windows => "correomqtt.exe",
            Self::Linux | Self::Macos => BIN_NAME,
        }
    }
}

#[derive(Debug)]
struct PackageConfig {
    target: Option<String>,
    out_dir: PathBuf,
    build: bool,
    show_help: bool,
}

impl PackageConfig {
    fn from_args(args: Vec<String>) -> Result<Self, XtaskError> {
        let mut target = None;
        let mut out_dir = PathBuf::from("dist/beta");
        let mut build = true;
        let mut show_help = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--target" => {
                    let value = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--target requires a value".to_owned())
                    })?;
                    target = Some(value);
                }
                "--out-dir" => {
                    let value = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--out-dir requires a value".to_owned())
                    })?;
                    out_dir = PathBuf::from(value);
                }
                "--no-build" => build = false,
                "-h" | "--help" => show_help = true,
                unknown => {
                    return Err(XtaskError::InvalidArguments(format!(
                        "unknown package option: {unknown}"
                    )));
                }
            }
        }

        Ok(Self {
            target,
            out_dir,
            build,
            show_help,
        })
    }
}

#[derive(Debug)]
struct PackagePlan {
    target: String,
    out_dir: PathBuf,
}

impl PackagePlan {
    fn new(target: String, out_dir: PathBuf) -> Self {
        Self { target, out_dir }
    }

    fn artifact_file_name(&self) -> String {
        format!(
            "{APP_NAME}-{}-beta-{}.zip",
            env!("CARGO_PKG_VERSION"),
            self.target
        )
    }

    fn artifact_path(&self) -> PathBuf {
        self.out_dir.join(self.artifact_file_name())
    }

    fn stage_dir(&self) -> PathBuf {
        self.out_dir.join("stage").join(format!(
            "{APP_NAME}-{}-beta-{}",
            env!("CARGO_PKG_VERSION"),
            self.target
        ))
    }
}

#[derive(Debug)]
struct PackageOutput {
    command: String,
    target: String,
    out_dir: PathBuf,
    artifact: PathBuf,
    checksum: String,
}

#[cfg(test)]
mod tests;
