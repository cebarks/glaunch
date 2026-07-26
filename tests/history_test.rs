use glaunch::config::CliOverrides;
use glaunch::history::{History, derive_app_slug, has_overrides, record_launch};
use tempfile::TempDir;

#[test]
fn test_derive_slug_from_steamapps_path() {
    let cmd = vec!["/home/user/.steam/steam/steamapps/common/Elden Ring/eldenring.exe".to_string()];
    assert_eq!(derive_app_slug(&cmd), "elden-ring");
}

#[test]
fn test_derive_slug_from_steamapps_path_nested() {
    let cmd =
        vec!["/home/user/.steam/steam/steamapps/common/Baldur's Gate 3/bin/bg3.exe".to_string()];
    assert_eq!(derive_app_slug(&cmd), "baldurs-gate-3");
}

#[test]
fn test_derive_slug_fallback_to_executable() {
    let cmd = vec!["/usr/bin/some-game".to_string(), "--fullscreen".to_string()];
    assert_eq!(derive_app_slug(&cmd), "some-game");
}

#[test]
fn test_derive_slug_empty_command() {
    let cmd: Vec<String> = vec![];
    assert_eq!(derive_app_slug(&cmd), "unknown");
}

#[test]
fn test_has_overrides_empty() {
    let cli = CliOverrides::default();
    assert!(!has_overrides(&cli));
}

#[test]
fn test_has_overrides_with_bool() {
    let cli = CliOverrides {
        fsr4: Some(true),
        ..Default::default()
    };
    assert!(has_overrides(&cli));
}

#[test]
fn test_has_overrides_with_string() {
    let cli = CliOverrides {
        mangohud_config: Some("minimal".to_string()),
        ..Default::default()
    };
    assert!(has_overrides(&cli));
}

#[test]
fn test_record_launch_creates_entry() {
    let mut history = History::default();
    let overrides = CliOverrides {
        fsr4: Some(true),
        ..Default::default()
    };

    record_launch(
        &mut history,
        "elden-ring",
        Some(1245620),
        vec!["eldenring.exe".to_string()],
        overrides,
    );

    assert_eq!(history.len(), 1);
    let app = &history["elden-ring"];
    assert_eq!(app.steam_app_id, Some(1245620));
    assert_eq!(app.launches.len(), 1);
    assert_eq!(app.launches[0].overrides.fsr4, Some(true));
}

#[test]
fn test_record_launch_rolling_cap() {
    let mut history = History::default();

    for i in 0..5 {
        let overrides = CliOverrides {
            width: Some(1000 + i),
            ..Default::default()
        };
        record_launch(
            &mut history,
            "test-game",
            None,
            vec!["game.exe".to_string()],
            overrides,
        );
    }

    let app = &history["test-game"];
    assert_eq!(app.launches.len(), 3);
    // Newest first — last inserted should have width=1004
    assert_eq!(app.launches[0].overrides.width, Some(1004));
    assert_eq!(app.launches[2].overrides.width, Some(1002));
}

#[test]
fn test_record_launch_updates_app_id() {
    let mut history = History::default();

    // First launch without app ID
    record_launch(
        &mut history,
        "test-game",
        None,
        vec!["game.exe".to_string()],
        CliOverrides::default(),
    );
    assert_eq!(history["test-game"].steam_app_id, None);

    // Second launch with app ID — should update
    record_launch(
        &mut history,
        "test-game",
        Some(12345),
        vec!["game.exe".to_string()],
        CliOverrides::default(),
    );
    assert_eq!(history["test-game"].steam_app_id, Some(12345));
}

#[test]
fn test_history_json_round_trip() {
    let mut history = History::default();
    let overrides = CliOverrides {
        fsr4: Some(true),
        vcache: Some(true),
        ..Default::default()
    };
    record_launch(
        &mut history,
        "test-game",
        Some(99999),
        vec!["game.exe".to_string()],
        overrides,
    );

    let json = serde_json::to_string_pretty(&history).unwrap();
    let deserialized: History = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.len(), 1);
    assert_eq!(deserialized["test-game"].steam_app_id, Some(99999));
    assert_eq!(
        deserialized["test-game"].launches[0].overrides.fsr4,
        Some(true)
    );
}

