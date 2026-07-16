
fn plugin_archive_file_name(manifest: &PluginManifest) -> Result<String, XtaskError> {
    Ok(format!(
        "{}-{}.zip",
        safe_plugin_id_component(&manifest.id)?,
        manifest.version
    ))
}

fn release_asset_url(base_url: &str, file_name: &str) -> String {
    format!("{}/{}", base_url.trim_end_matches('/'), file_name)
}

fn safe_plugin_id_component(value: &str) -> Result<&str, XtaskError> {
    let is_safe = !value.is_empty()
        && value != "."
        && value != ".."
        && value.chars().all(
            |character| matches!(character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_'),
        );
    if is_safe {
        Ok(value)
    } else {
        Err(XtaskError::InvalidArguments(format!(
            "plugin id `{value}` is not a safe single path component"
        )))
    }
}

fn print_help() {
    println!("Usage: cargo xtask plugin-repository [--out-dir <dir>]");
    println!();
    println!("Builds and stages local plugin packages plus local-repo.json.");
    println!("Default --out-dir is target/debug, next to the dev executable.");
}

fn print_release_help() {
    println!(
        "Usage: cargo xtask plugin-release [--out-dir <dir>] [--asset-base-url <url>] [--no-build]"
    );
    println!();
    println!("Builds plugin WASM archives plus default-repo.json for GitHub release assets.");
    println!("Default --out-dir is dist/plugins.");
}

fn print_release_smoke_help() {
    println!(
        "Usage: cargo xtask plugin-release-smoke --download-dir <dir> --profile-dir <dir> --asset-base-url <url>"
    );
    println!();
    println!("Verifies downloaded release ZIPs and installs them into a clean profile.");
}

#[derive(Debug)]
struct PluginRepositoryConfig {
    out_dir: PathBuf,
    show_help: bool,
}

#[derive(Debug)]
struct PluginReleaseConfig {
    out_dir: PathBuf,
    asset_base_url: String,
    build: bool,
    show_help: bool,
}

impl PluginReleaseConfig {
    fn from_args(args: Vec<String>) -> Result<Self, XtaskError> {
        let mut out_dir = PathBuf::from("dist/plugins");
        let mut asset_base_url = DEFAULT_RELEASE_ASSET_BASE_URL.to_owned();
        let mut build = true;
        let mut show_help = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--out-dir" => {
                    let value = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--out-dir requires a value".to_owned())
                    })?;
                    out_dir = PathBuf::from(value);
                }
                "--asset-base-url" => {
                    asset_base_url = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--asset-base-url requires a value".to_owned())
                    })?;
                    if asset_base_url.trim().is_empty() {
                        return Err(XtaskError::InvalidArguments(
                            "--asset-base-url cannot be empty".to_owned(),
                        ));
                    }
                }
                "--no-build" => build = false,
                "-h" | "--help" => show_help = true,
                unknown => {
                    return Err(XtaskError::InvalidArguments(format!(
                        "unknown plugin-release option: {unknown}"
                    )));
                }
            }
        }

        Ok(Self {
            out_dir,
            asset_base_url,
            build,
            show_help,
        })
    }
}

#[derive(Debug)]
struct PluginReleaseSmokeConfig {
    download_dir: PathBuf,
    profile_dir: PathBuf,
    asset_base_url: String,
    show_help: bool,
}

impl PluginReleaseSmokeConfig {
    fn from_args(args: Vec<String>) -> Result<Self, XtaskError> {
        let mut download_dir = None;
        let mut profile_dir = None;
        let mut asset_base_url = None;
        let mut show_help = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--download-dir" => {
                    download_dir = Some(PathBuf::from(iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--download-dir requires a value".to_owned())
                    })?))
                }
                "--profile-dir" => {
                    profile_dir = Some(PathBuf::from(iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--profile-dir requires a value".to_owned())
                    })?))
                }
                "--asset-base-url" => {
                    asset_base_url = Some(iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--asset-base-url requires a value".to_owned())
                    })?)
                }
                "-h" | "--help" => show_help = true,
                unknown => {
                    return Err(XtaskError::InvalidArguments(format!(
                        "unknown plugin-release-smoke option: {unknown}"
                    )));
                }
            }
        }

        if show_help {
            return Ok(Self {
                download_dir: PathBuf::new(),
                profile_dir: PathBuf::new(),
                asset_base_url: String::new(),
                show_help,
            });
        }

        Ok(Self {
            download_dir: download_dir.ok_or_else(|| {
                XtaskError::InvalidArguments("--download-dir is required".to_owned())
            })?,
            profile_dir: profile_dir.ok_or_else(|| {
                XtaskError::InvalidArguments("--profile-dir is required".to_owned())
            })?,
            asset_base_url: asset_base_url.ok_or_else(|| {
                XtaskError::InvalidArguments("--asset-base-url is required".to_owned())
            })?,
            show_help,
        })
    }
}

#[cfg(test)]
#[path = "plugin_repository/tests.rs"]
mod tests;

impl PluginRepositoryConfig {
    fn from_args(args: Vec<String>) -> Result<Self, XtaskError> {
        let mut out_dir = target_dir().join("debug");
        let mut show_help = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--out-dir" => {
                    let value = iter.next().ok_or_else(|| {
                        XtaskError::InvalidArguments("--out-dir requires a value".to_owned())
                    })?;
                    out_dir = PathBuf::from(value);
                }
                "-h" | "--help" => show_help = true,
                unknown => {
                    return Err(XtaskError::InvalidArguments(format!(
                        "unknown plugin-repository option: {unknown}"
                    )));
                }
            }
        }

        Ok(Self { out_dir, show_help })
    }
}
