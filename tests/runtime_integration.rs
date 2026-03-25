use std::process::Command;

fn roc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_roc")
}

fn assert_help_output(args: &[&str], markers: &[&str]) {
    let output = Command::new(roc_bin())
        .args(args)
        .arg("--help")
        .output()
        .unwrap_or_else(|_| panic!("failed to run roc {} --help", args.join(" ")));

    assert!(
        output.status.success(),
        "roc {} --help failed with status {:?}",
        args.join(" "),
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for marker in markers {
        assert!(
            stdout.contains(marker),
            "roc {} --help missing marker {marker:?}\nstdout:\n{}",
            args.join(" "),
            stdout
        );
    }
}

#[test]
fn roc_binary_launches_with_help_output() {
    assert_help_output(&[], &["Usage:", "topic", "work"]);
}

#[test]
fn help_smoke_matrix_covers_core_commands() {
    let matrix = [
        (
            vec!["topic", "list"],
            vec!["Output a list of available topics", "--show-types"],
        ),
        (
            vec!["topic", "echo"],
            vec!["Print messages from topic to screen", "--raw"],
        ),
        (
            vec!["topic", "pub"],
            vec!["Publish a message to a topic", "YAML format"],
        ),
        (
            vec!["work", "build"],
            vec!["Build packages in the workspace", "--merge-install"],
        ),
        (
            vec!["work", "test"],
            vec!["Run tests for packages in the workspace", "--ctest-args"],
        ),
        (
            vec!["work", "test-result"],
            vec![
                "Summarize test results from the workspace build tree",
                "--result-files-only",
            ],
        ),
        (vec!["run"], vec!["Run an executable", "--prefix"]),
        (
            vec!["bag", "info"],
            vec!["Show rosbag2 recording info", "metadata.yaml"],
        ),
    ];

    for (args, markers) in matrix {
        assert_help_output(&args, &markers);
    }
}
