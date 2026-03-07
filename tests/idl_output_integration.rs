use std::fs;
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

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn ros2msg_verbose_dry_run_uses_structured_output() {
    let temp = tempdir().unwrap();
    let msg_file = temp.path().join("Example.msg");
    fs::write(&msg_file, "string data\nint32 count\n").unwrap();

    let output = run_roc(
        temp.path(),
        &[
            "idl",
            "ros2msg",
            msg_file.to_str().unwrap(),
            "--dry-run",
            "--verbose",
        ],
    );
    assert_success(&output, "roc idl ros2msg");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ROS 2 Message Conversion"));
    assert!(stdout.contains("Mode"));
    assert!(stdout.contains("msg -> proto"));
    assert!(stdout.contains("Generated Content"));
    assert!(stdout.contains("message Example"));
}

#[test]
fn protobuf_discovery_verbose_on_empty_workspace_prints_guidance() {
    let temp = tempdir().unwrap();

    let output = run_roc(
        temp.path(),
        &[
            "idl",
            "protobuf",
            "--discover",
            "--search-root",
            temp.path().to_str().unwrap(),
            "--verbose",
        ],
    );
    assert_success(&output, "roc idl protobuf --discover");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("IDL Discovery Conversion"));
    assert!(stderr.contains("No ROS 2 packages with IDL files found"));
}

#[test]
fn protobuf_verbose_dry_run_uses_structured_output() {
    let temp = tempdir().unwrap();
    let proto_file = temp.path().join("example.proto");
    fs::write(
        &proto_file,
        r#"syntax = "proto3";

message Example {
  string data = 1;
}
"#,
    )
    .unwrap();

    let output = run_roc(
        temp.path(),
        &[
            "idl",
            "protobuf",
            proto_file.to_str().unwrap(),
            "--dry-run",
            "--verbose",
        ],
    );
    assert_success(&output, "roc idl protobuf");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("IDL Conversion"));
    assert!(stdout.contains("Direction"));
    assert!(stdout.contains(".proto -> .msg"));
    assert!(stdout.contains("Generated Content"));
    assert!(stdout.contains("string data"));
}
