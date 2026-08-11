//! §5 (AP2 Phase 4, Iteration 157): Visual rendering — the CLI
//! `--render-map` flag produces a real, valid PNG of the final world state
//! through the actual binary.

use std::process::Command;

/// The built binary (cargo sets this for integration tests of binary-only
/// crates).
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_mindstrata")
}

/// PNG signature: 89 50 4E 47 0D 0A 1A 0A.
const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[test]
fn cli_render_map_writes_a_valid_png() {
    let path = std::env::temp_dir().join(format!("ms_cli_map_{}.png", std::process::id()));
    let path_str = path.to_str().expect("temp path is utf-8");

    let out = Command::new(bin())
        .args([
            "sim",
            "--seed",
            "42",
            "--ticks",
            "200",
            "--agents",
            "12",
            "--render-map",
            path_str,
        ])
        .output()
        .expect("the render run executes");
    assert!(out.status.success(), "the render run must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Rendered world map"),
        "the run confirms the render: {stdout}"
    );

    // The file exists, starts with the PNG signature, and has the expected
    // dimensions. Coupling note: the CLI's SimConfig world is 16x16 and
    // DEFAULT_CELL_PIXELS is 12, so the image is 192x192 — update this pin
    // if either changes. IHDR width/height live at bytes 16..24 (8-byte
    // signature + 4-byte length + 4-byte "IHDR" type).
    let bytes = std::fs::read(&path).expect("the rendered PNG exists on disk");
    assert!(
        bytes.len() >= 24 && bytes[..8] == PNG_SIGNATURE,
        "output must be a PNG file"
    );
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    assert_eq!((width, height), (192, 192), "16x16 world at 12px/cell");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_render_map_reports_missing_parent_directory() {
    // A nonexistent parent directory must not panic the binary — it reports
    // the write failure and still exits cleanly.
    let missing = std::env::temp_dir()
        .join(format!("ms_cli_no_such_dir_{}", std::process::id()))
        .join("map.png");
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
            "--render-map",
            missing_str,
        ])
        .output()
        .expect("the render run executes");
    assert!(out.status.success(), "a write failure must still exit 0");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Failed to write rendered map"),
        "the failure is reported: {stderr}"
    );
}
