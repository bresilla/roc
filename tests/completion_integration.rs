use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

fn roc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_roc")
}

fn shell_exists(shell: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {shell} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn generate_completion_script(shell: &str, dir: &Path) -> std::path::PathBuf {
    let output = Command::new(roc_bin())
        .args(["completion", shell])
        .output()
        .expect("failed to run roc completion");
    assert!(output.status.success(), "roc completion {shell} failed");

    let path = dir.join(format!("roc.{shell}"));
    fs::write(&path, output.stdout).expect("failed to write completion script");
    path
}

#[test]
fn bash_completion_script_handles_work_build_flags_end_to_end() {
    if !shell_exists("bash") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("bash", temp.path());
    let output = Command::new("bash")
        .env("ROC_BIN", roc_bin())
        .env("ROC_COMPLETION_SCRIPT", &script)
        .arg("--noprofile")
        .arg("--norc")
        .arg("-c")
        .arg(
            r#"
roc() { "$ROC_BIN" "$@"; }
_init_completion() {
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    words=("${COMP_WORDS[@]}")
    cword=$COMP_CWORD
}
source "$ROC_COMPLETION_SCRIPT"
COMP_WORDS=(roc work build --)
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--merge-install"));
    assert!(stdout.contains("--packages-select"));
}

#[test]
fn bash_completion_script_handles_work_test_result_flags_end_to_end() {
    if !shell_exists("bash") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("bash", temp.path());
    let output = Command::new("bash")
        .env("ROC_BIN", roc_bin())
        .env("ROC_COMPLETION_SCRIPT", &script)
        .arg("--noprofile")
        .arg("--norc")
        .arg("-c")
        .arg(
            r#"
roc() { "$ROC_BIN" "$@"; }
_init_completion() {
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    words=("${COMP_WORDS[@]}")
    cword=$COMP_CWORD
}
source "$ROC_COMPLETION_SCRIPT"
COMP_WORDS=(roc work test-result --)
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--result-files-only"));
    assert!(stdout.contains("--delete-yes"));
}

#[test]
fn bash_completion_script_handles_param_import_flags_end_to_end() {
    if !shell_exists("bash") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("bash", temp.path());
    let output = Command::new("bash")
        .env("ROC_BIN", roc_bin())
        .env("ROC_COMPLETION_SCRIPT", &script)
        .arg("--noprofile")
        .arg("--norc")
        .arg("-c")
        .arg(
            r#"
roc() { "$ROC_BIN" "$@"; }
_init_completion() {
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    words=("${COMP_WORDS[@]}")
    cword=$COMP_CWORD
}
source "$ROC_COMPLETION_SCRIPT"
COMP_WORDS=(roc param import --)
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--no-use-wildcard"));
    assert!(stdout.contains("--output"));
}

#[test]
fn fish_completion_script_handles_work_build_flags_end_to_end() {
    if !shell_exists("fish") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("fish", temp.path());
    let output = Command::new("fish")
        .env("ROC_BIN", roc_bin())
        .arg("-c")
        .arg(format!(
            r#"
function roc
    $ROC_BIN $argv
end
source {}
complete --do-complete "roc work build --"
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--merge-install"));
    assert!(stdout.contains("--packages-select"));
}

#[test]
fn fish_completion_script_handles_param_export_flags_end_to_end() {
    if !shell_exists("fish") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("fish", temp.path());
    let output = Command::new("fish")
        .env("ROC_BIN", roc_bin())
        .arg("-c")
        .arg(format!(
            r#"
function roc
    $ROC_BIN $argv
end
source {}
complete --do-complete "roc param export --"
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--output-dir"));
    assert!(stdout.contains("--output"));
}

#[test]
fn fish_completion_script_handles_work_test_result_flags_end_to_end() {
    if !shell_exists("fish") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("fish", temp.path());
    let output = Command::new("fish")
        .env("ROC_BIN", roc_bin())
        .arg("-c")
        .arg(format!(
            r#"
function roc
    $ROC_BIN $argv
end
source {}
complete --do-complete "roc work test-result --"
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--result-files-only"));
    assert!(stdout.contains("--delete-yes"));
}

#[test]
fn zsh_completion_script_sources_and_uses_dynamic_helper() {
    if !shell_exists("zsh") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("zsh", temp.path());
    let output = Command::new("zsh")
        .env("ROC_BIN", roc_bin())
        .arg("-fc")
        .arg(format!(
            r#"
autoload -Uz compinit
compinit
roc() {{ "$ROC_BIN" "$@"; }}
source {}
_roc_dynamic_lines roc _complete work '' '' 1
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute zsh completion test");

    assert!(output.status.success(), "zsh completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"));
    assert!(stdout.contains("info"));
    assert!(stdout.contains("test-result"));
}
