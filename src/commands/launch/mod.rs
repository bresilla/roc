use crate::commands::cli::{required_string, run_async_command};
use crate::ui::blocks;
use crate::utils::get_ros_workspace_paths;
use clap::ArgMatches;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use walkdir::WalkDir;

async fn find_launch_file(
    package_name: &str,
    launch_file_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_paths = get_ros_workspace_paths();

    for workspace_path in workspace_paths {
        if workspace_path.exists() {
            // Look for launch files in various ROS workspace locations
            let search_paths = vec![
                workspace_path.join("src"),
                workspace_path.join("install"),
                workspace_path.join("share"),
            ];

            for search_path in search_paths {
                if search_path.exists() {
                    for entry in WalkDir::new(&search_path)
                        .follow_links(true)
                        .max_depth(6)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            let path = entry.path();

                            // Check if it's a launch file with the right name
                            if let Some(file_name) = path.file_name() {
                                let file_name_str = file_name.to_string_lossy();

                                // Match by exact filename or by stem (for files without extension)
                                let matches_name = file_name_str == launch_file_name
                                    || path.file_stem().map(|s| s.to_string_lossy())
                                        == Some(launch_file_name.into());

                                if matches_name {
                                    // Check if it's in the right package
                                    let path_str = path.to_string_lossy();
                                    if path_str.contains(&format!("/{}/", package_name))
                                        || path_str.contains(&format!("/{}/launch/", package_name))
                                        || path_str.contains(&format!(
                                            "/{}/share/{}/",
                                            package_name, package_name
                                        ))
                                    {
                                        // It's a launch file if it's in a launch directory
                                        if path_str.contains("/launch/") {
                                            return Ok(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err(format!(
        "Launch file '{}' not found in package '{}'",
        launch_file_name, package_name
    )
    .into())
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let package_name = required_string(&matches, "package_name")?;
    let launch_file_name = required_string(&matches, "launch_file_name")?;
    let launch_arguments = matches
        .get_one::<String>("launch_arguments")
        .map(|value| value.to_string())
        .unwrap_or_default();

    blocks::print_section("Launch");
    blocks::print_field("Package", package_name);
    blocks::print_field("Launch File", launch_file_name);
    if !launch_arguments.is_empty() {
        blocks::print_field("Arguments", &launch_arguments);
    }
    if matches.get_flag("noninteractive") {
        blocks::print_field("Noninteractive", "yes");
    }
    if matches.get_flag("debug") {
        blocks::print_field("Debug", "enabled");
    }
    if matches.get_flag("print") {
        blocks::print_field("Print Only", "yes");
    }
    if matches.get_flag("show_args") {
        blocks::print_field("Show Args", "yes");
    }
    if matches.get_flag("show_all") {
        blocks::print_field("Show All Output", "yes");
    }
    if let Some(prefix) = matches.get_one::<String>("launch_prefix") {
        blocks::print_field("Launch Prefix", prefix);
    }
    if let Some(filter) = matches.get_one::<String>("launch_prefix_filter") {
        blocks::print_field("Prefix Filter", filter);
    }

    // Find the actual launch file
    let launch_file_path = find_launch_file(package_name, launch_file_name).await?;
    blocks::print_field("Resolved Path", launch_file_path.display());

    // All launch files should be executed through ros2 launch command
    let mut cmd = Command::new("ros2");
    cmd.args(&["launch", package_name, launch_file_name]);

    // Add launch arguments if provided
    for arg in launch_arguments.split_whitespace() {
        cmd.arg(arg);
    }

    // Apply launch options
    if matches.get_flag("noninteractive") {
        cmd.arg("--noninteractive");
    }

    if matches.get_flag("debug") {
        cmd.arg("--debug");
    }

    if matches.get_flag("print") {
        cmd.arg("--print");
    }

    if matches.get_flag("show_args") {
        cmd.arg("--show-args");
    }

    if matches.get_flag("show_all") {
        cmd.arg("--show-all-subprocesses-output");
    }

    if let Some(launch_prefix) = matches.get_one::<String>("launch_prefix") {
        cmd.args(&["--launch-prefix", launch_prefix]);
    }

    if let Some(launch_prefix_filter) = matches.get_one::<String>("launch_prefix_filter") {
        cmd.args(&["--launch-prefix-filter", launch_prefix_filter]);
    }

    println!();
    blocks::print_field("Command", format!("{cmd:?}"));
    blocks::print_note("Child process stdio is attached to the terminal");

    // Set up stdio to inherit from parent (better for interactive launch files)
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let started_at = Instant::now();
    let status = cmd.status().await?;

    if !status.success() {
        return Err(format!("Launch failed with exit code: {:?}", status.code()).into());
    }

    println!();
    blocks::print_section("Launch Summary");
    blocks::print_field("Package", package_name);
    blocks::print_field("Launch File", launch_file_name);
    blocks::print_field(
        "Elapsed",
        format!("{:.2}s", started_at.elapsed().as_secs_f64()),
    );
    blocks::print_success("Launch command exited successfully");
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    run_async_command(run_command(matches));
}
