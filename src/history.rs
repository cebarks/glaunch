use std::collections::BTreeMap;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::{self, CliOverrides};

const MAX_LAUNCHES_PER_APP: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRecord {
    pub timestamp: DateTime<Utc>,
    pub command: Vec<String>,
    pub overrides: CliOverrides,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppHistory {
    pub steam_app_id: Option<u64>,
    pub launches: Vec<LaunchRecord>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct History(BTreeMap<String, AppHistory>);

impl Deref for History {
    type Target = BTreeMap<String, AppHistory>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for History {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn history_path() -> PathBuf {
    config::config_dir().join("history.json")
}

pub fn load_history() -> Result<History> {
    let path = history_path();
    if !path.exists() {
        return Ok(History::default());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn save_history(history: &History) -> Result<()> {
    let path = history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(history).context("failed to serialize history")?;
    fs::write(&path, contents).with_context(|| format!("failed to write {}", path.display()))
}

pub fn record_launch(
    history: &mut History,
    slug: &str,
    steam_app_id: Option<u64>,
    command: Vec<String>,
    overrides: CliOverrides,
) {
    let record = LaunchRecord {
        timestamp: Utc::now(),
        command,
        overrides,
    };

    let app = history
        .entry(slug.to_string())
        .or_insert_with(|| AppHistory {
            steam_app_id,
            launches: Vec::new(),
        });

    if let Some(id) = steam_app_id {
        app.steam_app_id = Some(id);
    }

    app.launches.insert(0, record);
    app.launches.truncate(MAX_LAUNCHES_PER_APP);
}

pub fn derive_app_slug(command: &[String]) -> String {
    for arg in command {
        if let Some(slug) = extract_slug_from_steamapps_path(arg) {
            return slug;
        }
    }

    // Fallback: use executable basename
    if let Some(first) = command.first() {
        let path = std::path::Path::new(first);
        if let Some(stem) = path.file_stem() {
            let slug = normalize_to_slug(&stem.to_string_lossy());
            if !slug.is_empty() {
                return slug;
            }
        }
    }

    "unknown".to_string()
}

fn extract_slug_from_steamapps_path(arg: &str) -> Option<String> {
    let lower = arg.to_lowercase();
    let marker = "steamapps/common/";
    let idx = lower.find(marker)?;
    let after = &arg[idx + marker.len()..];
    let folder = after.split('/').next().unwrap_or("");
    if folder.is_empty() {
        return None;
    }
    let slug = normalize_to_slug(folder);
    if slug.is_empty() { None } else { Some(slug) }
}

fn normalize_to_slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c == ' ' || c == '_' || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn has_overrides(cli: &CliOverrides) -> bool {
    cli.width.is_some()
        || cli.height.is_some()
        || cli.refresh_rate.is_some()
        || cli.hdr.is_some()
        || cli.vrr.is_some()
        || cli.gamescope.is_some()
        || cli.itm.is_some()
        || cli.fsr4.is_some()
        || cli.vcache.is_some()
        || cli.fix_mouse.is_some()
        || cli.mangohud.is_some()
        || cli.mangohud_config.is_some()
        || cli.vkbasalt.is_some()
        || cli.vkbasalt_profile.is_some()
}
