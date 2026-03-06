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

fn assert_failure(output: std::process::Output, context: &str) {
    if !output.status.success() {
        return;
    }

    panic!(
        "{context} unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_stdout_equals(output: std::process::Output, expected: &str, context: &str) {
    assert_success(output.clone(), context);
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(actual, expected, "{context} produced unexpected stdout");
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

    let colcon_prefix = run_shell(
        &colcon_ws,
        "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix demo_python_pkg",
    );
    assert_success(
        colcon_prefix,
        "colcon ros2 pkg prefix for ament_python fixture",
    );

    let roc_prefix = run_shell(
        &roc_ws,
        "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix demo_python_pkg",
    );
    assert_success(roc_prefix, "roc ros2 pkg prefix for ament_python fixture");
}

#[test]
#[ignore = "requires colcon and a sourced ROS 2 environment"]
fn validate_merged_install_workspace_against_colcon() {
    if !command_exists("colcon") || !Path::new("/opt/ros/jazzy/setup.bash").exists() {
        return;
    }

    let temp = tempdir().unwrap();
    let colcon_ws = temp.path().join("colcon_ws");
    let roc_ws = temp.path().join("roc_ws");
    copy_workspace(&fixture_workspace("dependency_chain"), &colcon_ws);
    copy_workspace(&fixture_workspace("dependency_chain"), &roc_ws);

    assert_success(
        run_shell(
            &colcon_ws,
            "source /opt/ros/jazzy/setup.bash && colcon build --merge-install --base-paths src",
        ),
        "colcon merged build for dependency chain fixture",
    );
    assert_success(
        run_shell(
            &roc_ws,
            &format!(
                "source /opt/ros/jazzy/setup.bash && {} work build --merge-install --base-paths src",
                env!("CARGO_BIN_EXE_roc")
            ),
        ),
        "roc merged build for dependency chain fixture",
    );

    assert_eq!(
        fs::read_to_string(colcon_ws.join("install/.colcon_install_layout"))
            .unwrap()
            .trim(),
        "merged"
    );
    assert_eq!(
        fs::read_to_string(roc_ws.join("install/.colcon_install_layout"))
            .unwrap()
            .trim(),
        "merged"
    );

    assert_stdout_equals(
        run_shell(
            &colcon_ws,
            "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix consumer_node_pkg",
        ),
        colcon_ws.join("install").to_string_lossy().as_ref(),
        "colcon merged ros2 pkg prefix for dependency chain fixture",
    );
    assert_stdout_equals(
        run_shell(
            &roc_ws,
            "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix consumer_node_pkg",
        ),
        roc_ws.join("install").to_string_lossy().as_ref(),
        "roc merged ros2 pkg prefix for dependency chain fixture",
    );
}

#[test]
#[ignore = "requires colcon and a sourced ROS 2 environment"]
fn validate_overlay_workspace_chaining_against_colcon() {
    if !command_exists("colcon") || !Path::new("/opt/ros/jazzy/setup.bash").exists() {
        return;
    }

    let temp = tempdir().unwrap();
    let colcon_underlay = temp.path().join("colcon_underlay");
    let colcon_overlay = temp.path().join("colcon_overlay");
    let roc_underlay = temp.path().join("roc_underlay");
    let roc_overlay = temp.path().join("roc_overlay");

    copy_workspace(&fixture_workspace("ament_cmake_minimal"), &colcon_underlay);
    copy_workspace(&fixture_workspace("ament_python_minimal"), &colcon_overlay);
    copy_workspace(&fixture_workspace("ament_cmake_minimal"), &roc_underlay);
    copy_workspace(&fixture_workspace("ament_python_minimal"), &roc_overlay);

    assert_success(
        run_shell(
            &colcon_underlay,
            "source /opt/ros/jazzy/setup.bash && colcon build --base-paths src",
        ),
        "colcon underlay build",
    );
    assert_success(
        run_shell(
            &roc_underlay,
            &format!(
                "source /opt/ros/jazzy/setup.bash && {} work build --base-paths src",
                env!("CARGO_BIN_EXE_roc")
            ),
        ),
        "roc underlay build",
    );

    assert_success(
        run_shell(
            &colcon_overlay,
            &format!(
                "source /opt/ros/jazzy/setup.bash && source {}/install/setup.bash && colcon build --base-paths src",
                colcon_underlay.display()
            ),
        ),
        "colcon overlay build",
    );
    assert_success(
        run_shell(
            &roc_overlay,
            &format!(
                "source /opt/ros/jazzy/setup.bash && source {}/install/setup.bash && {} work build --base-paths src",
                roc_underlay.display(),
                env!("CARGO_BIN_EXE_roc")
            ),
        ),
        "roc overlay build",
    );

    assert_stdout_equals(
        run_shell(
            &colcon_overlay,
            &format!(
                "source /opt/ros/jazzy/setup.bash && source {}/install/setup.bash && source install/setup.bash && ros2 pkg prefix demo_cmake_pkg",
                colcon_underlay.display()
            ),
        ),
        colcon_underlay
            .join("install/demo_cmake_pkg")
            .to_string_lossy()
            .as_ref(),
        "colcon overlay underlay package discovery",
    );
    assert_stdout_equals(
        run_shell(
            &roc_overlay,
            &format!(
                "source /opt/ros/jazzy/setup.bash && source {}/install/setup.bash && source install/setup.bash && ros2 pkg prefix demo_cmake_pkg",
                roc_underlay.display()
            ),
        ),
        roc_underlay
            .join("install/demo_cmake_pkg")
            .to_string_lossy()
            .as_ref(),
        "roc overlay underlay package discovery",
    );
    assert_stdout_equals(
        run_shell(
            &colcon_overlay,
            &format!(
                "source /opt/ros/jazzy/setup.bash && source {}/install/setup.bash && source install/setup.bash && ros2 pkg prefix demo_python_pkg",
                colcon_underlay.display()
            ),
        ),
        colcon_overlay
            .join("install/demo_python_pkg")
            .to_string_lossy()
            .as_ref(),
        "colcon overlay overlay package discovery",
    );
    assert_stdout_equals(
        run_shell(
            &roc_overlay,
            &format!(
                "source /opt/ros/jazzy/setup.bash && source {}/install/setup.bash && source install/setup.bash && ros2 pkg prefix demo_python_pkg",
                roc_underlay.display()
            ),
        ),
        roc_overlay
            .join("install/demo_python_pkg")
            .to_string_lossy()
            .as_ref(),
        "roc overlay overlay package discovery",
    );
}

#[test]
#[ignore = "requires colcon and a sourced ROS 2 environment"]
fn validate_failed_build_resume_against_colcon() {
    if !command_exists("colcon") || !Path::new("/opt/ros/jazzy/setup.bash").exists() {
        return;
    }

    let temp = tempdir().unwrap();
    let colcon_ws = temp.path().join("colcon_ws");
    let roc_ws = temp.path().join("roc_ws");
    let fixture_root = fixture_workspace("dependency_chain");
    let valid_consumer_cmake = fixture_root.join("src/consumer_node_pkg/CMakeLists.txt");

    copy_workspace(&fixture_root, &colcon_ws);
    copy_workspace(&fixture_root, &roc_ws);

    let broken_cmake = "cmake_minimum_required(VERSION 3.8)\nproject(consumer_node_pkg)\nthis_is_not_valid_cmake()\n";
    fs::write(
        colcon_ws.join("src/consumer_node_pkg/CMakeLists.txt"),
        broken_cmake,
    )
    .unwrap();
    fs::write(roc_ws.join("src/consumer_node_pkg/CMakeLists.txt"), broken_cmake).unwrap();

    assert_failure(
        run_shell(
            &colcon_ws,
            "source /opt/ros/jazzy/setup.bash && colcon build --continue-on-error --base-paths src",
        ),
        "colcon failing build for dependency chain fixture",
    );
    assert_failure(
        run_shell(
            &roc_ws,
            &format!(
                "source /opt/ros/jazzy/setup.bash && {} work build --continue-on-error --base-paths src",
                env!("CARGO_BIN_EXE_roc")
            ),
        ),
        "roc failing build for dependency chain fixture",
    );

    let valid_cmake = fs::read_to_string(valid_consumer_cmake).unwrap();
    fs::write(
        colcon_ws.join("src/consumer_node_pkg/CMakeLists.txt"),
        &valid_cmake,
    )
    .unwrap();
    fs::write(roc_ws.join("src/consumer_node_pkg/CMakeLists.txt"), &valid_cmake).unwrap();

    assert_success(
        run_shell(
            &colcon_ws,
            "source /opt/ros/jazzy/setup.bash && colcon build --base-paths src --packages-select-build-failed",
        ),
        "colcon resume build for dependency chain fixture",
    );
    assert_success(
        run_shell(
            &roc_ws,
            &format!(
                "source /opt/ros/jazzy/setup.bash && {} work build --base-paths src --packages-select-build-failed",
                env!("CARGO_BIN_EXE_roc")
            ),
        ),
        "roc resume build for dependency chain fixture",
    );

    assert_stdout_equals(
        run_shell(
            &colcon_ws,
            "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix consumer_node_pkg",
        ),
        colcon_ws
            .join("install/consumer_node_pkg")
            .to_string_lossy()
            .as_ref(),
        "colcon resumed ros2 pkg prefix for dependency chain fixture",
    );
    assert_stdout_equals(
        run_shell(
            &roc_ws,
            "source /opt/ros/jazzy/setup.bash && source install/setup.bash && ros2 pkg prefix consumer_node_pkg",
        ),
        roc_ws
            .join("install/consumer_node_pkg")
            .to_string_lossy()
            .as_ref(),
        "roc resumed ros2 pkg prefix for dependency chain fixture",
    );
}
