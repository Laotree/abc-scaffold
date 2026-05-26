//! Smoke tests for the abc-init CLI binary.

use std::process::Command;

/// Path to the built abc-init binary, set by Cargo during `cargo test`.
const ABC_INIT: &str = env!("CARGO_BIN_EXE_abc-init");

#[test]
fn help_exits_cleanly() {
    let output = Command::new(ABC_INIT)
        .arg("--help")
        .output()
        .expect("failed to run abc-init --help");
    assert!(output.status.success(), "exit: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("abc-init"));
    assert!(stdout.contains("Usage"));
}

#[test]
fn version_exits_cleanly() {
    let output = Command::new(ABC_INIT)
        .arg("--version")
        .output()
        .expect("failed to run abc-init --version");
    assert!(output.status.success(), "exit: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("abc-init"));
}

#[test]
fn unknown_flag_shows_error() {
    let output = Command::new(ABC_INIT)
        .arg("--bogus")
        .output()
        .expect("failed to run abc-init --bogus");
    assert!(!output.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}
