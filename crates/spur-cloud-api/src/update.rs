//! Auto-update check for spur-cloud-api.
//!
//! On startup, queries the GitHub releases API for ROCm/spur-cloud
//! to see if a newer version is available and logs an info message.
//! Results are cached to disk (1h TTL) to avoid API spam.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

const REPO: &str = "ROCm/spur-cloud";
const CACHE_FILENAME: &str = "update-check.json";
const CACHE_TTL_HOURS: i64 = 1;

// ── Config ──

/// Update check configuration (parsed from [update] in spur-cloud.toml).
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateConfig {
    /// Check for updates on startup (default: true).
    #[serde(default = "default_true")]
    pub check_on_startup: bool,

    /// Release channel: "stable" or "nightly" (default: "stable").
    #[serde(default = "default_stable")]
    pub channel: String,

    /// Cache directory for version check results.
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

fn default_true() -> bool {
    true
}
fn default_stable() -> String {
    "stable".into()
}
fn default_cache_dir() -> String {
    "/var/cache/spur-cloud".into()
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            channel: "stable".into(),
            cache_dir: "/var/cache/spur-cloud".into(),
        }
    }
}

// ── Cache ──

#[derive(Debug, Serialize, Deserialize)]
struct UpdateCache {
    checked_at: DateTime<Utc>,
    current_version: String,
    latest_tag: String,
    update_available: bool,
}

fn read_cache(cache_dir: &Path) -> Option<UpdateCache> {
    let path = cache_dir.join(CACHE_FILENAME);
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(cache_dir: &Path, cache: &UpdateCache) {
    let _ = std::fs::create_dir_all(cache_dir);
    let path = cache_dir.join(CACHE_FILENAME);
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = std::fs::write(&path, json);
    }
}

fn is_cache_fresh(cache: &UpdateCache) -> bool {
    let age = Utc::now() - cache.checked_at;
    age < Duration::hours(CACHE_TTL_HOURS)
}

// ── GitHub API ──

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Check result returned by `check_for_update`.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_tag: String,
    pub update_available: bool,
}

/// Query the GitHub releases API for the latest version.
pub async fn check_for_update(current_version: &str) -> anyhow::Result<UpdateCheckResult> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");

    let client = reqwest::Client::builder()
        .user_agent(format!("spur-cloud/{current_version}"))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let release: GitHubRelease = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let tag = &release.tag_name;
    let version_str = tag.strip_prefix('v').unwrap_or(tag);

    let update_available = match (
        semver::Version::parse(current_version),
        semver::Version::parse(version_str),
    ) {
        (Ok(current), Ok(latest)) => latest > current,
        _ => false,
    };

    Ok(UpdateCheckResult {
        current_version: current_version.to_string(),
        latest_tag: tag.clone(),
        update_available,
    })
}

// ── Startup check ──

/// Spawn a non-blocking background update check.
/// Call from main() after tracing is initialized.
pub fn spawn_startup_check(current_version: &'static str, config: &UpdateConfig) {
    if !config.check_on_startup {
        return;
    }

    let cache_dir = PathBuf::from(&config.cache_dir);

    tokio::spawn(async move {
        // Check cache first
        if let Some(cached) = read_cache(&cache_dir) {
            if is_cache_fresh(&cached) {
                if cached.update_available {
                    info!(
                        current = %cached.current_version,
                        latest = %cached.latest_tag,
                        "Update available for spur-cloud. See https://github.com/{REPO}/releases"
                    );
                }
                return;
            }
        }

        match check_for_update(current_version).await {
            Ok(result) => {
                write_cache(
                    &cache_dir,
                    &UpdateCache {
                        checked_at: Utc::now(),
                        current_version: result.current_version.clone(),
                        latest_tag: result.latest_tag.clone(),
                        update_available: result.update_available,
                    },
                );

                if result.update_available {
                    info!(
                        current = %result.current_version,
                        latest = %result.latest_tag,
                        "Update available for spur-cloud. See https://github.com/{REPO}/releases"
                    );
                } else {
                    debug!("spur-cloud is up to date ({})", current_version);
                }
            }
            Err(e) => {
                debug!("update check failed (non-fatal): {e}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_freshness() {
        let fresh = UpdateCache {
            checked_at: Utc::now(),
            current_version: "0.1.0".into(),
            latest_tag: "v0.1.3".into(),
            update_available: true,
        };
        assert!(is_cache_fresh(&fresh));

        let stale = UpdateCache {
            checked_at: Utc::now() - Duration::hours(2),
            current_version: "0.1.0".into(),
            latest_tag: "v0.1.3".into(),
            update_available: true,
        };
        assert!(!is_cache_fresh(&stale));
    }

    #[test]
    fn update_config_defaults() {
        let config = UpdateConfig::default();
        assert!(config.check_on_startup);
        assert_eq!(config.channel, "stable");
    }
}
