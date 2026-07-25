# glaunch Rust Rewrite — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the glaunch bash game-launch wrapper as a Rust CLI with per-game profiles, MangoHud integration, TUI profile editor, dry-run/verbose modes, and hardware detection.

**Architecture:** Single-binary monolithic CLI using clap (derive) for subcommands and ratatui for the TUI. Config is TOML files under `~/.config/glaunch/`. The `run` subcommand replaces the process via Unix exec — no child processes.

**Tech Stack:** Rust stable (1.97+), clap 4 (derive), serde + toml, ratatui + crossterm, dirs, anyhow

## Global Constraints

- Rust edition 2024, stable toolchain only
- All profile/config fields are `Option<T>` to support layered resolution (CLI > profile > global > hardcoded)
- `anyhow` for application errors, no custom error types
- XDG config directory: `~/.config/glaunch/`
- Linux-only (Unix exec, sysfs reads) — no cross-platform abstraction needed
- The `run` subcommand must be fast — no unnecessary I/O when no profile is loaded

## File Structure

```
src/
├── main.rs              # Entry point, clap CLI definition, subcommand dispatch
├── cli.rs               # Clap derive structs for all subcommands and flags
├── config.rs            # Global config + profile TOML parsing, layered resolution
├── hardware.rs          # V-Cache CCD detection, GPU/display info, hardware cache
├── launch.rs            # Command building, env var setup, exec logic
├── profile.rs           # Profile CRUD operations (list, show, create, edit, delete)
├── tui/
│   ├── mod.rs           # TUI app state, event loop, view dispatch
│   ├── profile_list.rs  # Profile list table view
│   └── profile_edit.rs  # Profile editor form view
tests/
├── config_test.rs       # Config parsing and layered resolution tests
├── hardware_test.rs     # V-Cache detection with mock sysfs data
├── launch_test.rs       # Command building and dry-run output tests
└── integration_test.rs  # End-to-end dry-run tests
```

---

### Task 1: Project Scaffolding & CLI Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/cli.rs`

**Interfaces:**
- Produces: `Cli` struct (clap derive) with `RunArgs`, `ProfileCommand`, `InfoArgs` subcommands; `main()` that dispatches on subcommand match

- [ ] **Step 1: Initialize the Cargo project**

Run from the project root (`/home/anten/code/glaunch`):

```bash
cargo init --name glaunch
```

- [ ] **Step 2: Add dependencies to Cargo.toml**

Replace the generated `Cargo.toml` with:

```toml
[package]
name = "glaunch"
version = "0.1.0"
edition = "2024"
description = "Steam game launch wrapper with gamescope, gamemode, HDR, V-Cache, and per-game profiles"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
crossterm = "0.28"
dirs = "6"
ratatui = "0.29"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

- [ ] **Step 3: Create `src/cli.rs` with all clap derive structs**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "glaunch", about = "Steam game launch wrapper with gamescope, gamemode, HDR, and per-game profiles")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Launch a game (drop-in replacement for Steam launch options)
    Run(RunArgs),
    /// Manage per-game profiles
    Profile(ProfileArgs),
    /// Interactive profile editor
    Tui,
    /// Show detected hardware info
    Info(InfoArgs),
}

#[derive(clap::Args)]
pub struct RunArgs {
    /// Enable HDR inverse tone mapping (for SDR games)
    #[arg(long)]
    pub itm: bool,

    /// Enable FSR4 RDNA3 upgrade
    #[arg(long)]
    pub fsr4: bool,

    /// Disable HDR entirely
    #[arg(long)]
    pub no_hdr: bool,

    /// Disable adaptive sync (VRR)
    #[arg(long)]
    pub no_vrr: bool,

    /// Skip gamescope entirely (just gamemoderun + env)
    #[arg(long)]
    pub no_gamescope: bool,

    /// Pin to V-Cache CCD (auto-detects X3D topology)
    #[arg(long)]
    pub vcache: bool,

    /// Disable V-Cache CCD pinning
    #[arg(long)]
    pub no_vcache: bool,

    /// Fix Steam mouse stutter (LD_PRELOAD gamemode)
    #[arg(long)]
    pub fix_mouse: bool,

    /// Enable vkBasalt post-processing with optional profile name
    #[arg(long, num_args = 0..=1, default_missing_value = "")]
    pub vkbasalt: Option<String>,

    /// Override width (default: 3840)
    #[arg(short, long)]
    pub width: Option<u32>,

    /// Override height (default: 2160)
    #[arg(short = 'H', long)]
    pub height: Option<u32>,

    /// Enable MangoHud with optional config preset name
    #[arg(long, num_args = 0..=1, default_missing_value = "")]
    pub mangohud: Option<String>,

    /// Load a saved profile by name or Steam app ID
    #[arg(long)]
    pub profile: Option<String>,

    /// Print the final command instead of executing it
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Log each decision to stderr
    #[arg(short, long)]
    pub verbose: bool,

    /// The game command (everything after --)
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

#[derive(clap::Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(Subcommand)]
pub enum ProfileCommand {
    /// List all profiles
    List,
    /// Show a profile's settings
    Show { name: String },
    /// Create a new profile
    Create { name: String },
    /// Open a profile in $EDITOR
    Edit { name: String },
    /// Delete a profile
    Delete { name: String },
}

#[derive(clap::Args)]
pub struct InfoArgs {
    /// Force re-detection (bypass cache)
    #[arg(long)]
    pub refresh: bool,
}
```

- [ ] **Step 4: Create `src/main.rs` with dispatch skeleton**

```rust
mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => {
            eprintln!("run: not yet implemented");
            std::process::exit(1);
        }
        Command::Profile(args) => {
            eprintln!("profile: not yet implemented");
            std::process::exit(1);
        }
        Command::Tui => {
            eprintln!("tui: not yet implemented");
            std::process::exit(1);
        }
        Command::Info(args) => {
            eprintln!("info: not yet implemented");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 5: Verify it builds and the help text is correct**

```bash
cargo build
cargo run -- --help
cargo run -- run --help
cargo run -- profile --help
cargo run -- info --help
```

Verify: each subcommand shows the expected flags. `run` shows all the bash-script flags plus the new ones (dry-run, verbose, profile, mangohud). `profile` shows list/show/create/edit/delete. `info` shows `--refresh`.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/cli.rs
git commit -m "feat: scaffold Rust project with clap CLI skeleton"
```

---

### Task 2: Config & Profile Parsing with Layered Resolution

**Files:**
- Create: `src/config.rs`
- Create: `tests/config_test.rs`

**Interfaces:**
- Consumes: nothing (standalone module)
- Produces:
  - `GlobalConfig` — parsed from `config.toml`, all fields `Option<T>`
  - `Profile` — parsed from `profiles/<slug>.toml`, all fields `Option<T>`
  - `ResolvedSettings` — fully resolved (no Options), used by launch
  - `fn config_dir() -> PathBuf` — returns `~/.config/glaunch`
  - `fn load_global_config() -> Result<GlobalConfig>`
  - `fn load_profile(name: &str) -> Result<Option<Profile>>`
  - `fn list_profiles() -> Result<Vec<(String, Profile)>>`
  - `fn resolve_settings(cli: &RunArgs, profile: Option<&Profile>, global: &GlobalConfig) -> ResolvedSettings`

- [ ] **Step 1: Write tests for config parsing and resolution**

Create `tests/config_test.rs`:

