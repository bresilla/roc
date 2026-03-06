use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;
use walkdir::WalkDir;

fn fixture_workspace(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("workspaces")
        .join(name)
}

fn copy_workspace(src: &Path, dst: &Path) {
    for entry in WalkDir::new(src) {
        let entry = entry.unwrap();
        let path = entry.path();
        let relative = path.strip_prefix(src).unwrap();
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).unwrap();
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(path, target).unwrap();
        }
    }
}

fn command_exists(name: &str) -> bool {
    Command::new("bash")
        .args(["-lc", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_shell(workdir: &Path, command: &str) -> std::process::Output {
    Command::new("bash")
        .args(["-lc", command])
        .current_dir(workdir)
        .output()
        .unwrap()
}

fn assert_success(output: std::process::Output, context: &str) {
    if output.status.success() {
        return;
    }

    panic!(
        "{context} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires colcon and a sourced ROS 2 environment"]
fn validate_ament_cmake_workspace_against_colcon() {
    if !command_exists("colcon") || !Path::new("/opt/ros/jazzy/setup.bash").exists() {
        return;
    }

    let temp = tempdir().unwrap();
    let colcon_ws = temp.path().join("colcon_ws");
    let roc_ws = temp.path().join("roc_ws");
    copy_workspace(&fixture_workspace("ament_cmake_minimal"), &colcon_ws);
    copy_workspace(&fixture_workspace("ament_cmake_minimal"), &roc_ws);

    assert_success(
        run_shell(
            &colcon_ws,
            "source /opt/ros/jazzy/setup.bash && colcon build --base-paths src",
        ),
        "colcon build for ament_cmake fixture",
    );
    assert_success(
        run_shell(
            &roc_ws,
            &format!(
                "source /opt/ros/jazzy/setup.bash && {} work build --base-paths src",
                env!("CARGO_BIN_EXE_roc")
            ),
        ),
        "roc build for ament_cmake fixture",
    );

    let colcon_prefix = run_shell(
        &colcon_ws,
        "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix demo_cmake_pkg",
    );
    assert_success(
        colcon_prefix,
        "colcon ros2 pkg prefix for ament_cmake fixture",
    );

    let roc_prefix = run_shell(
        &roc_ws,
        "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix demo_cmake_pkg",
    );
    assert_success(roc_prefix, "roc ros2 pkg prefix for ament_cmake fixture");
}

#[test]
#[ignore = "requires colcon and a sourced ROS 2 environment"]
fn validate_ament_python_workspace_runtime() {
    if !command_exists("colcon") || !Path::new("/opt/ros/jazzy/setup.bash").exists() {
        return;
    }

    let temp = tempdir().unwrap();
    let colcon_ws = temp.path().join("colcon_ws");
    let roc_ws = temp.path().join("roc_ws");
    copy_workspace(&fixture_workspace("ament_python_minimal"), &colcon_ws);
    copy_workspace(&fixture_workspace("ament_python_minimal"), &roc_ws);

    assert_success(
        run_shell(
            &colcon_ws,
            "source /opt/ros/jazzy/setup.bash && colcon build --base-paths src",
        ),
        "colcon build for ament_python fixture",
    );
    assert_success(
        run_shell(
            &roc_ws,
            &format!(
                "source /opt/ros/jazzy/setup.bash && {} work build --base-paths src",
                env!("CARGO_BIN_EXE_roc")
            ),
        ),
        "roc build for ament_python fixture",
    );

    let colcon_import = run_shell(
        &colcon_ws,
        "source /opt/ros/jazzy/setup.bash && source install/setup.bash && python3 -c \"import demo_python_pkg\"",
    );
    assert_success(
        colcon_import,
        "colcon python import for ament_python fixture",
    );

    let roc_import = run_shell(
        &roc_ws,
        "source /opt/ros/jazzy/setup.bash && source install/setup.bash && python3 -c \"import demo_python_pkg\"",
    );
    assert_success(roc_import, "roc python import for ament_python fixture");
}
