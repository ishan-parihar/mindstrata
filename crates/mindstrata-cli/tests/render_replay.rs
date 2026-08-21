//! §20 (AP2 Phase 5, Iteration 171): Replay visualizations — the CLI
//! `--render-replay` flag produces a real, valid animated GIF sampled from
//! the live simulation through the actual binary.

use std::process::Command;

/// The built binary (cargo sets this for integration tests of binary-only
/// crates).
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_mindstrata")
}

/// GIF signatures: both 87a and 89a are valid.
fn is_gif_signature(bytes: &[u8]) -> bool {
    bytes.len() >= 6 && (bytes[..6] == *b"GIF87a" || bytes[..6] == *b"GIF89a")
}

#[test]
fn cli_render_replay_writes_a_valid_animated_gif() {
    let path = std::env::temp_dir().join(format!("ms_cli_replay_{}.gif", std::process::id()));
    let path_str = path.to_str().expect("temp path is utf-8");

    // 200 ticks at 24-tick cadence = frame 0 + 8 sampled frames = 9 frames.
    let out = Command::new(bin())
        .args([
            "sim",
            "--seed",
            "42",
            "--ticks",
            "200",
            "--agents",
            "12",
            "--render-replay",
            path_str,
        ])
        .output()
        .expect("the replay run executes");
    assert!(out.status.success(), "the replay run must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Animated replay") && stdout.contains("written to"),
        "the run confirms the replay write: {stdout}"
    );
    assert!(
        stdout.contains("9 frames"),
        "frame-0 + ticks sampled every 24 = 9 frames: {stdout}"
    );

    // The file exists and is a GIF. The logical screen size is declared in
    // the header: 16x16 world at 12px/cell = 192x192, and the stream ends
    // with the GIF trailer byte 0x3B.
    let bytes = std::fs::read(&path).expect("the replay GIF exists on disk");
    assert!(is_gif_signature(&bytes), "output must be a GIF file");
    let width = u16::from_le_bytes([bytes[6], bytes[7]]);
    let height = u16::from_le_bytes([bytes[8], bytes[9]]);
    assert_eq!((width, height), (192, 192), "16x16 world at 12px/cell");
    assert!(
        bytes.ends_with(&[0x3B]),
        "GIF stream must end with the trailer"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_render_replay_obeys_custom_cadence() {
    let path = std::env::temp_dir().join(format!("ms_cli_replay_cad_{}.gif", std::process::id()));
    let path_str = path.to_str().expect("temp path is utf-8");

    // 60 ticks at 30-tick cadence = frame 0 + 2 sampled frames = 3 frames.
    let out = Command::new(bin())
        .args([
            "sim",
            "--seed",
            "1",
            "--ticks",
            "60",
            "--agents",
            "6",
            "--render-replay",
            path_str,
            "--replay-every",
            "30",
        ])
        .output()
        .expect("the replay run executes");
    assert!(out.status.success(), "the replay run must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("3 frames"),
        "frame-0 + 30/60 cadence = 3 frames: {stdout}"
    );
    let bytes = std::fs::read(&path).expect("the replay GIF exists on disk");
    assert!(is_gif_signature(&bytes), "output must be a GIF file");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_render_replay_reports_missing_parent_directory() {
    // A nonexistent parent directory must not panic the binary — it reports
    // the write failure and still exits cleanly.
    let missing = std::env::temp_dir()
        .join(format!("ms_cli_no_such_replay_{}", std::process::id()))
        .join("replay.gif");
    let missing_str = missing.to_str().expect("temp path is utf-8");

    let out = Command::new(bin())
        .args([
            "sim",
            "--seed",
            "42",
            "--ticks",
            "20",
            "--agents",
            "6",
            "--render-replay",
            missing_str,
        ])
        .output()
        .expect("the replay run executes");
    assert!(out.status.success(), "a write failure must still exit 0");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Failed to write replay GIF"),
        "the failure is reported: {stderr}"
    );
}