```rust
use std::io::Write;

use glaunch::config::{
    GlobalConfig, Profile, ResolvedSettings, resolve_settings_from_layers,
};

#[test]
fn test_parse_global_config() {
    let toml_str = r#"
[defaults]
width = 2560
height = 1440
hdr = false
vrr = true
gamescope = true

[mangohud]
enabled = true
"#;
    let config: GlobalConfig = toml::from_str(toml_str).unwrap();
    let defaults = config.defaults.unwrap();
    assert_eq!(defaults.width, Some(2560));
    assert_eq!(defaults.height, Some(1440));
    assert_eq!(defaults.hdr, Some(false));
    assert_eq!(defaults.vrr, Some(true));
    assert_eq!(defaults.gamescope, Some(true));
    let mangohud = config.mangohud.unwrap();
    assert_eq!(mangohud.enabled, Some(true));
}

#[test]
fn test_parse_profile() {
    let toml_str = r#"
name = "Elden Ring"
steam_app_id = 1245620

[settings]
hdr = true
itm = true
vcache = true
width = 3840
height = 2160

[vkbasalt]
enabled = true
profile = "reshade-film"
"#;
    let profile: Profile = toml::from_str(toml_str).unwrap();
    assert_eq!(profile.name, Some("Elden Ring".to_string()));
    assert_eq!(profile.steam_app_id, Some(1245620));
    let settings = profile.settings.unwrap();
    assert_eq!(settings.itm, Some(true));
    assert_eq!(settings.vcache, Some(true));
    let vkbasalt = profile.vkbasalt.unwrap();
    assert_eq!(vkbasalt.enabled, Some(true));
    assert_eq!(vkbasalt.profile, Some("reshade-film".to_string()));
}

#[test]
fn test_parse_minimal_profile() {
    let toml_str = r#"
name = "Minimal Game"
"#;
    let profile: Profile = toml::from_str(toml_str).unwrap();
    assert_eq!(profile.name, Some("Minimal Game".to_string()));
    assert!(profile.settings.is_none());
    assert!(profile.mangohud.is_none());
    assert!(profile.vkbasalt.is_none());
}

#[test]
fn test_resolve_settings_hardcoded_defaults() {
    let resolved = resolve_settings_from_layers(None, None, &GlobalConfig::default());
    assert_eq!(resolved.width, 3840);
    assert_eq!(resolved.height, 2160);
    assert!(resolved.hdr);
    assert!(resolved.vrr);
    assert!(resolved.gamescope);
    assert!(!resolved.itm);
    assert!(!resolved.fsr4);
    assert!(!resolved.vcache);
    assert!(!resolved.fix_mouse);
    assert!(!resolved.mangohud);
    assert!(!resolved.vkbasalt);
}

#[test]
fn test_resolve_global_overrides_hardcoded() {
    let toml_str = r#"
[defaults]
hdr = false
width = 2560
"#;
    let global: GlobalConfig = toml::from_str(toml_str).unwrap();
    let resolved = resolve_settings_from_layers(None, None, &global);
    assert!(!resolved.hdr);
    assert_eq!(resolved.width, 2560);
    assert_eq!(resolved.height, 2160); // still hardcoded default
}

#[test]
fn test_resolve_profile_overrides_global() {
    let global_toml = r#"
[defaults]
hdr = false
width = 2560
"#;
    let profile_toml = r#"
name = "Test"

[settings]
hdr = true
"#;
    let global: GlobalConfig = toml::from_str(global_toml).unwrap();
    let profile: Profile = toml::from_str(profile_toml).unwrap();
    let resolved = resolve_settings_from_layers(None, Some(&profile), &global);
    assert!(resolved.hdr); // profile wins over global
    assert_eq!(resolved.width, 2560); // global wins over hardcoded
}

#[test]
fn test_resolve_cli_overrides_everything() {
    let global_toml = r#"
[defaults]
hdr = false
"#;
    let profile_toml = r#"
name = "Test"

[settings]
hdr = true
itm = true
"#;
    let global: GlobalConfig = toml::from_str(global_toml).unwrap();
    let profile: Profile = toml::from_str(profile_toml).unwrap();

    use glaunch::config::CliOverrides;
    let cli = CliOverrides {
        hdr: Some(false), // --no-hdr on CLI
        itm: None,        // not specified on CLI, profile's true should win
        ..Default::default()
    };

    let resolved = resolve_settings_from_layers(Some(&cli), Some(&profile), &global);
    assert!(!resolved.hdr); // CLI wins
    assert!(resolved.itm);  // profile wins (CLI didn't set it)
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --test config_test
```

Expected: compilation error — `glaunch::config` module doesn't exist yet.

- [ ] **Step 3: Implement `src/config.rs`**

```rust
use std::fs;
use std::path::{Path, PathBuf};

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
        width: cli.and_then(|c| c.width)
            .or_else(|| prof_settings.and_then(|s| s.width))
            .or_else(|| defaults.and_then(|d| d.width))
            .unwrap_or(3840),
        height: cli.and_then(|c| c.height)
            .or_else(|| prof_settings.and_then(|s| s.height))
            .or_else(|| defaults.and_then(|d| d.height))
            .unwrap_or(2160),
        hdr: cli.and_then(|c| c.hdr)
            .or_else(|| prof_settings.and_then(|s| s.hdr))
            .or_else(|| defaults.and_then(|d| d.hdr))
            .unwrap_or(true),
        vrr: cli.and_then(|c| c.vrr)
            .or_else(|| prof_settings.and_then(|s| s.vrr))
            .or_else(|| defaults.and_then(|d| d.vrr))
            .unwrap_or(true),
        gamescope: cli.and_then(|c| c.gamescope)
            .or_else(|| prof_settings.and_then(|s| s.gamescope))
            .or_else(|| defaults.and_then(|d| d.gamescope))
            .unwrap_or(true),
        itm: cli.and_then(|c| c.itm)
            .or_else(|| prof_settings.and_then(|s| s.itm))
            .unwrap_or(false),
        fsr4: cli.and_then(|c| c.fsr4)
            .or_else(|| prof_settings.and_then(|s| s.fsr4))
            .unwrap_or(false),
        vcache: cli.and_then(|c| c.vcache)
            .or_else(|| prof_settings.and_then(|s| s.vcache))
            .unwrap_or(false),
        fix_mouse: cli.and_then(|c| c.fix_mouse)
            .or_else(|| prof_settings.and_then(|s| s.fix_mouse))
            .unwrap_or(false),
        mangohud: cli.and_then(|c| c.mangohud)
            .or_else(|| prof_mangohud.and_then(|m| m.enabled))
            .or_else(|| global_mangohud.and_then(|m| m.enabled))
            .unwrap_or(false),
        mangohud_config: cli.and_then(|c| c.mangohud_config.clone())
            .or_else(|| prof_mangohud.and_then(|m| m.config.clone()))
            .or_else(|| global_mangohud.and_then(|m| m.config.clone())),
        vkbasalt: cli.and_then(|c| c.vkbasalt)
            .or_else(|| prof_vkbasalt.and_then(|v| v.enabled))
            .unwrap_or(false),
        vkbasalt_profile: cli.and_then(|c| c.vkbasalt_profile.clone())
            .or_else(|| prof_vkbasalt.and_then(|v| v.profile.clone())),
    }
}

// --- File I/O ---

pub fn load_global_config() -> Result<GlobalConfig> {
    let path = config_dir().join("config.toml");
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))
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
    let contents = toml::to_string_pretty(profile)
        .context("failed to serialize profile")?;
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
```

- [ ] **Step 4: Expose the config module as a public library**

Update `src/main.rs` to add:

```rust
pub mod config;
```

at the top (alongside `mod cli;`). This makes the config types available to integration tests via `glaunch::config`.

Also add to `Cargo.toml`:

```toml
[[bin]]
name = "glaunch"
path = "src/main.rs"

[lib]
name = "glaunch"
path = "src/lib.rs"
```

Create `src/lib.rs`:

```rust
pub mod config;
```

