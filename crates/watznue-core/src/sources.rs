use crate::cache::DiskCache;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BodhiBug {
    pub bug_id: serde_json::Value,
    pub title: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BodhiUpdate {
    pub alias: Option<String>,
    pub updateid: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub update_type: Option<String>,
    pub severity: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub bugs: Vec<BodhiBug>,
}

#[derive(Deserialize)]
struct BodhiResponse {
    pub updates: Vec<BodhiUpdate>,
}

pub struct SourceAggregator {
    client: reqwest::Client,
    cache: DiskCache<BodhiUpdate>,
}

impl Default for SourceAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceAggregator {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(3000))
            .build()
            .unwrap_or_default();

        let cache = DiskCache::new("bodhi_cache.json");

        Self { client, cache }
    }

    pub fn get_source_package_name(pkg_name: &str) -> &str {
        if pkg_name.starts_with("mesa-") {
            "mesa"
        } else if pkg_name.starts_with("kernel-") {
            "kernel"
        } else if pkg_name.starts_with("systemd-") {
            "systemd"
        } else {
            pkg_name
        }
    }

    pub async fn fetch_bodhi_update(&mut self, package_name: &str) -> Option<BodhiUpdate> {
        let lookup_name = Self::get_source_package_name(package_name);

        if let Some(cached) = self.cache.get(lookup_name) {
            return cached;
        }

        let url = format!(
            "https://bodhi.fedoraproject.org/updates/?packages={}&rows_per_page=1",
            lookup_name
        );

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.json::<BodhiResponse>().await {
                    let update = body.updates.into_iter().next();
                    self.cache.insert(lookup_name, update.clone());
                    return update;
                }
            }
            _ => {}
        }

        // Cache negative result to avoid repeated network timeouts
        self.cache.insert(lookup_name, None);
        None
    }

    pub fn fetch_rpm_changelog(&self, package_name: &str, max_lines: usize) -> Option<String> {
        let output = Command::new("rpm")
            .args(["-q", "--changelog", package_name])
            .output()
            .ok()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = text.lines().take(max_lines).collect();
            if !lines.is_empty() {
                return Some(lines.join("\n"));
            }
        }
        None
    }
}
