use std::process::{Command, Output};

fn roc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_roc")
}

fn run_roc(args: &[&str]) -> Output {
    Command::new(roc_bin())
        .args(args)
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
fn top_level_help_uses_current_product_language() {
    let output = run_roc(&["--help"]);
    assert_success(&output, "roc --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Utilities Commands"));
    assert!(stdout.contains("Transform tree inspection and publishing"));
    assert!(stdout.contains("IDL conversion and discovery tools"));
    assert!(!stdout.contains("wannabe"));
    assert!(!stdout.contains("replacer"));
}

#[test]
fn frame_help_no_longer_marks_command_as_wip() {
    let output = run_roc(&["frame", "--help"]);
    assert_success(&output, "roc frame --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Transform tree inspection and publishing"));
    assert!(!stdout.contains("[WIP]"));
}

#[test]
fn daemon_help_lists_implemented_subcommands() {
    let output = run_roc(&["daemon", "--help"]);
    assert_success(&output, "roc daemon --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("start"));
    assert!(stdout.contains("stop"));
    assert!(stdout.contains("status"));
}

#[test]
fn middleware_help_lists_implemented_subcommands() {
    let output = run_roc(&["middleware", "--help"]);
    assert_success(&output, "roc middleware --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("get"));
    assert!(stdout.contains("set"));
}