Move the `mod cli;` into `main.rs` only (it's private to the binary).

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test --test config_test
```

Expected: all 7 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/config.rs src/lib.rs src/main.rs Cargo.toml Cargo.lock tests/config_test.rs
git commit -m "feat: add config and profile parsing with layered resolution"
```

---

### Task 3: Hardware Detection (V-Cache)

**Files:**
- Create: `src/hardware.rs`
- Create: `tests/hardware_test.rs`

**Interfaces:**
- Consumes: `config::config_dir()` for hardware cache path
- Produces:
  - `VcacheInfo { cpus: String, l3_size_kb: u64 }` — detected V-Cache CCD info
  - `fn detect_vcache() -> Result<Option<VcacheInfo>>` — returns `None` if no asymmetric L3
  - `fn detect_vcache_cached(force_refresh: bool) -> Result<Option<VcacheInfo>>` — with 30-day cache
  - `HardwareInfo { vcache: Option<VcacheInfo>, gpu: Option<GpuInfo> }` — full hardware report
  - `fn detect_hardware(force_refresh: bool) -> Result<HardwareInfo>`

- [ ] **Step 1: Write tests for V-Cache detection**

Create `tests/hardware_test.rs`:

```rust
use std::fs;
use std::path::Path;

use glaunch::hardware::{detect_vcache_from_path, VcacheInfo};

fn create_mock_sysfs(base: &Path, cpus: &[(u32, u64, &str)]) {
    for (cpu_id, size_kb, shared_list) in cpus {
        let cache_dir = base
            .join(format!("cpu{cpu_id}"))
            .join("cache")
            .join("index3");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(cache_dir.join("size"), format!("{size_kb}K")).unwrap();
        fs::write(cache_dir.join("shared_cpu_list"), shared_list).unwrap();
    }
}

#[test]
fn test_vcache_asymmetric_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("cpu");
    // CCD0: 96MB L3 (V-Cache), CCD1: 32MB L3
    create_mock_sysfs(&base, &[
        (0, 98304, "0-7"),   // 96MB
        (1, 98304, "0-7"),
        (8, 32768, "8-15"),  // 32MB
        (9, 32768, "8-15"),
    ]);

    let result = detect_vcache_from_path(&base).unwrap();
    assert!(result.is_some());
    let info = result.unwrap();
    assert_eq!(info.cpus, "0-7");
    assert_eq!(info.l3_size_kb, 98304);
}

#[test]
fn test_vcache_symmetric_not_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("cpu");
    // Both CCDs 32MB — no X3D
    create_mock_sysfs(&base, &[
        (0, 32768, "0-7"),
        (8, 32768, "8-15"),
    ]);

    let result = detect_vcache_from_path(&base).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_vcache_single_ccd_not_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("cpu");
    // Single CCD — can't be asymmetric
    create_mock_sysfs(&base, &[
        (0, 32768, "0-7"),
        (1, 32768, "0-7"),
    ]);

    let result = detect_vcache_from_path(&base).unwrap();
    assert!(result.is_none());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --test hardware_test
```

Expected: compilation error — `glaunch::hardware` module doesn't exist yet. Also add `tempfile` as a dev dependency:

```bash
cargo add --dev tempfile
```

- [ ] **Step 3: Implement `src/hardware.rs`**

```rust
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::config_dir;

const CACHE_MAX_AGE_SECS: u64 = 30 * 24 * 60 * 60; // 30 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcacheInfo {
    pub cpus: String,
    pub l3_size_kb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub driver: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub vcache: Option<VcacheInfo>,
    pub gpus: Vec<GpuInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HardwareCache {
    #[serde(default)]
    timestamp: u64,
    #[serde(flatten)]
    info: HardwareInfo,
}

pub fn detect_vcache() -> Result<Option<VcacheInfo>> {
    detect_vcache_from_path(Path::new("/sys/devices/system/cpu"))
}

pub fn detect_vcache_from_path(cpu_base: &Path) -> Result<Option<VcacheInfo>> {
    let mut ccds: HashMap<String, u64> = HashMap::new();

    for entry in fs::read_dir(cpu_base).unwrap_or_else(|_| fs::read_dir(".").unwrap()) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let cache_dir = entry.path().join("cache").join("index3");
        if !cache_dir.exists() {
            continue;
        }
        let size_str = match fs::read_to_string(cache_dir.join("size")) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let size_kb: u64 = size_str
            .trim()
            .trim_end_matches('K')
            .parse()
            .unwrap_or(0);
        let cpus = match fs::read_to_string(cache_dir.join("shared_cpu_list")) {
            Ok(s) => s.trim().to_string(),
            Err(_) => continue,
        };
        ccds.entry(cpus).or_insert(size_kb);
    }

    if ccds.len() < 2 {
        return Ok(None);
    }

    let max_size = ccds.values().copied().max().unwrap_or(0);
    let min_size = ccds.values().copied().min().unwrap_or(0);

    if max_size == min_size {
        return Ok(None);
    }

    let (cpus, size) = ccds
        .into_iter()
        .max_by_key(|(_, size)| *size)
        .unwrap();

    Ok(Some(VcacheInfo {
        cpus,
        l3_size_kb: size,
    }))
}

pub fn detect_gpus() -> Vec<GpuInfo> {
    let drm_dir = Path::new("/sys/class/drm");
    let mut gpus = Vec::new();
    if !drm_dir.exists() {
        return gpus;
    }

    for entry in fs::read_dir(drm_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let device_dir = path.join("device");
        let gpu_name = fs::read_to_string(device_dir.join("label"))
            .or_else(|_| fs::read_to_string(device_dir.join("product_name")))
            .unwrap_or_else(|_| name.to_string())
            .trim()
            .to_string();
        let driver = device_dir
            .join("driver")
            .read_link()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_default();
        gpus.push(GpuInfo {
            name: gpu_name,
            driver,
        });
    }

    gpus
}

pub fn detect_hardware(force_refresh: bool) -> Result<HardwareInfo> {
    if !force_refresh {
        if let Some(cached) = load_hardware_cache()? {
            return Ok(cached);
        }
    }

    let info = HardwareInfo {
        vcache: detect_vcache()?,
        gpus: detect_gpus(),
    };

    save_hardware_cache(&info)?;
    Ok(info)
}

fn cache_path() -> PathBuf {
    config_dir().join("hardware_cache.toml")
}

fn load_hardware_cache() -> Result<Option<HardwareInfo>> {
    let path = cache_path();
    if !path.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(&path)?;
    let age = metadata
        .modified()?
        .elapsed()
        .unwrap_or_default()
        .as_secs();

    if age > CACHE_MAX_AGE_SECS {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)?;
    let cache: HardwareCache = toml::from_str(&contents)?;
    Ok(Some(cache.info))
}

fn save_hardware_cache(info: &HardwareInfo) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let cache = HardwareCache {
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        info: info.clone(),
    };
    let contents = toml::to_string_pretty(&cache)?;
    fs::write(cache_path(), contents)?;
    Ok(())
}

pub fn detect_vcache_cached(force_refresh: bool) -> Result<Option<VcacheInfo>> {
    let info = detect_hardware(force_refresh)?;
    Ok(info.vcache)
}
```

- [ ] **Step 4: Add the module to `src/lib.rs`**

```rust
pub mod config;
pub mod hardware;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test --test hardware_test
```

Expected: all 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/hardware.rs src/lib.rs tests/hardware_test.rs Cargo.toml Cargo.lock
git commit -m "feat: add V-Cache CCD detection with sysfs parsing and caching"
```

---

### Task 4: Launch Command Builder & Exec

**Files:**
- Create: `src/launch.rs`
- Create: `tests/launch_test.rs`

**Interfaces:**
- Consumes: `config::ResolvedSettings`, `hardware::VcacheInfo`
- Produces:
  - `LaunchPlan { env_vars: Vec<(String, String)>, command: Vec<String> }` — the fully built command
  - `fn build_launch_plan(settings: &ResolvedSettings, vcache: Option<&VcacheInfo>, game_command: &[String]) -> Result<LaunchPlan>`
  - `fn execute(plan: &LaunchPlan) -> Result<()>` — calls exec, never returns on success
  - `fn dry_run_output(plan: &LaunchPlan) -> String` — formatted command string

- [ ] **Step 1: Write tests for command building**

Create `tests/launch_test.rs`:

```rust
use glaunch::config::ResolvedSettings;
use glaunch::hardware::VcacheInfo;
use glaunch::launch::build_launch_plan;

fn default_settings() -> ResolvedSettings {
    ResolvedSettings {
        width: 3840,
        height: 2160,
        hdr: true,
        vrr: true,
        gamescope: true,
        itm: false,
        fsr4: false,
        vcache: false,
        fix_mouse: false,
        mangohud: false,
        mangohud_config: None,
        vkbasalt: false,
        vkbasalt_profile: None,
    }
}

#[test]
fn test_basic_gamescope_command() {
    let settings = default_settings();
    let game_cmd = vec!["wine".to_string(), "game.exe".to_string()];

    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert_eq!(plan.command[0], "gamemoderun");
    assert_eq!(plan.command[1], "gamescope");
    assert!(plan.command.contains(&"-W".to_string()));
    assert!(plan.command.contains(&"3840".to_string()));
    assert!(plan.command.contains(&"-H".to_string()));
    assert!(plan.command.contains(&"2160".to_string()));
    assert!(plan.command.contains(&"--fullscreen".to_string()));
    assert!(plan.command.contains(&"-r".to_string()));
    assert!(plan.command.contains(&"240".to_string()));
    assert!(plan.command.contains(&"--hdr-enabled".to_string()));
    assert!(plan.command.contains(&"--adaptive-sync".to_string()));
    // game command should be after the --
    let separator_pos = plan.command.iter().position(|s| s == "--").unwrap();
    assert_eq!(plan.command[separator_pos + 1], "wine");
    assert_eq!(plan.command[separator_pos + 2], "game.exe");
}

#[test]
fn test_no_gamescope() {
    let mut settings = default_settings();
    settings.gamescope = false;

    let game_cmd = vec!["wine".to_string(), "game.exe".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert_eq!(plan.command[0], "gamemoderun");
    assert_eq!(plan.command[1], "wine");
    assert_eq!(plan.command[2], "game.exe");
    assert!(!plan.command.contains(&"gamescope".to_string()));
}

#[test]
fn test_hdr_disabled() {
    let mut settings = default_settings();
    settings.hdr = false;

    let game_cmd = vec!["game".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert!(!plan.command.contains(&"--hdr-enabled".to_string()));
}

#[test]
fn test_itm_adds_flag() {
    let mut settings = default_settings();
    settings.itm = true;

    let game_cmd = vec!["game".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert!(plan.command.contains(&"--hdr-itm-enable".to_string()));
}

#[test]
fn test_fsr4_env_var() {
    let mut settings = default_settings();
    settings.fsr4 = true;

    let game_cmd = vec!["game".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert!(plan.env_vars.contains(&("PROTON_FSR4_RDNA3_UPGRADE".to_string(), "1".to_string())));
}

#[test]
fn test_vkbasalt_env_vars() {
    let mut settings = default_settings();
    settings.vkbasalt = true;
    settings.vkbasalt_profile = Some("reshade-film".to_string());

    let game_cmd = vec!["game".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert!(plan.env_vars.contains(&("ENABLE_VKBASALT".to_string(), "1".to_string())));
    let vkbasalt_config = plan.env_vars.iter()
        .find(|(k, _)| k == "VKBASALT_CONFIG_FILE")
        .map(|(_, v)| v.as_str());
    assert!(vkbasalt_config.is_some());
    assert!(vkbasalt_config.unwrap().contains("reshade-film.conf"));
}

#[test]
fn test_mangohud_env_vars() {
    let mut settings = default_settings();
    settings.mangohud = true;
    settings.mangohud_config = Some("fps-only".to_string());

    let game_cmd = vec!["game".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert!(plan.env_vars.contains(&("MANGOHUD".to_string(), "1".to_string())));
    let mangohud_config = plan.env_vars.iter()
        .find(|(k, _)| k == "MANGOHUD_CONFIG")
        .map(|(_, v)| v.as_str());
    assert!(mangohud_config.is_some());
    assert!(mangohud_config.unwrap().contains("fps-only.conf"));
}

#[test]
fn test_fix_mouse_ld_preload() {
    let mut settings = default_settings();
    settings.fix_mouse = true;

    let game_cmd = vec!["game".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    assert!(plan.env_vars.contains(&("LD_PRELOAD".to_string(), "/usr/lib/libgamemodeauto.so.0".to_string())));
}

#[test]
fn test_vcache_pinning_wraps_command() {
    let settings = default_settings();
    let vcache = VcacheInfo {
        cpus: "0-7".to_string(),
        l3_size_kb: 98304,
    };

    let game_cmd = vec!["game".to_string()];
    let plan = build_launch_plan(&settings, Some(&vcache), &game_cmd).unwrap();

    assert_eq!(plan.command[0], "systemd-run");
    assert!(plan.command.contains(&"--user".to_string()));
    assert!(plan.command.contains(&"--scope".to_string()));
    assert!(plan.command.contains(&"AllowedCPUs=0-7".to_string()));
}

#[test]
fn test_dry_run_output_format() {
    let mut settings = default_settings();
    settings.fsr4 = true;
    settings.gamescope = false;

    let game_cmd = vec!["wine".to_string(), "game.exe".to_string()];
    let plan = build_launch_plan(&settings, None, &game_cmd).unwrap();

    let output = glaunch::launch::dry_run_output(&plan);
    assert!(output.contains("PROTON_FSR4_RDNA3_UPGRADE=1"));
    assert!(output.contains("gamemoderun wine game.exe"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --test launch_test
```

Expected: compilation error — `glaunch::launch` module doesn't exist.

- [ ] **Step 3: Implement `src/launch.rs`**

```rust
use std::os::unix::process::CommandExt;
use std::process::Command as StdCommand;

use anyhow::{Result, bail};

use crate::config::ResolvedSettings;
use crate::hardware::VcacheInfo;

#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub env_vars: Vec<(String, String)>,
    pub command: Vec<String>,
}

pub fn build_launch_plan(
    settings: &ResolvedSettings,
    vcache: Option<&VcacheInfo>,
    game_command: &[String],
) -> Result<LaunchPlan> {
    if game_command.is_empty() {
        bail!("no game command provided");
    }

    let mut env_vars: Vec<(String, String)> = Vec::new();

    if settings.fsr4 {
        env_vars.push(("PROTON_FSR4_RDNA3_UPGRADE".into(), "1".into()));
    }

    if settings.vkbasalt {
        env_vars.push(("ENABLE_VKBASALT".into(), "1".into()));
        if let Some(profile) = &settings.vkbasalt_profile {
            if !profile.is_empty() {
                let home = dirs::home_dir().expect("could not determine home directory");
                let config_path = home
                    .join(".config")
                    .join("vkBasalt")
                    .join(format!("{profile}.conf"));
                env_vars.push(("VKBASALT_CONFIG_FILE".into(), config_path.to_string_lossy().to_string()));
            }
        }
    }

    if settings.mangohud {
        env_vars.push(("MANGOHUD".into(), "1".into()));
        if let Some(config) = &settings.mangohud_config {
            if !config.is_empty() {
                let home = dirs::home_dir().expect("could not determine home directory");
                let config_path = home
                    .join(".config")
                    .join("MangoHud")
                    .join(format!("{config}.conf"));
                env_vars.push(("MANGOHUD_CONFIG".into(), config_path.to_string_lossy().to_string()));
            }
        }
    }

    if settings.fix_mouse {
        env_vars.push(("LD_PRELOAD".into(), "/usr/lib/libgamemodeauto.so.0".into()));
    }

    let mut command: Vec<String> = Vec::new();

    // Gamescope wrapping
    if settings.gamescope {
        command.push("gamemoderun".into());
        command.push("gamescope".into());
        command.extend(["-W".into(), settings.width.to_string()]);
        command.extend(["-H".into(), settings.height.to_string()]);
        command.push("--fullscreen".into());
        command.extend(["-r".into(), "240".into()]);

        if settings.vrr {
            command.push("--adaptive-sync".into());
        }
        if settings.hdr {
            command.push("--hdr-enabled".into());
        }
        if settings.itm {
            command.push("--hdr-itm-enable".into());
        }

        command.push("--".into());
        command.extend_from_slice(game_command);
    } else {
        command.push("gamemoderun".into());
        command.extend_from_slice(game_command);
    }

    // V-Cache pinning wraps the whole thing
    if let Some(vcache) = vcache {
        let mut wrapped = vec![
            "systemd-run".into(),
            "--user".into(),
            "--scope".into(),
            "-p".into(),
            format!("AllowedCPUs={}", vcache.cpus),
            "--".into(),
        ];
        wrapped.append(&mut command);
        command = wrapped;
    }

    Ok(LaunchPlan { env_vars, command })
}

pub fn dry_run_output(plan: &LaunchPlan) -> String {
    let mut parts = Vec::new();
    for (key, value) in &plan.env_vars {
        parts.push(format!("{key}={value}"));
    }
    parts.extend(plan.command.iter().cloned());
    parts.join(" ")
}

pub fn execute(plan: &LaunchPlan) -> Result<()> {
    if plan.command.is_empty() {
        bail!("empty command");
    }

    let mut cmd = StdCommand::new(&plan.command[0]);
    cmd.args(&plan.command[1..]);
    for (key, value) in &plan.env_vars {
        cmd.env(key, value);
    }

    let err = cmd.exec();
    Err(err).with_context(|| format!("failed to exec {}", plan.command[0]))
}

use anyhow::Context;
```

- [ ] **Step 4: Add the module to `src/lib.rs`**

```rust
pub mod config;
pub mod hardware;
pub mod launch;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test --test launch_test
```

Expected: all 10 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/launch.rs src/lib.rs tests/launch_test.rs
git commit -m "feat: add launch command builder with env vars, gamescope, and vcache pinning"
```

---

### Task 5: Wire Up `run` and `info` Subcommands

**Files:**
- Modify: `src/main.rs`
- Modify: `src/cli.rs`
- Create: `tests/integration_test.rs`

**Interfaces:**
- Consumes: `config::*`, `hardware::*`, `launch::*`, `cli::RunArgs`
- Produces: Working `glaunch run` and `glaunch info` commands

- [ ] **Step 1: Write integration tests for dry-run**

Create `tests/integration_test.rs`:

```rust
use std::process::Command;

fn glaunch_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_glaunch"))
}

#[test]
fn test_run_dry_run_basic() {
    let output = glaunch_cmd()
        .args(["run", "--dry-run", "--", "wine", "game.exe"])
        .output()
        .expect("failed to run glaunch");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("gamemoderun"));
    assert!(stdout.contains("gamescope"));
    assert!(stdout.contains("wine"));
    assert!(stdout.contains("game.exe"));
}

#[test]
fn test_run_dry_run_no_gamescope() {
    let output = glaunch_cmd()
        .args(["run", "--dry-run", "--no-gamescope", "--", "wine", "game.exe"])
        .output()
        .expect("failed to run glaunch");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("gamemoderun"));
    assert!(!stdout.contains("gamescope"));
    assert!(stdout.contains("wine"));
}

#[test]
fn test_run_dry_run_fsr4() {
    let output = glaunch_cmd()
        .args(["run", "--dry-run", "--fsr4", "--", "game"])
        .output()
        .expect("failed to run glaunch");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("PROTON_FSR4_RDNA3_UPGRADE=1"));
}

#[test]
fn test_run_dry_run_no_hdr() {
    let output = glaunch_cmd()
        .args(["run", "--dry-run", "--no-hdr", "--", "game"])
        .output()
        .expect("failed to run glaunch");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(!stdout.contains("--hdr-enabled"));
}

#[test]
fn test_run_dry_run_verbose() {
    let output = glaunch_cmd()
        .args(["run", "--dry-run", "--verbose", "--itm", "--", "game"])
        .output()
        .expect("failed to run glaunch");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ITM"));
    assert!(stderr.contains("enabled"));
}

#[test]
fn test_run_missing_command() {
    let output = glaunch_cmd()
        .args(["run", "--dry-run"])
        .output()
        .expect("failed to run glaunch");

    assert!(!output.status.success());
}

#[test]
fn test_info_runs() {
    let output = glaunch_cmd()
        .args(["info", "--refresh"])
        .output()
        .expect("failed to run glaunch");

    // Should succeed even if no V-Cache detected
    assert!(output.status.success());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --test integration_test
```

Expected: failures because `run` and `info` still print "not yet implemented" and exit 1.

- [ ] **Step 3: Add app ID auto-detection to `src/config.rs`**

Add a function to find a profile by Steam app ID:

```rust
pub fn find_profile_by_app_id(app_id: u64) -> Result<Option<(String, Profile)>> {
    let profiles = list_profiles()?;
    Ok(profiles
        .into_iter()
        .find(|(_, p)| p.steam_app_id == Some(app_id)))
}

pub fn extract_steam_app_id(command: &[String]) -> Option<u64> {
    for arg in command {
        // Steam sometimes passes AppId=XXXXXX
        if let Some(id_str) = arg.strip_prefix("AppId=") {
            if let Ok(id) = id_str.parse::<u64>() {
                return Some(id);
            }
        }
        // Also try to extract from steamapps path: .../steamapps/common/GameName/...
        // by looking for sibling appmanifest files
        if arg.contains("steamapps") {
            if let Some(steamapps_idx) = arg.find("steamapps") {
                let steamapps_dir = &arg[..steamapps_idx + "steamapps".len()];
                let steamapps_path = std::path::Path::new(steamapps_dir);
                // Extract game folder name from path
                let after_common = arg[steamapps_idx + "steamapps".len()..].strip_prefix("/common/");
                if let Some(rest) = after_common {
                    let game_folder = rest.split('/').next().unwrap_or("");
                    // Scan appmanifest files to find matching installdir
                    if let Ok(entries) = std::fs::read_dir(steamapps_path) {
                        for entry in entries.flatten() {
                            let fname = entry.file_name();
                            let fname = fname.to_string_lossy();
                            if fname.starts_with("appmanifest_") && fname.ends_with(".acf") {
                                if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                                    if contents.contains(&format!("\"installdir\"\t\t\"{game_folder}\""))
                                        || contents.contains(&format!("\"installdir\"		\"{game_folder}\""))
                                    {
                                        let id_str = fname
                                            .strip_prefix("appmanifest_")
                                            .and_then(|s| s.strip_suffix(".acf"))
                                            .unwrap_or("");
                                        if let Ok(id) = id_str.parse::<u64>() {
                                            return Some(id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
```

- [ ] **Step 4: Add `CliOverrides::from_run_args` to `src/config.rs`**

Add this impl at the bottom of `src/config.rs`:

```rust
impl CliOverrides {
    pub fn from_run_args(args: &crate::cli::RunArgs) -> Self {
        Self {
            width: args.width,
            height: args.height,
            hdr: if args.no_hdr { Some(false) } else { None },
            vrr: if args.no_vrr { Some(false) } else { None },
            gamescope: if args.no_gamescope { Some(false) } else { None },
            itm: if args.itm { Some(true) } else { None },
            fsr4: if args.fsr4 { Some(true) } else { None },
            vcache: if args.vcache {
                Some(true)
            } else if args.no_vcache {
                Some(false)
            } else {
                None
            },
            fix_mouse: if args.fix_mouse { Some(true) } else { None },
            mangohud: args.mangohud.as_ref().map(|_| true),
            mangohud_config: args.mangohud.clone().filter(|s| !s.is_empty()),
            vkbasalt: args.vkbasalt.as_ref().map(|_| true),
            vkbasalt_profile: args.vkbasalt.clone().filter(|s| !s.is_empty()),
        }
    }
}
```

This requires `cli` to be visible from `config`. Since `cli` is in the binary crate and `config` is in the library, move `cli.rs` to the lib or pass `RunArgs` from main. The cleanest approach: keep `CliOverrides::from_run_args` in `main.rs` instead, as a free function, since it bridges the binary-only CLI types with the library types.

Instead, add this function in `src/main.rs`:

```rust
fn cli_overrides_from_run_args(args: &cli::RunArgs) -> config::CliOverrides {
    config::CliOverrides {
        width: args.width,
        height: args.height,
        hdr: if args.no_hdr { Some(false) } else { None },
        vrr: if args.no_vrr { Some(false) } else { None },
        gamescope: if args.no_gamescope { Some(false) } else { None },
        itm: if args.itm { Some(true) } else { None },
        fsr4: if args.fsr4 { Some(true) } else { None },
        vcache: if args.vcache {
            Some(true)
        } else if args.no_vcache {
            Some(false)
        } else {
            None
        },
        fix_mouse: if args.fix_mouse { Some(true) } else { None },
        mangohud: args.mangohud.as_ref().map(|_| true),
        mangohud_config: args.mangohud.clone().filter(|s| !s.is_empty()),
        vkbasalt: args.vkbasalt.as_ref().map(|_| true),
        vkbasalt_profile: args.vkbasalt.clone().filter(|s| !s.is_empty()),
    }
}
```

- [ ] **Step 4: Implement the `run` subcommand in `src/main.rs`**

Replace the `Command::Run` arm in `main()`:

```rust
Command::Run(args) => {
    let global = config::load_global_config()?;

    // Load profile (explicit flag, or auto-detect from Steam app ID)
    let profile = if let Some(ref profile_name) = args.profile {
        config::load_profile(profile_name)?
    } else if let Some(app_id) = config::extract_steam_app_id(&args.command) {
        if args.verbose {
            eprintln!("glaunch: detected Steam app ID {app_id}");
        }
        config::find_profile_by_app_id(app_id)?.map(|(_, p)| p)
    } else {
        None
    };

    let cli_overrides = cli_overrides_from_run_args(&args);
    let settings = config::resolve_settings_from_layers(
        Some(&cli_overrides),
        profile.as_ref(),
        &global,
    );

    if args.verbose {
        log_settings(&settings, profile.as_ref());
    }

    // V-Cache detection (only if settings say vcache is enabled)
    let vcache = if settings.vcache {
        match hardware::detect_vcache() {
            Ok(Some(info)) => {
                if args.verbose {
                    eprintln!(
                        "glaunch: V-Cache detected — pinning to CPUs {} (L3: {}MB)",
                        info.cpus,
                        info.l3_size_kb / 1024
                    );
                }
                Some(info)
            }
            Ok(None) => {
                eprintln!("glaunch: No X3D V-Cache topology detected — all CCDs have equal L3");
                None
            }
            Err(e) => {
                eprintln!("glaunch: V-Cache detection failed: {e}");
                None
            }
        }
    } else {
        None
    };

    let plan = launch::build_launch_plan(&settings, vcache.as_ref(), &args.command)?;

    if args.dry_run {
        println!("{}", launch::dry_run_output(&plan));
        return Ok(());
    }

    launch::execute(&plan)?;
    unreachable!()
}
```

Add the verbose logging helper in `main.rs`:

```rust
fn log_settings(settings: &config::ResolvedSettings, profile: Option<&config::Profile>) {
    let src = |from_profile: bool| {
        if from_profile {
            profile
                .and_then(|p| p.name.as_deref())
                .map(|n| format!(" (from profile '{n}')"))
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    eprintln!("glaunch: Resolution: {}x{}", settings.width, settings.height);
    eprintln!("glaunch: HDR: {}{}", if settings.hdr { "enabled" } else { "disabled" }, src(profile.is_some()));
    eprintln!("glaunch: VRR: {}", if settings.vrr { "enabled" } else { "disabled" });
    eprintln!("glaunch: Gamescope: {}", if settings.gamescope { "enabled" } else { "disabled" });
    if settings.itm {
        eprintln!("glaunch: ITM: enabled{}", src(profile.is_some()));
    }
    if settings.fsr4 {
        eprintln!("glaunch: FSR4: enabled");
    }
    if settings.mangohud {
        eprintln!("glaunch: MangoHud: enabled");
    }
    if settings.vkbasalt {
        let profile_info = settings.vkbasalt_profile.as_deref().unwrap_or("default");
        eprintln!("glaunch: vkBasalt: enabled (profile: {profile_info})");
    }
    if settings.fix_mouse {
        eprintln!("glaunch: Mouse fix: enabled");
    }
    if settings.vcache {
        eprintln!("glaunch: V-Cache pinning: requested");
    }
}
```

- [ ] **Step 5: Implement the `info` subcommand in `src/main.rs`**

Replace the `Command::Info` arm:

```rust
Command::Info(args) => {
    let info = hardware::detect_hardware(args.refresh)?;

    println!("=== glaunch hardware info ===\n");

    match &info.vcache {
        Some(vc) => {
            println!("V-Cache: detected");
            println!("  CPUs: {}", vc.cpus);
            println!("  L3 cache: {} MB", vc.l3_size_kb / 1024);
        }
        None => println!("V-Cache: not detected (symmetric L3 or single CCD)"),
    }

    if info.gpus.is_empty() {
        println!("\nGPU: not detected");
    } else {
        for (i, gpu) in info.gpus.iter().enumerate() {
            println!("\nGPU {i}: {}", gpu.name);
            if !gpu.driver.is_empty() {
                println!("  Driver: {}", gpu.driver);
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 6: Run all tests**

```bash
cargo test
```

Expected: all unit tests and integration tests pass.

- [ ] **Step 7: Manual smoke test**

```bash
cargo run -- run --dry-run -- echo "hello from game"
cargo run -- run --dry-run --verbose --itm --fsr4 --no-vrr -- wine game.exe
cargo run -- run --dry-run --no-gamescope --mangohud -- wine game.exe
cargo run -- info --refresh
```

Verify: dry-run output matches expected commands. Info shows hardware detection results.

- [ ] **Step 8: Commit**

```bash
git add src/main.rs src/config.rs tests/integration_test.rs
git commit -m "feat: wire up run (with dry-run/verbose) and info subcommands"
```

---

### Task 6: Profile CRUD Subcommands

**Files:**
- Modify: `src/main.rs`
- Create: `src/profile.rs`

**Interfaces:**
- Consumes: `config::*` (Profile, load_profile, list_profiles, save_profile, delete_profile, profile_path)
- Produces: Working `glaunch profile list|show|create|edit|delete` commands

- [ ] **Step 1: Create `src/profile.rs` with profile command handlers**

```rust
use std::io::{self, Write};
use std::process::Command as StdCommand;

use anyhow::{Context, Result, bail};

use crate::config::{
    self, MangoHudConfig, Profile, ProfileSettings, VkBasaltConfig,
};

pub fn list() -> Result<()> {
    let profiles = config::list_profiles()?;
    if profiles.is_empty() {
        println!("No profiles found. Create one with: glaunch profile create <name>");
        return Ok(());
    }

    println!("{:<20} {:<12} {:<5} {:<5} {:<7} {:<8} {:<9}",
        "NAME", "APP ID", "HDR", "VRR", "VCACHE", "MANGOHUD", "VKBASALT");
    println!("{}", "-".repeat(70));

    for (slug, profile) in &profiles {
        let display_name = profile.name.as_deref().unwrap_or(slug);
        let app_id = profile.steam_app_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string());
        let settings = profile.settings.as_ref();
        let yn = |opt: Option<bool>| match opt {
            Some(true) => "yes",
            Some(false) => "no",
            None => "-",
        };

        println!("{:<20} {:<12} {:<5} {:<5} {:<7} {:<8} {:<9}",
            display_name,
            app_id,
            yn(settings.and_then(|s| s.hdr)),
            yn(settings.and_then(|s| s.vrr)),
            yn(settings.and_then(|s| s.vcache)),
            yn(profile.mangohud.as_ref().and_then(|m| m.enabled)),
            yn(profile.vkbasalt.as_ref().and_then(|v| v.enabled)),
        );
    }

    Ok(())
}

pub fn show(name: &str) -> Result<()> {
    let profile = config::load_profile(name)?;
    match profile {
        Some(p) => {
            let toml_str = toml::to_string_pretty(&p)
                .context("failed to serialize profile")?;
            println!("# Profile: {name}\n");
            println!("{toml_str}");
        }
        None => bail!("profile '{name}' not found"),
    }
    Ok(())
}

pub fn create(name: &str) -> Result<()> {
    let path = config::profile_path(name);
    if path.exists() {
        bail!("profile '{name}' already exists. Use 'glaunch profile edit {name}' to modify it.");
    }

    let profile = Profile {
        name: Some(name.replace('-', " ")),
        steam_app_id: None,
        settings: Some(ProfileSettings::default()),
        mangohud: None,
        vkbasalt: None,
    };

    config::save_profile(name, &profile)?;
    println!("Created profile: {}", config::profile_path(name).display());
    println!("Edit it with: glaunch profile edit {name}");
    Ok(())
}

pub fn edit(name: &str) -> Result<()> {
    let path = config::profile_path(name);
    if !path.exists() {
        bail!("profile '{name}' not found. Create it first: glaunch profile create {name}");
    }

    let editor = std::env::var("EDITOR")
        .unwrap_or_else(|_| "vi".to_string());

    let status = StdCommand::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor '{editor}'"))?;

    if !status.success() {
        bail!("editor exited with non-zero status");
    }

    // Validate the edited file parses correctly
    match config::load_profile(name) {
        Ok(Some(_)) => println!("Profile '{name}' saved successfully."),
        Ok(None) => bail!("profile file disappeared after editing"),
        Err(e) => {
            eprintln!("Warning: profile '{name}' has parse errors: {e}");
            eprintln!("The file was saved but may not load correctly.");
        }
    }

    Ok(())
}

pub fn delete(name: &str) -> Result<()> {
    let path = config::profile_path(name);
    if !path.exists() {
        bail!("profile '{name}' not found");
    }

    print!("Delete profile '{name}'? [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        config::delete_profile(name)?;
        println!("Deleted profile '{name}'.");
    } else {
        println!("Cancelled.");
    }

    Ok(())
}
```

- [ ] **Step 2: Wire up profile subcommands in `src/main.rs`**

Replace the `Command::Profile` arm:

```rust
Command::Profile(args) => {
    match args.command {
        cli::ProfileCommand::List => profile::list()?,
        cli::ProfileCommand::Show { name } => profile::show(&name)?,
        cli::ProfileCommand::Create { name } => profile::create(&name)?,
        cli::ProfileCommand::Edit { name } => profile::edit(&name)?,
        cli::ProfileCommand::Delete { name } => profile::delete(&name)?,
    }
    Ok(())
}
```

Add `mod profile;` to `main.rs`.

- [ ] **Step 3: Add profile module to `src/lib.rs`**

```rust
pub mod config;
pub mod hardware;
pub mod launch;
pub mod profile;
```

- [ ] **Step 4: Build and manual smoke test**

```bash
cargo build
cargo run -- profile list
cargo run -- profile create test-game
cargo run -- profile show test-game
cargo run -- profile list
cargo run -- profile delete test-game
```

Verify: list shows empty initially, create makes a file in `~/.config/glaunch/profiles/`, show prints the TOML, list shows the new profile, delete removes it.

- [ ] **Step 5: Commit**

```bash
git add src/profile.rs src/main.rs src/lib.rs
git commit -m "feat: add profile CRUD subcommands (list, show, create, edit, delete)"
```

---

### Task 7: TUI Profile Editor

**Files:**
- Create: `src/tui/mod.rs`
- Create: `src/tui/profile_list.rs`
- Create: `src/tui/profile_edit.rs`

**Interfaces:**
- Consumes: `config::*` (Profile, list_profiles, save_profile, delete_profile, load_global_config), `hardware::detect_hardware`
- Produces: Working `glaunch tui` command with profile list view and profile editor

- [ ] **Step 1: Create `src/tui/mod.rs` — app state and event loop**

```rust
use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen,
    disable_raw_mode, enable_raw_mode,
};
use crossterm::execute;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::config::{self, Profile};
use crate::hardware::{self, HardwareInfo};

mod profile_edit;
mod profile_list;

#[derive(Debug, Clone, PartialEq)]
enum View {
    List,
    Edit(usize),
    New,
}

pub struct App {
    profiles: Vec<(String, Profile)>,
    hardware: HardwareInfo,
    view: View,
    list_state: profile_list::ListState,
    edit_state: Option<profile_edit::EditState>,
    should_quit: bool,
    status_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let profiles = config::list_profiles()?;
        let hardware = hardware::detect_hardware(false).unwrap_or_default();
        Ok(Self {
            list_state: profile_list::ListState::new(profiles.len()),
            profiles,
            hardware,
            view: View::List,
            edit_state: None,
            should_quit: false,
            status_message: None,
        })
    }

    fn reload_profiles(&mut self) -> Result<()> {
        self.profiles = config::list_profiles()?;
        self.list_state = profile_list::ListState::new(self.profiles.len());
        Ok(())
    }
}

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
        terminal.draw(|frame| {
            match &app.view {
                View::List => profile_list::render(frame, &app),
                View::Edit(_) | View::New => {
                    if let Some(ref edit_state) = app.edit_state {
                        profile_edit::render(frame, edit_state);
                    }
                }
            }
        })?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Ctrl+C always quits
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }

                match &app.view {
                    View::List => profile_list::handle_input(&mut app, key),
                    View::Edit(_) | View::New => {
                        profile_edit::handle_input(&mut app, key);
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
```

- [ ] **Step 2: Create `src/tui/profile_list.rs` — profile list table view**

```rust
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

use super::{App, View};
use crate::config;
use crate::tui::profile_edit;

#[derive(Debug, Clone)]
pub struct ListState {
    pub table_state: TableState,
    pub filter: String,
    pub filtering: bool,
    pub confirm_delete: bool,
}

impl ListState {
    pub fn new(count: usize) -> Self {
        let mut table_state = TableState::default();
        if count > 0 {
            table_state.select(Some(0));
        }
        Self {
            table_state,
            filter: String::new(),
            filtering: false,
            confirm_delete: false,
        }
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .split(frame.area());

    render_table(frame, app, chunks[0]);
    render_status_bar(frame, app, chunks[1]);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("App ID"),
        Cell::from("HDR"),
        Cell::from("VRR"),
        Cell::from("VCache"),
        Cell::from("MangoHud"),
        Cell::from("vkBasalt"),
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let yn = |opt: Option<bool>| match opt {
        Some(true) => Span::styled("yes", Style::default().fg(Color::Green)),
        Some(false) => Span::styled("no", Style::default().fg(Color::Red)),
        None => Span::styled("-", Style::default().fg(Color::DarkGray)),
    };

    let rows: Vec<Row> = app
        .profiles
        .iter()
        .filter(|(slug, profile)| {
            if app.list_state.filter.is_empty() {
                return true;
            }
            let filter = app.list_state.filter.to_lowercase();
            let name = profile.name.as_deref().unwrap_or(slug).to_lowercase();
            name.contains(&filter) || slug.to_lowercase().contains(&filter)
        })
        .map(|(slug, profile)| {
            let name = profile.name.as_deref().unwrap_or(slug);
            let app_id = profile
                .steam_app_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string());
            let s = profile.settings.as_ref();
            Row::new(vec![
                Cell::from(name.to_string()),
                Cell::from(app_id),
                Cell::from(yn(s.and_then(|s| s.hdr))),
                Cell::from(yn(s.and_then(|s| s.vrr))),
                Cell::from(yn(s.and_then(|s| s.vcache))),
                Cell::from(yn(profile.mangohud.as_ref().and_then(|m| m.enabled))),
                Cell::from(yn(profile.vkbasalt.as_ref().and_then(|v| v.enabled))),
            ])
        })
        .collect();

    let title = if app.list_state.confirm_delete {
        " Profiles — Press 'y' to confirm delete, any other key to cancel "
    } else {
        " Profiles — (n)ew  (e)dit  (d)elete  (/)search  (q)uit "
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Length(9),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = app.list_state.table_state.clone();
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let vcache_str = match &app.hardware.vcache {
        Some(vc) => format!("V-Cache: CPUs {} ({}MB L3)", vc.cpus, vc.l3_size_kb / 1024),
        None => "V-Cache: not detected".to_string(),
    };

    let text = if app.list_state.filtering {
        format!("/{} | {vcache_str}", app.list_state.filter)
    } else if let Some(msg) = &app.status_message {
        format!("{msg} | {vcache_str}")
    } else {
        format!("{} profiles | {vcache_str}", app.profiles.len())
    };

    let bar = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(bar, area);
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    if app.list_state.filtering {
        match key.code {
            KeyCode::Esc => {
                app.list_state.filtering = false;
                app.list_state.filter.clear();
            }
            KeyCode::Enter => {
                app.list_state.filtering = false;
            }
            KeyCode::Backspace => {
                app.list_state.filter.pop();
            }
            KeyCode::Char(c) => {
                app.list_state.filter.push(c);
            }
            _ => {}
        }
        return;
    }

    if app.list_state.confirm_delete {
        app.list_state.confirm_delete = false;
        if key.code == KeyCode::Char('y') {
            if let Some(idx) = app.list_state.table_state.selected() {
                if let Some((slug, _)) = app.profiles.get(idx) {
                    let slug = slug.clone();
                    match config::delete_profile(&slug) {
                        Ok(true) => {
                            app.status_message = Some(format!("Deleted '{slug}'"));
                            let _ = app.reload_profiles();
                        }
                        Ok(false) => {
                            app.status_message = Some(format!("'{slug}' not found"));
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Error: {e}"));
                        }
                    }
                }
            }
        } else {
            app.status_message = Some("Delete cancelled".to_string());
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => {
            let count = app.profiles.len();
            if count > 0 {
                let i = app.list_state.table_state.selected().unwrap_or(0);
                app.list_state.table_state.select(Some((i + 1) % count));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let count = app.profiles.len();
            if count > 0 {
                let i = app.list_state.table_state.selected().unwrap_or(0);
                app.list_state
                    .table_state
                    .select(Some(if i == 0 { count - 1 } else { i - 1 }));
            }
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            if let Some(idx) = app.list_state.table_state.selected() {
                if idx < app.profiles.len() {
                    let (slug, profile) = &app.profiles[idx];
                    app.edit_state = Some(profile_edit::EditState::from_profile(
                        slug.clone(),
                        profile.clone(),
                        false,
                    ));
                    app.view = View::Edit(idx);
                }
            }
        }
        KeyCode::Char('n') => {
            app.edit_state = Some(profile_edit::EditState::new_profile());
            app.view = View::New;
        }
        KeyCode::Char('d') => {
            if app.list_state.table_state.selected().is_some() {
                app.list_state.confirm_delete = true;
            }
        }
        KeyCode::Char('/') => {
            app.list_state.filtering = true;
            app.list_state.filter.clear();
        }
        _ => {}
    }
}
```

- [ ] **Step 3: Create `src/tui/profile_edit.rs` — profile editor form**

```rust
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::App;
use super::View;
use crate::config::{
    self, MangoHudConfig, Profile, ProfileSettings, VkBasaltConfig,
};

#[derive(Debug, Clone)]
pub struct EditState {
    pub slug: String,
    pub is_new: bool,
    pub fields: Vec<Field>,
    pub selected: usize,
    pub editing_text: bool,
}

#[derive(Debug, Clone)]
pub enum Field {
    Text { label: String, value: String },
    Toggle { label: String, value: bool },
}

impl Field {
    fn label(&self) -> &str {
        match self {
            Field::Text { label, .. } | Field::Toggle { label, .. } => label,
        }
    }
}

impl EditState {
    pub fn new_profile() -> Self {
        Self {
            slug: String::new(),
            is_new: true,
            fields: vec![
                Field::Text { label: "Slug".into(), value: String::new() },
                Field::Text { label: "Name".into(), value: String::new() },
                Field::Text { label: "Steam App ID".into(), value: String::new() },
                Field::Text { label: "Width".into(), value: "3840".into() },
                Field::Text { label: "Height".into(), value: "2160".into() },
                Field::Toggle { label: "HDR".into(), value: true },
                Field::Toggle { label: "ITM".into(), value: false },
                Field::Toggle { label: "VRR".into(), value: true },
                Field::Toggle { label: "FSR4".into(), value: false },
                Field::Toggle { label: "Gamescope".into(), value: true },
                Field::Toggle { label: "V-Cache".into(), value: false },
                Field::Toggle { label: "Fix Mouse".into(), value: false },
                Field::Toggle { label: "MangoHud".into(), value: false },
                Field::Toggle { label: "vkBasalt".into(), value: false },
                Field::Text { label: "vkBasalt Profile".into(), value: String::new() },
            ],
            selected: 0,
            editing_text: false,
        }
    }

    pub fn from_profile(slug: String, profile: Profile, is_new: bool) -> Self {
        let s = profile.settings.as_ref();
        let m = profile.mangohud.as_ref();
        let v = profile.vkbasalt.as_ref();

        Self {
            slug: slug.clone(),
            is_new,
            fields: vec![
                Field::Text { label: "Slug".into(), value: slug },
                Field::Text { label: "Name".into(), value: profile.name.unwrap_or_default() },
                Field::Text { label: "Steam App ID".into(), value: profile.steam_app_id.map(|id| id.to_string()).unwrap_or_default() },
                Field::Text { label: "Width".into(), value: s.and_then(|s| s.width).map(|w| w.to_string()).unwrap_or_default() },
                Field::Text { label: "Height".into(), value: s.and_then(|s| s.height).map(|h| h.to_string()).unwrap_or_default() },
                Field::Toggle { label: "HDR".into(), value: s.and_then(|s| s.hdr).unwrap_or(true) },
                Field::Toggle { label: "ITM".into(), value: s.and_then(|s| s.itm).unwrap_or(false) },
                Field::Toggle { label: "VRR".into(), value: s.and_then(|s| s.vrr).unwrap_or(true) },
                Field::Toggle { label: "FSR4".into(), value: s.and_then(|s| s.fsr4).unwrap_or(false) },
                Field::Toggle { label: "Gamescope".into(), value: s.and_then(|s| s.gamescope).unwrap_or(true) },
                Field::Toggle { label: "V-Cache".into(), value: s.and_then(|s| s.vcache).unwrap_or(false) },
                Field::Toggle { label: "Fix Mouse".into(), value: s.and_then(|s| s.fix_mouse).unwrap_or(false) },
                Field::Toggle { label: "MangoHud".into(), value: m.and_then(|m| m.enabled).unwrap_or(false) },
                Field::Toggle { label: "vkBasalt".into(), value: v.and_then(|v| v.enabled).unwrap_or(false) },
                Field::Text { label: "vkBasalt Profile".into(), value: v.and_then(|v| v.profile.clone()).unwrap_or_default() },
            ],
            selected: 0,
            editing_text: false,
        }
    }

    fn to_profile(&self) -> (String, Profile) {
        let field_val = |label: &str| -> String {
            self.fields.iter().find_map(|f| match f {
                Field::Text { label: l, value } if l == label => Some(value.clone()),
                _ => None,
            }).unwrap_or_default()
        };
        let field_bool = |label: &str| -> bool {
            self.fields.iter().find_map(|f| match f {
                Field::Toggle { label: l, value } if l == label => Some(*value),
                _ => None,
            }).unwrap_or(false)
        };

        let slug = field_val("Slug");
        let name = field_val("Name");
        let app_id_str = field_val("Steam App ID");
        let width_str = field_val("Width");
        let height_str = field_val("Height");
        let vkbasalt_profile = field_val("vkBasalt Profile");

        let profile = Profile {
            name: if name.is_empty() { None } else { Some(name) },
            steam_app_id: app_id_str.parse().ok(),
            settings: Some(ProfileSettings {
                width: width_str.parse().ok(),
                height: height_str.parse().ok(),
                hdr: Some(field_bool("HDR")),
                vrr: Some(field_bool("VRR")),
                gamescope: Some(field_bool("Gamescope")),
                itm: Some(field_bool("ITM")),
                fsr4: Some(field_bool("FSR4")),
                vcache: Some(field_bool("V-Cache")),
                fix_mouse: Some(field_bool("Fix Mouse")),
            }),
            mangohud: Some(MangoHudConfig {
                enabled: Some(field_bool("MangoHud")),
                config: None,
            }),
            vkbasalt: Some(VkBasaltConfig {
                enabled: Some(field_bool("vkBasalt")),
                profile: if vkbasalt_profile.is_empty() { None } else { Some(vkbasalt_profile) },
            }),
        };

        (slug, profile)
    }
}

pub fn render(frame: &mut Frame, state: &EditState) {
    let title = if state.is_new { " New Profile " } else { " Edit Profile " };
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(frame.area());
    frame.render_widget(block, frame.area());

    let field_count = state.fields.len();
    let constraints: Vec<Constraint> = (0..field_count + 1)
        .map(|_| Constraint::Length(1))
        .collect();

    let chunks = Layout::vertical(constraints).split(inner);

    for (i, field) in state.fields.iter().enumerate() {
        let is_selected = i == state.selected;
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let text = match field {
            Field::Text { label, value } => {
                let cursor = if is_selected && state.editing_text { "|" } else { "" };
                format!("  {label:<18} {value}{cursor}")
            }
            Field::Toggle { label, value } => {
                let indicator = if *value { "[x]" } else { "[ ]" };
                format!("  {label:<18} {indicator}")
            }
        };

        if i < chunks.len() {
            frame.render_widget(
                Paragraph::new(text).style(style),
                chunks[i],
            );
        }
    }

    // Help text at the bottom
    let help_idx = field_count;
    if help_idx < chunks.len() {
        let help = if state.editing_text {
            "  Enter: confirm | Esc: cancel"
        } else {
            "  j/k: navigate | Enter/Space: edit/toggle | s: save | Esc: cancel"
        };
        frame.render_widget(
            Paragraph::new(help).style(Style::default().fg(Color::DarkGray)),
            chunks[help_idx],
        );
    }
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    let edit_state = match &mut app.edit_state {
        Some(s) => s,
        None => return,
    };

    if edit_state.editing_text {
        match key.code {
            KeyCode::Esc => {
                edit_state.editing_text = false;
            }
            KeyCode::Enter => {
                edit_state.editing_text = false;
            }
            KeyCode::Backspace => {
                if let Field::Text { value, .. } = &mut edit_state.fields[edit_state.selected] {
                    value.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Field::Text { value, .. } = &mut edit_state.fields[edit_state.selected] {
                    value.push(c);
                }
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.edit_state = None;
            app.view = View::List;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let count = edit_state.fields.len();
            edit_state.selected = (edit_state.selected + 1) % count;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let count = edit_state.fields.len();
            edit_state.selected = if edit_state.selected == 0 {
                count - 1
            } else {
                edit_state.selected - 1
            };
        }
        KeyCode::Enter | KeyCode::Char(' ') => match &mut edit_state.fields[edit_state.selected] {
            Field::Toggle { value, .. } => *value = !*value,
            Field::Text { .. } => edit_state.editing_text = true,
        },
        KeyCode::Char('s') => {
            let (slug, profile) = edit_state.to_profile();
            if slug.is_empty() {
                app.status_message = Some("Slug cannot be empty".to_string());
                return;
            }
            match config::save_profile(&slug, &profile) {
                Ok(()) => {
                    app.status_message = Some(format!("Saved profile '{slug}'"));
                    let _ = app.reload_profiles();
                    app.edit_state = None;
                    app.view = View::List;
                }
                Err(e) => {
                    app.status_message = Some(format!("Save failed: {e}"));
                }
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Wire up TUI in `src/main.rs`**

Add `mod tui;` to the top of `main.rs` (the binary, not lib.rs — tui depends on binary-level modules).

Actually, since the TUI module uses `crate::config` and `crate::hardware`, it should live in the lib crate. Add to `src/lib.rs`:

```rust
pub mod config;
pub mod hardware;
pub mod launch;
pub mod profile;
pub mod tui;
```

Replace the `Command::Tui` arm in `main.rs`:

```rust
Command::Tui => {
    glaunch::tui::run_tui()?;
    Ok(())
}
```

- [ ] **Step 5: Build and manual smoke test**

```bash
cargo build
# Create a test profile first
cargo run -- profile create test-game
# Launch the TUI
cargo run -- tui
```

In the TUI:
- Verify the profile list shows `test-game`
- Press `e` to edit, verify the form shows all fields
- Toggle some fields with Space, change text with Enter
- Press `s` to save
- Press `n` to create a new profile
- Press `q` to quit
- Verify `Ctrl+C` also exits cleanly

- [ ] **Step 6: Commit**

```bash
git add src/tui/ src/lib.rs src/main.rs
git commit -m "feat: add ratatui TUI with profile list and editor views"
```

---

### Task 8: Polish, Validation & Final Integration

**Files:**
- Modify: `src/main.rs` — ensure all arms return proper exit codes
- Modify: `src/launch.rs` — vkBasalt config file validation
- Modify: `src/config.rs` — ensure `config_dir()` creates directories on first use
- Create: `.gitignore`

**Interfaces:**
- Consumes: all modules
- Produces: production-ready binary with proper error messages and exit codes

- [ ] **Step 1: Create `.gitignore`**

```gitignore
/target
```

- [ ] **Step 2: Add vkBasalt config file validation to launch**

In `src/launch.rs`, in the vkbasalt env var section of `build_launch_plan`, add validation after computing `config_path`:

```rust
if settings.vkbasalt {
    env_vars.push(("ENABLE_VKBASALT".into(), "1".into()));
    if let Some(profile) = &settings.vkbasalt_profile {
        if !profile.is_empty() {
            let home = dirs::home_dir().expect("could not determine home directory");
            let config_path = home
                .join(".config")
                .join("vkBasalt")
                .join(format!("{profile}.conf"));
            if !config_path.exists() {
                bail!("vkBasalt profile not found: {}", config_path.display());
            }
            env_vars.push(("VKBASALT_CONFIG_FILE".into(), config_path.to_string_lossy().to_string()));
        }
    }
}
```

Do the same for MangoHud config:

```rust
if settings.mangohud {
    env_vars.push(("MANGOHUD".into(), "1".into()));
    if let Some(config) = &settings.mangohud_config {
        if !config.is_empty() {
            let home = dirs::home_dir().expect("could not determine home directory");
            let config_path = home
                .join(".config")
                .join("MangoHud")
                .join(format!("{config}.conf"));
            if !config_path.exists() {
                bail!("MangoHud config not found: {}", config_path.display());
            }
            env_vars.push(("MANGOHUD_CONFIG".into(), config_path.to_string_lossy().to_string()));
        }
    }
}
```

- [ ] **Step 3: Add exit code handling to `main.rs`**

Wrap the `main()` function to set exit codes per spec:

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("glaunch: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        // ... existing match arms, but returning Ok(()) or Err
    }
}
```

- [ ] **Step 4: Ensure config directory is created on first profile save**

Already handled by `config::save_profile` calling `fs::create_dir_all`. Verify by testing with a clean `~/.config/glaunch/` (rename it temporarily):

```bash
mv ~/.config/glaunch ~/.config/glaunch.bak 2>/dev/null
cargo run -- profile create fresh-test
ls ~/.config/glaunch/profiles/fresh-test.toml
mv ~/.config/glaunch.bak ~/.config/glaunch 2>/dev/null
```

- [ ] **Step 5: Run the full test suite**

```bash
cargo test
cargo clippy -- -D warnings
```

Fix any clippy warnings.

- [ ] **Step 6: End-to-end manual test matching the original bash script**

Run the Rust version with the same flags the bash script supports and compare dry-run output:

```bash
# Basic gamescope launch
cargo run -- run --dry-run -- wine game.exe

# All features on
cargo run -- run --dry-run --itm --fsr4 --vcache --fix-mouse --mangohud --vkbasalt -- wine game.exe

# No gamescope
cargo run -- run --dry-run --no-gamescope -- wine game.exe

# Custom resolution
cargo run -- run --dry-run -w 2560 -H 1440 --no-hdr --no-vrr -- wine game.exe

# Verbose mode
cargo run -- run --dry-run --verbose --itm -- wine game.exe
```

Compare each against what the bash script would produce.

- [ ] **Step 7: Commit**

```bash
git add .gitignore src/main.rs src/launch.rs src/config.rs
git commit -m "feat: add validation, exit codes, and polish for production readiness"
```

- [ ] **Step 8: Install the binary**

```bash
cargo install --path .
```

Verify `glaunch` is now the Rust binary:

```bash
which glaunch
glaunch --help
glaunch info
```

Note: this will shadow the bash script in `~/bin/glaunch` if `~/.cargo/bin` is earlier in `$PATH`. Consider renaming or removing the old bash script once satisfied.
