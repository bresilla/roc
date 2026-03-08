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

fn shell_install_path(shell: &str, home: &Path) -> std::path::PathBuf {
    let output = Command::new(roc_bin())
        .env("HOME", home)
        .args(["completion", shell, "--print-path"])
        .output()
        .expect("failed to run roc completion --print-path");
    assert!(
        output.status.success(),
        "roc completion {shell} --print-path failed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    std::path::PathBuf::from(stdout.trim())
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
fn bash_completion_script_handles_launch_flags_end_to_end() {
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
COMP_WORDS=(roc launch --)
COMP_CWORD=2
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--launch_prefix"));
    assert!(stdout.contains("--output"));
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
fn bash_completion_script_handles_daemon_flags_end_to_end() {
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
COMP_WORDS=(roc daemon status --)
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--output"));
}

#[test]
fn bash_completion_script_handles_service_call_flags_end_to_end() {
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
COMP_WORDS=(roc service call --)
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--rate"));
    assert!(stdout.contains("--output"));
}

#[test]
fn bash_completion_script_handles_completion_flags_end_to_end() {
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
COMP_WORDS=(roc completion bash --)
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--install"));
    assert!(stdout.contains("--print-path"));
}

#[test]
fn bash_completion_script_handles_idl_protobuf_flags_end_to_end() {
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
COMP_WORDS=(roc idl protobuf --)
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--search-root"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn bash_completion_script_handles_idl_file_completion_end_to_end() {
    if !shell_exists("bash") {
        return;
    }

    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("msg")).unwrap();
    fs::create_dir_all(temp.path().join("proto")).unwrap();
    fs::write(temp.path().join("msg/Example.msg"), "string data\n").unwrap();
    fs::write(
        temp.path().join("proto/example.proto"),
        "syntax = \"proto3\";\n",
    )
    .unwrap();

    let script = generate_completion_script("bash", temp.path());
    let output = Command::new("bash")
        .current_dir(temp.path())
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
COMP_WORDS=(roc idl protobuf "")
COMP_CWORD=3
_roc_completion
printf '%s\n' "${COMPREPLY[@]}"
"#,
        )
        .output()
        .expect("failed to execute bash completion test");

    assert!(output.status.success(), "bash completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("msg/Example.msg"));
    assert!(stdout.contains("proto/example.proto"));
}

#[test]
fn completion_print_path_prefers_user_locations() {
    let temp = tempdir().unwrap();

    let bash_path = shell_install_path("bash", temp.path());
    let zsh_path = shell_install_path("zsh", temp.path());
    let fish_path = shell_install_path("fish", temp.path());

    assert!(bash_path.starts_with(temp.path()));
    assert!(bash_path.ends_with(".local/share/bash-completion/completions/roc"));
    assert!(zsh_path.starts_with(temp.path()));
    assert!(zsh_path.ends_with(".zfunc/_roc"));
    assert!(fish_path.starts_with(temp.path()));
    assert!(fish_path.ends_with(".config/fish/completions/roc.fish"));
}

#[test]
fn completion_install_writes_script_into_printed_path() {
    let temp = tempdir().unwrap();
    let path = shell_install_path("bash", temp.path());

    let output = Command::new(roc_bin())
        .env("HOME", temp.path())
        .args(["completion", "bash", "--install"])
        .output()
        .expect("failed to run roc completion bash --install");
    assert!(
        output.status.success(),
        "roc completion bash --install failed"
    );

    let installed = fs::read_to_string(&path).expect("completion script should be installed");
    assert!(installed.contains("_roc_completion"));
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
fn fish_completion_script_handles_middleware_flags_end_to_end() {
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
complete --do-complete "roc middleware set --"
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--output"));
}

#[test]
fn fish_completion_script_handles_idl_ros2msg_flags_end_to_end() {
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
complete --do-complete "roc idl ros2msg --"
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--dry-run"));
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
fn fish_completion_script_handles_run_flags_end_to_end() {
    if !shell_exists("fish") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("fish", temp.path());
    let output = Command::new("fish")
        .env("ROC_COMPLETION_SCRIPT", &script)
        .arg("-c")
        .arg(
            r#"
source $ROC_COMPLETION_SCRIPT
complete --do-complete "roc run --"
"#,
        )
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--prefix"));
    assert!(stdout.contains("--output"));
}

#[test]
fn fish_completion_script_handles_frame_pub_flags_end_to_end() {
    if !shell_exists("fish") {
        return;
    }

    let temp = tempdir().unwrap();
    let script = generate_completion_script("fish", temp.path());
    let output = Command::new("fish")
        .env("ROC_COMPLETION_SCRIPT", &script)
        .arg("-c")
        .arg(
            r#"
source $ROC_COMPLETION_SCRIPT
complete --do-complete "roc frame pub --"
"#,
        )
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--detach"));
    assert!(stdout.contains("--output"));
}

#[test]
fn fish_completion_script_handles_completion_flags_end_to_end() {
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
complete --do-complete "roc completion bash --"
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute fish completion test");

    assert!(output.status.success(), "fish completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--install"));
    assert!(stdout.contains("--print-path"));
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

#[test]
fn zsh_completion_script_exposes_idl_subcommands() {
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
_roc_dynamic_lines roc _complete idl '' '' 1
"#,
            script.display()
        ))
        .output()
        .expect("failed to execute zsh completion test");

    assert!(output.status.success(), "zsh completion probe failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("protobuf"));
    assert!(stdout.contains("ros2msg"));
}
