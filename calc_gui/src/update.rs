//! Update checker for native builds.
//!
//! Queries GitHub Releases API to check for newer versions.
//! Only compiled for native targets (not WASM).

use semver::Version;
use serde::Deserialize;

/// Current application version (from Cargo.toml)
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository for release checks
const GITHUB_OWNER: &str = "Hotschmoe";
const GITHUB_REPO: &str = "stratify";

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
    pub html_url: String,
}

/// Result of an update check
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// A newer version is available
    UpdateAvailable(UpdateInfo),
    /// Already running the latest version
    UpToDate,
    /// Check failed (with error message)
    Failed(String),
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Check GitHub for updates asynchronously
pub async fn check_for_updates() -> UpdateCheckResult {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    let client = match reqwest::Client::builder()
        .user_agent(format!("Stratify/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return UpdateCheckResult::Failed(format!("Failed to create HTTP client: {}", e)),
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => return UpdateCheckResult::Failed(format!("Network error: {}", e)),
    };

    if !response.status().is_success() {
        return UpdateCheckResult::Failed(format!("GitHub API returned {}", response.status()));
    }

    let release: GitHubRelease = match response.json().await {
        Ok(r) => r,
        Err(e) => return UpdateCheckResult::Failed(format!("Failed to parse response: {}", e)),
    };

    // Parse versions (strip 'v' prefix if present)
    let remote_version_str = release.tag_name.trim_start_matches('v');
    let remote_version = match Version::parse(remote_version_str) {
        Ok(v) => v,
        Err(e) => {
            return UpdateCheckResult::Failed(format!(
                "Invalid remote version '{}': {}",
                release.tag_name, e
            ))
        }
    };

    let current_version = match Version::parse(CURRENT_VERSION) {
        Ok(v) => v,
        Err(e) => {
            return UpdateCheckResult::Failed(format!(
                "Invalid current version '{}': {}",
                CURRENT_VERSION, e
            ))
        }
    };

    if remote_version > current_version {
        // Find the appropriate download for this platform
        let download_url = find_platform_asset(&release.assets)
            .map(|a| a.browser_download_url.clone())
            .unwrap_or_else(|| release.html_url.clone());

        UpdateCheckResult::UpdateAvailable(UpdateInfo {
            version: remote_version.to_string(),
            download_url,
            release_notes: release.body.unwrap_or_default(),
            html_url: release.html_url,
        })
    } else {
        UpdateCheckResult::UpToDate
    }
}

/// Find the download asset for the current platform
fn find_platform_asset(assets: &[GitHubAsset]) -> Option<&GitHubAsset> {
    let target = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        }
    } else {
        "linux"
    };

    assets.iter().find(|a| {
        let name = a.name.to_lowercase();
        name.contains(target) || (target == "windows" && name.ends_with(".exe"))
    })
}

/// Open a URL in the default browser
pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
