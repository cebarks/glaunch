use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("could not determine XDG config directory")
        .join("glaunch")
}

pub fn profiles_dir() -> PathBuf {
    config_dir().join("profiles")
}

// --- TOML structs (all Option for layered resolution) ---

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub defaults: Option<DefaultsConfig>,
    pub mangohud: Option<MangoHudConfig>,
    pub hardware: Option<HardwareConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub hdr: Option<bool>,
    pub vrr: Option<bool>,
    pub gamescope: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MangoHudConfig {
    pub enabled: Option<bool>,
    pub config: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub vcache_cpus: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: Option<String>,
    pub steam_app_id: Option<u64>,
    pub settings: Option<ProfileSettings>,
    pub mangohud: Option<MangoHudConfig>,
    pub vkbasalt: Option<VkBasaltConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub hdr: Option<bool>,
    pub vrr: Option<bool>,
    pub gamescope: Option<bool>,
    pub itm: Option<bool>,
    pub fsr4: Option<bool>,
    pub vcache: Option<bool>,
    pub fix_mouse: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VkBasaltConfig {
    pub enabled: Option<bool>,
    pub profile: Option<String>,
}

// --- CLI override layer (constructed from RunArgs) ---

#[derive(Debug, Default)]
pub struct CliOverrides {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub hdr: Option<bool>,
    pub vrr: Option<bool>,
    pub gamescope: Option<bool>,
    pub itm: Option<bool>,
    pub fsr4: Option<bool>,
    pub vcache: Option<bool>,
    pub fix_mouse: Option<bool>,
    pub mangohud: Option<bool>,
    pub mangohud_config: Option<String>,
    pub vkbasalt: Option<bool>,
    pub vkbasalt_profile: Option<String>,
}

// --- Fully resolved settings (no Options) ---

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSettings {
    pub width: u32,
    pub height: u32,
    pub hdr: bool,
    pub vrr: bool,
    pub gamescope: bool,
    pub itm: bool,
    pub fsr4: bool,
    pub vcache: bool,
    pub fix_mouse: bool,
    pub mangohud: bool,
    pub mangohud_config: Option<String>,
    pub vkbasalt: bool,
    pub vkbasalt_profile: Option<String>,
}

/// Resolve settings with layered precedence: CLI > profile > global > hardcoded
pub fn resolve_settings_from_layers(
    cli: Option<&CliOverrides>,
    profile: Option<&Profile>,
    global: &GlobalConfig,
) -> ResolvedSettings {
    let defaults = global.defaults.as_ref();
    let prof_settings = profile.and_then(|p| p.settings.as_ref());
    let prof_mangohud = profile.and_then(|p| p.mangohud.as_ref());
    let prof_vkbasalt = profile.and_then(|p| p.vkbasalt.as_ref());
    let global_mangohud = global.mangohud.as_ref();

    ResolvedSettings {
        width: cli
            .and_then(|c| c.width)
            .or_else(|| prof_settings.and_then(|s| s.width))
            .or_else(|| defaults.and_then(|d| d.width))
            .unwrap_or(3840),
        height: cli
            .and_then(|c| c.height)
            .or_else(|| prof_settings.and_then(|s| s.height))
            .or_else(|| defaults.and_then(|d| d.height))
            .unwrap_or(2160),
        hdr: cli
            .and_then(|c| c.hdr)
            .or_else(|| prof_settings.and_then(|s| s.hdr))
            .or_else(|| defaults.and_then(|d| d.hdr))
            .unwrap_or(true),
        vrr: cli
            .and_then(|c| c.vrr)
            .or_else(|| prof_settings.and_then(|s| s.vrr))
            .or_else(|| defaults.and_then(|d| d.vrr))
            .unwrap_or(true),
        gamescope: cli
            .and_then(|c| c.gamescope)
            .or_else(|| prof_settings.and_then(|s| s.gamescope))
            .or_else(|| defaults.and_then(|d| d.gamescope))
            .unwrap_or(true),
        itm: cli
            .and_then(|c| c.itm)
            .or_else(|| prof_settings.and_then(|s| s.itm))
            .unwrap_or(false),
        fsr4: cli
            .and_then(|c| c.fsr4)
            .or_else(|| prof_settings.and_then(|s| s.fsr4))
            .unwrap_or(false),
        vcache: cli
            .and_then(|c| c.vcache)
            .or_else(|| prof_settings.and_then(|s| s.vcache))
            .unwrap_or(false),
        fix_mouse: cli
            .and_then(|c| c.fix_mouse)
            .or_else(|| prof_settings.and_then(|s| s.fix_mouse))
            .unwrap_or(false),
        mangohud: cli
            .and_then(|c| c.mangohud)
            .or_else(|| prof_mangohud.and_then(|m| m.enabled))
            .or_else(|| global_mangohud.and_then(|m| m.enabled))
            .unwrap_or(false),
        mangohud_config: cli
            .and_then(|c| c.mangohud_config.clone())
            .or_else(|| prof_mangohud.and_then(|m| m.config.clone()))
            .or_else(|| global_mangohud.and_then(|m| m.config.clone())),
        vkbasalt: cli
            .and_then(|c| c.vkbasalt)
            .or_else(|| prof_vkbasalt.and_then(|v| v.enabled))
            .unwrap_or(false),
        vkbasalt_profile: cli
            .and_then(|c| c.vkbasalt_profile.clone())
            .or_else(|| prof_vkbasalt.and_then(|v| v.profile.clone())),
    }
}

// --- File I/O ---

pub fn load_global_config() -> Result<GlobalConfig> {
    let path = config_dir().join("config.toml");
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn load_profile(name: &str) -> Result<Option<Profile>> {
    let path = profiles_dir().join(format!("{name}.toml"));
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read profile {}", path.display()))?;
    let profile: Profile = toml::from_str(&contents)
        .with_context(|| format!("failed to parse profile {}", path.display()))?;
    Ok(Some(profile))
}

pub fn list_profiles() -> Result<Vec<(String, Profile)>> {
    let dir = profiles_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut profiles = Vec::new();
    for entry in fs::read_dir(&dir)
        .with_context(|| format!("failed to read profiles directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let slug = path.file_stem().unwrap().to_string_lossy().to_string();
            let contents = fs::read_to_string(&path)
                .with_context(|| format!("failed to read profile {}", path.display()))?;
            let profile: Profile = toml::from_str(&contents)
                .with_context(|| format!("failed to parse profile {}", path.display()))?;
            profiles.push((slug, profile));
        }
    }
    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(profiles)
}

pub fn save_profile(slug: &str, profile: &Profile) -> Result<()> {
    let dir = profiles_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create profiles directory {}", dir.display()))?;
    let path = dir.join(format!("{slug}.toml"));
    let contents = toml::to_string_pretty(profile).context("failed to serialize profile")?;
    fs::write(&path, contents)
        .with_context(|| format!("failed to write profile {}", path.display()))
}

pub fn delete_profile(slug: &str) -> Result<bool> {
    let path = profiles_dir().join(format!("{slug}.toml"));
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(&path)
        .with_context(|| format!("failed to delete profile {}", path.display()))?;
    Ok(true)
}

pub fn profile_path(slug: &str) -> PathBuf {
    profiles_dir().join(format!("{slug}.toml"))
}
