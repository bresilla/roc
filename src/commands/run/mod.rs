use crate::commands::cli::{required_string, run_async_command};
use crate::ui::blocks;
use crate::utils::{get_ros_workspace_paths, is_executable};
use clap::ArgMatches;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use walkdir::WalkDir;

async fn find_executable(
    package_name: &str,
    executable_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_paths = get_ros_workspace_paths();

    for workspace_path in workspace_paths {
        if workspace_path.exists() {
            // Look for executables in install spaces
            let install_path = workspace_path.join("install");
            if install_path.exists() {
                for entry in WalkDir::new(&install_path)
                    .follow_links(true)
                    .max_depth(4)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        let path = entry.path();

                        // Check if it's in a lib directory (where executables are typically stored)
                        if path.to_string_lossy().contains("/lib/") {
                            if let Some(file_name) = path.file_name() {
                                if file_name == executable_name {
                                    // Check if it's in the right package directory
                                    let path_str = path.to_string_lossy();
                                    if path_str.contains(&format!("/{}/", package_name))
                                        || path_str.contains(&format!("/install/{}/", package_name))
                                    {
                                        // Verify it's executable
                                        if is_executable(path) {
                                            return Ok(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Also look in devel spaces (for catkin workspaces)
            let devel_path = workspace_path.join("devel/lib");
            if devel_path.exists() {
                for entry in WalkDir::new(&devel_path)
                    .follow_links(true)
                    .max_depth(3)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if let Some(parent) = path.parent() {
                            if let Some(package_dir) = parent.file_name() {
                                if package_dir == package_name {
                                    if let Some(file_name) = path.file_name() {
                                        if file_name == executable_name && is_executable(path) {
                                            return Ok(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Look in system installation paths
            let system_lib_path = workspace_path.join("lib").join(package_name);
            if system_lib_path.exists() {
                let executable_path = system_lib_path.join(executable_name);
                if executable_path.exists() && is_executable(&executable_path) {
                    return Ok(executable_path);
                }
            }
        }
    }

    Err(format!(
        "Executable '{}' not found in package '{}'",
        executable_name, package_name
    )
    .into())
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let package_name = required_string(&matches, "package_name")?;
    let executable_name = required_string(&matches, "executable_name")?;
    let argv = matches
        .get_one::<String>("argv")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let prefix = matches.get_one::<String>("prefix").cloned();

    blocks::print_section("Run");
    blocks::print_field("Package", package_name);
    blocks::print_field("Executable", executable_name);
    if !argv.is_empty() {
        blocks::print_field("Arguments", &argv);
    }
    if let Some(prefix) = &prefix {
        blocks::print_field("Prefix", prefix);
    }

    // Find the actual executable file
    let executable_path = find_executable(package_name, executable_name).await?;
    blocks::print_field("Resolved Path", executable_path.display());
    println!();

    let argv_parts = argv
        .split_whitespace()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

    // Set up environment for ROS2
    let mut cmd = if let Some(prefix) = prefix {
        let prefix_parts: Vec<&str> = prefix.split_whitespace().collect();
        if prefix_parts.is_empty() {
            Command::new(&executable_path)
        } else {
            let mut prefixed_cmd = Command::new(prefix_parts[0]);
            for part in &prefix_parts[1..] {
                prefixed_cmd.arg(part);
            }
            prefixed_cmd.arg(&executable_path);
            prefixed_cmd
        }
    } else {
        Command::new(&executable_path)
    };

    // Add current environment, making sure ROS environment is preserved
    for (key, value) in env::vars() {
        cmd.env(key, value);
    }

    for arg in &argv_parts {
        cmd.arg(arg);
    }

    let command_preview = format!("{cmd:?}");
    blocks::print_field("Command", &command_preview);
    blocks::print_note("Child process stdio is attached to the terminal");

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let started_at = Instant::now();
    let status = cmd.status().await?;

    if !status.success() {
        return Err(format!("Executable failed with exit code: {:?}", status.code()).into());
    }

    println!();
    blocks::print_section("Run Summary");
    blocks::print_field("Package", package_name);
    blocks::print_field("Executable", executable_name);
    blocks::print_field(
        "Elapsed",
        format!("{:.2}s", started_at.elapsed().as_secs_f64()),
    );
    blocks::print_success("Process exited successfully");
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    run_async_command(run_command(matches));
}
