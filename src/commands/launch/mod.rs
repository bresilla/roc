use crate::utils::get_ros_workspace_paths;
use clap::ArgMatches;
use std::path::PathBuf;
use std::process::Stdio;
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
    let package_name = matches.get_one::<String>("package_name").unwrap();
    let launch_file_name = matches.get_one::<String>("launch_file_name").unwrap();

    // Find the actual launch file
    let launch_file_path = find_launch_file(package_name, launch_file_name).await?;

    println!("Launching: {}", launch_file_path.display());

    // All launch files should be executed through ros2 launch command
    let mut cmd = Command::new("ros2");
    cmd.args(&["launch", package_name, launch_file_name]);

    // Add launch arguments if provided
    if let Some(launch_arguments) = matches.get_one::<String>("launch_arguments") {
        // Parse and add arguments
        for arg in launch_arguments.split_whitespace() {
            cmd.arg(arg);
        }
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

    // Set up stdio to inherit from parent (better for interactive launch files)
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let status = cmd.status().await?;

    if !status.success() {
        return Err(format!("Launch failed with exit code: {:?}", status.code()).into());
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches));
}
