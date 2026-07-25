use glaunch::config::{GlobalConfig, Profile, resolve_settings_from_layers};

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
    assert!(resolved.itm); // profile wins (CLI didn't set it)
}