#[test]
fn test_save_and_load_history_file() {
    let tmp = TempDir::new().unwrap();
    let history_file = tmp.path().join("history.json");

    let mut history = History::default();
    let overrides = CliOverrides {
        fsr4: Some(true),
        mangohud: Some(true),
        mangohud_config: Some("fps-only".to_string()),
        ..Default::default()
    };
    record_launch(
        &mut history,
        "test-game",
        Some(12345),
        vec!["game.exe".to_string()],
        overrides,
    );

    // Write directly to temp path
    let json = serde_json::to_string_pretty(&history).unwrap();
    std::fs::write(&history_file, &json).unwrap();

    // Read back
    let contents = std::fs::read_to_string(&history_file).unwrap();
    let loaded: History = serde_json::from_str(&contents).unwrap();

    assert_eq!(loaded.len(), 1);
    let app = &loaded["test-game"];
    assert_eq!(app.steam_app_id, Some(12345));
    assert_eq!(app.launches.len(), 1);
    assert_eq!(app.launches[0].overrides.fsr4, Some(true));
    assert_eq!(app.launches[0].overrides.mangohud, Some(true));
    assert_eq!(
        app.launches[0].overrides.mangohud_config,
        Some("fps-only".to_string())
    );
    // None fields should not appear in JSON
    assert!(!json.contains("\"width\""));
    assert!(!json.contains("\"vkbasalt\""));
}

#[test]
fn test_load_empty_file_fails_gracefully() {
    let tmp = TempDir::new().unwrap();
    let history_file = tmp.path().join("history.json");
    std::fs::write(&history_file, "").unwrap();

    // Read the empty file and attempt to deserialize
    let contents = std::fs::read_to_string(&history_file).unwrap();
    let result: Result<History, _> = serde_json::from_str(&contents);
    assert!(result.is_err());
}

#[test]
fn test_default_history_is_empty() {
    // Tests the serialization contract that History::default() is an empty map.
    // Note: load_history()'s fallback to default when the file doesn't exist
    // is tested indirectly through this contract — when load_history() encounters
    // a missing ~.config/glaunch/history.json, it returns History::default().
    let history = History::default();
    assert!(history.is_empty());
}

#[test]
fn test_promote_builds_correct_profile() {
    let overrides = CliOverrides {
        fsr4: Some(true),
        vcache: Some(true),
        hdr: Some(false),
        mangohud: Some(true),
        mangohud_config: Some("minimal".to_string()),
        vkbasalt: Some(true),
        vkbasalt_profile: Some("reshade".to_string()),
        width: Some(2560),
        height: Some(1440),
        ..Default::default()
    };

    let profile =
        glaunch::history::build_profile_from_overrides(&overrides, Some(1245620), "Elden Ring");

    assert_eq!(profile.name, Some("Elden Ring".to_string()));
    assert_eq!(profile.steam_app_id, Some(1245620));

    let settings = profile.settings.unwrap();
    assert_eq!(settings.fsr4, Some(true));
    assert_eq!(settings.vcache, Some(true));
    assert_eq!(settings.hdr, Some(false));
    assert_eq!(settings.width, Some(2560));
    assert_eq!(settings.height, Some(1440));
    // fix_mouse was not set in overrides, so should be None
    assert_eq!(settings.fix_mouse, None);

    let mangohud = profile.mangohud.unwrap();
    assert_eq!(mangohud.enabled, Some(true));
    assert_eq!(mangohud.config, Some("minimal".to_string()));

    let vkbasalt = profile.vkbasalt.unwrap();
    assert_eq!(vkbasalt.enabled, Some(true));
    assert_eq!(vkbasalt.profile, Some("reshade".to_string()));
}

#[test]
fn test_promote_minimal_overrides() {
    let overrides = CliOverrides {
        fsr4: Some(true),
        ..Default::default()
    };

    let profile = glaunch::history::build_profile_from_overrides(&overrides, None, "test-game");

    assert_eq!(profile.name, Some("test-game".to_string()));
    assert_eq!(profile.steam_app_id, None);

    let settings = profile.settings.unwrap();
    assert_eq!(settings.fsr4, Some(true));
    assert_eq!(settings.hdr, None);
    assert!(profile.mangohud.is_none());
    assert!(profile.vkbasalt.is_none());
}
