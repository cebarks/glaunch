use std::io::{self, Write};
use std::process::Command as StdCommand;

use anyhow::{Context, Result, bail};

use crate::config::{self, Profile, ProfileSettings};

pub fn list() -> Result<()> {
    let profiles = config::list_profiles()?;
    if profiles.is_empty() {
        println!("No profiles found. Create one with: glaunch profile create <name>");
        return Ok(());
    }

    println!(
        "{:<20} {:<12} {:<5} {:<5} {:<7} {:<8} {:<9}",
        "NAME", "APP ID", "HDR", "VRR", "VCACHE", "MANGOHUD", "VKBASALT"
    );
    println!("{}", "-".repeat(70));

    for (slug, profile) in &profiles {
        let display_name = profile.name.as_deref().unwrap_or(slug);
        let app_id = profile
            .steam_app_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string());
        let settings = profile.settings.as_ref();
        let yn = |opt: Option<bool>| match opt {
            Some(true) => "yes",
            Some(false) => "no",
            None => "-",
        };

        println!(
            "{:<20} {:<12} {:<5} {:<5} {:<7} {:<8} {:<9}",
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
            let toml_str = toml::to_string_pretty(&p).context("failed to serialize profile")?;
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

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

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
