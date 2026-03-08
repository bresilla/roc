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

fn run_roc_with_env(workdir: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(roc_bin());
    command.args(args).current_dir(workdir);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("failed to run roc")
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
fn run_json_failure_is_structured() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &["run", "missing_pkg", "missing_exec", "--output", "json"],
    );
    assert_failure(&output, "roc run --output json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\": \"run\""));
    assert!(stdout.contains("\"package\": \"missing_pkg\""));
    assert!(stdout.contains("\"executable\": \"missing_exec\""));
    assert!(stdout.contains("\"status\": \"error\""));
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
fn launch_plain_failure_is_structured() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "launch",
            "missing_pkg",
            "missing.launch.py",
            "--output",
            "plain",
        ],
    );
    assert_failure(&output, "roc launch --output plain");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("launch:"));
    assert!(stdout.contains("package: missing_pkg"));
    assert!(stdout.contains("launch_file: missing.launch.py"));
    assert!(stdout.contains("status: error"));
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
fn topic_pub_json_failure_is_structured() {
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
            "--output",
            "json",
        ],
    );
    assert_failure(&output, "roc topic pub --output json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\": \"topic pub\""));
    assert!(stdout.contains("\"status\": \"error\""));
    assert!(stdout.contains("\"topic\": \"/demo\""));
    assert!(stdout.contains("\"type\": \"missing_pkg/msg/Missing\""));
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

#[test]
fn frame_pub_json_failure_is_structured() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "frame",
            "pub",
            "map",
            "base_link",
            "[0, 0]",
            "[0, 0, 0, 1]",
            "--output",
            "json",
        ],
    );
    assert_failure(&output, "roc frame pub --output json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\": \"frame pub\""));
    assert!(stdout.contains("\"status\": \"error\""));
    assert!(stdout.contains("\"parent\": \"map\""));
    assert!(stdout.contains("\"child\": \"base_link\""));
}

#[test]
fn service_call_failure_prints_request_block() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "service",
            "call",
            "/demo_service",
            "demo_interfaces/srv/Demo",
            "{data: 1}",
        ],
    );
    assert_failure(&output, "roc service call");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Service Call"));
    assert!(stdout.contains("Service"));
    assert!(stdout.contains("/demo_service"));
    assert!(stdout.contains("Type"));
    assert!(stdout.contains("demo_interfaces/srv/Demo"));
    assert!(stdout.contains("Request"));
    assert!(stdout.contains("{data: 1}"));
    assert!(stdout.contains("Command"));
}

#[test]
fn service_call_json_failure_is_structured() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "service",
            "call",
            "/demo_service",
            "demo_interfaces/srv/Demo",
            "{data: 1}",
            "--output",
            "json",
        ],
    );
    assert_failure(&output, "roc service call --output json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\": \"service call\""));
    assert!(stdout.contains("\"service\": \"/demo_service\""));
    assert!(stdout.contains("\"type\": \"demo_interfaces/srv/Demo\""));
    assert!(stdout.contains("\"status\": \"error\""));
}

#[test]
fn action_goal_failure_prints_request_block() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "action",
            "goal",
            "/demo_action",
            "demo_interfaces/action/Demo",
            "{order: 10}",
            "--feedback",
        ],
    );
    assert_failure(&output, "roc action goal");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Action Goal"));
    assert!(stdout.contains("Action"));
    assert!(stdout.contains("/demo_action"));
    assert!(stdout.contains("Type"));
    assert!(stdout.contains("demo_interfaces/action/Demo"));
    assert!(stdout.contains("Goal"));
    assert!(stdout.contains("{order: 10}"));
    assert!(stdout.contains("Feedback"));
    assert!(stdout.contains("Command"));
}

#[test]
fn action_goal_plain_failure_is_structured() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "action",
            "goal",
            "/demo_action",
            "demo_interfaces/action/Demo",
            "{order: 10}",
            "--feedback",
            "--output",
            "plain",
        ],
    );
    assert_failure(&output, "roc action goal --output plain");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("action-goal:"));
    assert!(stdout.contains("action: /demo_action"));
    assert!(stdout.contains("type: demo_interfaces/action/Demo"));
    assert!(stdout.contains("feedback: true"));
    assert!(stdout.contains("status: error"));
}

#[test]
fn daemon_status_json_reports_direct_dds_mode() {
    let temp = tempdir().unwrap();
    let output = run_roc(temp.path(), &["daemon", "status", "--output", "json"]);

    assert!(
        output.status.success(),
        "roc daemon status failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"mode\": \"direct-dds\""));
    assert!(stdout.contains("\"uses_daemon\": false"));
}

#[test]
fn middleware_list_discovers_fake_rmw_libraries() {
    let temp = tempdir().unwrap();
    let prefix = temp.path().join("prefix");
    let lib_dir = prefix.join("lib");
    std::fs::create_dir_all(&lib_dir).unwrap();
    std::fs::write(lib_dir.join("librmw_fastrtps_cpp.so"), "").unwrap();
    std::fs::write(lib_dir.join("librmw_dds_common.so"), "").unwrap();

    let output = run_roc_with_env(
        temp.path(),
        &["middleware", "list", "--output", "json"],
        &[("AMENT_PREFIX_PATH", prefix.to_str().unwrap())],
    );

    assert!(
        output.status.success(),
        "roc middleware list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rmw_fastrtps_cpp"));
    assert!(!stdout.contains("rmw_dds_common"));
}

#[test]
fn middleware_set_plain_prints_shell_export_command() {
    let temp = tempdir().unwrap();
    let output = run_roc(
        temp.path(),
        &[
            "middleware",
            "set",
            "rmw_fastrtps_cpp",
            "--output",
            "plain",
        ],
    );

    assert!(
        output.status.success(),
        "roc middleware set failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "export RMW_IMPLEMENTATION=rmw_fastrtps_cpp");
}
