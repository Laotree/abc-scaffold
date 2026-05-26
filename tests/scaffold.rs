//! Integration tests for the scaffold function.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the built abc-init binary, set by Cargo during `cargo test`.
const ABC_INIT: &str = env!("CARGO_BIN_EXE_abc-init");

/// Run the fully non-interactive path: `abc-init <name> --lang <lang> --yes`
/// inside the given directory and return the output.
fn run_scaffold(dir: &Path, name: &str, lang: &str) -> std::process::Output {
    Command::new(ABC_INIT)
        .args([name, "--lang", lang, "--yes"])
        .current_dir(dir)
        .output()
        .expect("failed to run abc-init")
}

#[test]
fn scaffold_rust() {
    let dir = tempdir();
    let output = run_scaffold(&dir, "my-rust-app", "rust");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let project = dir.join("my-rust-app");
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("src/main.rs").exists());
    assert!(project.join("CLAUDE.md").exists());
    assert!(project.join("README.md").exists());
    assert!(project.join(".gitignore").exists());
    assert!(project.join("Makefile").exists());
    assert!(project.join("hooks/pre-push").exists());
    assert!(project.join(".git").exists());

    // Verify it's a valid Cargo project
    let check = Command::new("cargo")
        .args(["check"])
        .current_dir(&project)
        .output()
        .expect("cargo check failed");
    assert!(check.status.success(), "cargo check failed: {}", String::from_utf8_lossy(&check.stderr));
}

#[test]
fn scaffold_go() {
    let dir = tempdir();
    let output = run_scaffold(&dir, "my-go-app", "go");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let project = dir.join("my-go-app");
    assert!(project.join("go.mod").exists());
    assert!(project.join("main.go").exists());
    assert!(project.join("CLAUDE.md").exists());
    assert!(project.join(".git").exists());
}

#[test]
fn scaffold_python() {
    let dir = tempdir();
    let output = run_scaffold(&dir, "my-py-app", "python");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let project = dir.join("my-py-app");
    assert!(project.join("pyproject.toml").exists());
    assert!(project.join("main.py").exists());
    assert!(project.join("CLAUDE.md").exists());
    assert!(project.join(".git").exists());
}

#[test]
fn scaffold_typescript() {
    let dir = tempdir();
    let output = run_scaffold(&dir, "my-ts-app", "typescript");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let project = dir.join("my-ts-app");
    assert!(project.join("package.json").exists());
    assert!(project.join("tsconfig.json").exists());
    assert!(project.join("src/index.ts").exists());
    assert!(project.join("CLAUDE.md").exists());
    assert!(project.join(".git").exists());
}

#[test]
fn scaffold_existing_directory_errors() {
    let dir = tempdir();
    let name = "existing-dir";
    fs::create_dir(dir.join(name)).expect("create dir");

    let output = run_scaffold(&dir, name, "rust");
    assert!(!output.status.success(), "expected failure for existing dir");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "stderr: {stderr}");
}

#[test]
fn scaffold_with_slash_creates_nested_directory() {
    let dir = tempdir();
    // A name with a slash creates nested directories — this is valid
    // filesystem behavior (the TUI name-input screen has its own validation).
    let output = run_scaffold(&dir, "my/app", "rust");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.join("my/app/Cargo.toml").exists());
    assert!(dir.join("my/app/.git").exists());
}

/// Create a temporary directory that is automatically cleaned up on drop.
fn tempdir() -> PathBuf {
    let base = std::env::temp_dir().join("abc-scaffold-test");
    // Use a counter to get unique dirs
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = base.join(format!("test-{ts}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}
