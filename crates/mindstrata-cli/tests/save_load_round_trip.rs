//! §16.1 (AP2 §5, Iteration 154): the CLI save→load round trip works
//! end-to-end through the real binary — run a simulation, save a snapshot
//! to disk, then resume from that snapshot.

use std::process::Command;

/// The built binary (cargo sets this for integration tests of binary-only
/// crates).
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_mindstrata")
}

#[test]
fn cli_save_then_load_round_trips_a_snapshot() {
    let path = std::env::temp_dir().join(format!("ms_cli_test_{}.snap", std::process::id()));
    let path_str = path.to_str().expect("temp path is utf-8");

    // Save: run 300 ticks and write a snapshot.
    let save = Command::new(bin())
        .args(["sim", "--seed", "42", "--ticks", "300", "--save-snapshot", path_str])
        .output()
        .expect("the save run executes");
    assert!(save.status.success(), "the save run must exit 0");
    let save_out = String::from_utf8_lossy(&save.stdout);
    assert!(
        save_out.contains("Snapshot saved"),
        "the save run confirms the snapshot: {save_out}"
    );
    assert!(path.exists(), "the snapshot file exists after the save");

    // Load: resume from the saved snapshot for 100 more ticks.
    let load = Command::new(bin())
        .args(["sim", "--seed", "42", "--ticks", "100", "--load-snapshot", path_str])
        .output()
        .expect("the load run executes");
    assert!(load.status.success(), "the load run must exit 0");
    let load_out = String::from_utf8_lossy(&load.stdout);
    assert!(
        load_out.contains("Loaded snapshot from tick 300"),
        "the load run confirms the resume point: {load_out}"
    );

    let _ = std::fs::remove_file(&path);
}
