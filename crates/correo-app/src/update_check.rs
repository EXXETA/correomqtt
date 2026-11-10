use std::time::Duration;

// Same endpoint and env override the Java app uses.
const GITHUB_API_LATEST: &str = "https://api.github.com/repos/exxeta/correomqtt/releases/latest";
const ENV_OVERRIDE: &str = "CORREO_GITHUB_API_LATEST_URL";

pub fn check_latest_release() -> (String, bool) {
    // Treat an empty override as unset so a blank env var does not point the
    // update check at "".
    let url = std::env::var(ENV_OVERRIDE)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| GITHUB_API_LATEST.to_owned());
    match fetch_latest_tag(&url) {
        Ok(tag) => summarize(env!("CARGO_PKG_VERSION"), &tag),
        Err(error) => (format!("Last check failed: {error}"), false),
    }
}

fn fetch_latest_tag(url: &str) -> Result<String, String> {
    let response = ureq::get(url)
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|error| error.to_string())?;
    let body = response.into_string().map_err(|error| error.to_string())?;
    let release: serde_json::Value =
        serde_json::from_str(&body).map_err(|error| error.to_string())?;
    release
        .get("tag_name")
        .and_then(|tag| tag.as_str())
        .map(str::to_owned)
        .ok_or_else(|| "release response had no tag_name".to_owned())
}

fn summarize(current: &str, tag: &str) -> (String, bool) {
    let latest = tag.trim_start_matches('v');
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(current_version), Ok(latest_version)) if latest_version > current_version => (
            format!("Update available: {tag} (current v{current})"),
            true,
        ),
        (Ok(_), Ok(_)) => (format!("Up to date (v{current})"), false),
        _ => (format!("Latest release: {tag} (current v{current})"), false),
    }
}

#[cfg(test)]
mod tests {
    use super::summarize;

    #[test]
    fn summarize_compares_versions() {
        let (message, available) = summarize("0.1.0", "v1.2.3");
        assert!(available);
        assert!(message.contains("v1.2.3"));

        let (message, available) = summarize("1.2.3", "v1.2.3");
        assert!(!available);
        assert!(message.contains("Up to date"));

        let (_, available) = summarize("1.2.3", "not-a-version");
        assert!(!available);
    }
}
