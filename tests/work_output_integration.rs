use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::tempdir;
use walkdir::WalkDir;

fn roc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_roc")
}

fn fixture_workspace(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("workspaces")
        .join(name)
}

fn copy_workspace(src: &Path, dst: &Path) {
    for entry in WalkDir::new(src) {
        let entry = entry.expect("fixture walk failed");
        let path = entry.path();
        let relative = path
            .strip_prefix(src)
            .expect("invalid fixture relative path");
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).expect("failed to create fixture directory");
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).expect("failed to create fixture parent");
            }
            fs::copy(path, target).expect("failed to copy fixture file");
        }
    }
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

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context} unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn work_list_human_output_uses_workspace_table() {
    let temp = tempdir().unwrap();
    copy_workspace(&fixture_workspace("ament_cmake_minimal"), temp.path());

    let output = run_roc(temp.path(), &["work", "list"]);
    assert_success(&output, "roc work list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace Packages"));
    assert!(stdout.contains("Package"));
    assert!(stdout.contains("Build Type"));
    assert!(stdout.contains("demo_cmake_pkg"));
    assert!(stdout.contains("ament_cmake"));
}

#[test]
fn work_info_human_output_uses_section_blocks() {
    let temp = tempdir().unwrap();
    copy_workspace(&fixture_workspace("ament_cmake_minimal"), temp.path());

    let output = run_roc(temp.path(), &["work", "info", "demo_cmake_pkg"]);
    assert_success(&output, "roc work info");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Package"));
    assert!(stdout.contains("Name"));
    assert!(stdout.contains("demo_cmake_pkg"));
    assert!(stdout.contains("Build Type"));
    assert!(stdout.contains("ament_cmake"));
}

#[test]
fn work_build_failure_still_renders_workspace_header() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();

    let output = run_roc(temp.path(), &["work", "build"]);
    assert_failure(&output, "roc work build");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Build"));
    assert!(stdout.contains("Workspace"));
    assert!(stdout.contains("Build Base"));
    assert!(stdout.contains("Install Base"));
    assert!(stdout.contains("Log Base"));
    assert!(stderr.contains("No ROS packages found"));
}

#[test]
fn work_test_failure_still_renders_workspace_header() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();

    let output = run_roc(temp.path(), &["work", "test"]);
    assert_failure(&output, "roc work test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Test"));
    assert!(stdout.contains("Workspace"));
    assert!(stdout.contains("Build Base"));
    assert!(stdout.contains("Install Base"));
    assert!(stdout.contains("Log Base"));
    assert!(stderr.contains("No ROS packages found"));
}

#[test]
fn work_test_result_human_output_uses_summary_sections() {
    let temp = tempdir().unwrap();
    let package_dir = temp
        .path()
        .join("build")
        .join("demo_pkg")
        .join("test_results");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        temp.path()
            .join("build")
            .join("demo_pkg")
            .join("colcon_test.rc"),
        "1\n",
    )
    .unwrap();
    fs::write(
        package_dir.join("pytest.xml"),
        r#"<testsuite tests="1" failures="1" errors="0" skipped="0"><testcase classname="demo_pkg.tests" name="test_demo"><failure message="assert False">boom</failure></testcase></testsuite>"#,
    )
    .unwrap();

    let output = run_roc(
        temp.path(),
        &["work", "test-result", "--test-result-base", "build"],
    );
    assert_success(&output, "roc work test-result");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test Results"));
    assert!(stdout.contains("Result Summary"));
    assert!(stdout.contains("Results"));
    assert!(stdout.contains("demo_pkg"));
    assert!(stdout.contains("failed"));
}

#[test]
fn work_test_result_verbose_output_prints_totals_and_failure_details() {
    let temp = tempdir().unwrap();
    let package_dir = temp
        .path()
        .join("build")
        .join("demo_pkg")
        .join("test_results");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        temp.path()
            .join("build")
            .join("demo_pkg")
            .join("colcon_test.rc"),
        "1\n",
    )
    .unwrap();
    fs::write(
        package_dir.join("pytest.xml"),
        r#"<testsuite tests="2" failures="1" errors="0" skipped="1"><testcase classname="demo_pkg.tests" name="test_demo"><failure message="assert False">boom</failure></testcase></testsuite>"#,
    )
    .unwrap();

    let output = run_roc(
        temp.path(),
        &[
            "work",
            "test-result",
            "--test-result-base",
            "build",
            "--all",
            "--verbose",
        ],
    );
    assert_success(&output, "roc work test-result --verbose");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Package Totals"));
    assert!(stdout.contains("Tests"));
    assert!(stdout.contains("Failures"));
    assert!(stdout.contains("demo_pkg"));
    assert!(stderr.contains("Failure Details"));
    assert!(stderr.contains("test_demo"));
    assert!(stderr.contains("boom"));
}
