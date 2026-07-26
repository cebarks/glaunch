use std::collections::BTreeMap;

use glaunch::config::CliOverrides;
use glaunch::history::{
    self, AppHistory, History, LaunchRecord, derive_app_slug, has_overrides, record_launch,
};

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
