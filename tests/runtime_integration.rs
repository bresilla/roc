use std::process::Command;

fn roc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_roc")
}

#[test]
fn roc_binary_launches_with_help_output() {
    let output = Command::new(roc_bin())
        .arg("--help")
        .output()
        .expect("failed to run roc --help");

    assert!(output.status.success(), "roc --help failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("topic"));
    assert!(stdout.contains("work"));
}
