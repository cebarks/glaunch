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
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("gamemoderun"));
    assert!(stdout.contains("gamescope"));
    assert!(stdout.contains("wine"));
    assert!(stdout.contains("game.exe"));
}

#[test]
fn test_run_dry_run_no_gamescope() {
    let output = glaunch_cmd()
        .args([
            "run",
            "--dry-run",
            "--no-gamescope",
            "--",
            "wine",
            "game.exe",
        ])
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
