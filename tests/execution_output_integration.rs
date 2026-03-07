use std::path::Path;
use std::process::{Command, Output};

use tempfile::tempdir;

fn roc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_roc")
}

fn run_roc(workdir: &Path, args: &[&str]) -> Output {
    Command::new(roc_bin())
        .args(args)
        .current_dir(workdir)
        .output()
        .expect("failed to run roc")
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context} unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_failure_prints_execution_block() {
    let temp = tempdir().unwrap();
    let output = run_roc(temp.path(), &["run", "missing_pkg", "missing_exec"]);
    assert_failure(&output, "roc run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Run"));
    assert!(stdout.contains("Package"));
    assert!(stdout.contains("Executable"));
    assert!(stdout.contains("missing_pkg"));
    assert!(stdout.contains("missing_exec"));
    assert!(stderr.contains("Executable"));
}

#[test]
fn launch_failure_prints_execution_block() {
    let temp = tempdir().unwrap();
    let output = run_roc(temp.path(), &["launch", "missing_pkg", "missing.launch.py"]);
    assert_failure(&output, "roc launch");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Launch"));
    assert!(stdout.contains("Package"));
    assert!(stdout.contains("Launch File"));
    assert!(stdout.contains("missing_pkg"));
    assert!(stdout.contains("missing.launch.py"));
    assert!(stderr.contains("Launch file"));
}

#[test]
fn topic_pub_failure_prints_publish_block_before_runtime_error() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "topic",
            "pub",
            "/demo",
            "missing_pkg/msg/Missing",
            "data: hello",
            "--once",
        ],
    );
    assert_failure(&output, "roc topic pub");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Topic Publish"));
    assert!(stdout.contains("Topic"));
    assert!(stdout.contains("/demo"));
    assert!(stdout.contains("Type"));
    assert!(stdout.contains("missing_pkg/msg/Missing"));
    assert!(stdout.contains("Mode"));
}

#[test]
fn frame_pub_detach_prints_publish_block() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "frame",
            "pub",
            "map",
            "base_link",
            "[0, 0, 0]",
            "[0, 0, 0, 1]",
            "--detach",
        ],
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Frame Publish"));
    assert!(stdout.contains("Parent"));
    assert!(stdout.contains("Child"));
    assert!(stdout.contains("map"));
    assert!(stdout.contains("base_link"));
    assert!(stdout.contains("Mode"));
}
