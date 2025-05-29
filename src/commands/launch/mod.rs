use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use std::env;
use walkdir::WalkDir;

async fn find_launch_file(package_name: &str, launch_file_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
                            if let Some(file_stem) = path.file_stem() {
                                if file_stem == launch_file_name || 
                                   (launch_file_name.ends_with(".py") && path.file_name() == Some(std::ffi::OsStr::new(launch_file_name))) ||
                                   (launch_file_name.ends_with(".xml") && path.file_name() == Some(std::ffi::OsStr::new(launch_file_name))) ||
                                   (launch_file_name.ends_with(".launch") && path.file_name() == Some(std::ffi::OsStr::new(launch_file_name))) {
                                    
                                    // Check if it's in the right package
                                    let path_str = path.to_string_lossy();
                                    if path_str.contains(&format!("/{}/", package_name)) ||
                                       path_str.contains(&format!("/{}/launch/", package_name)) ||
                                       path_str.contains(&format!("/{}/share/{}/", package_name, package_name)) {
                                        
                                        // Verify it's actually a launch file
                                        if let Some(extension) = path.extension() {
                                            if extension == "py" || extension == "launch" || extension == "xml" {
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
    }
    
    Err(format!("Launch file '{}' not found in package '{}'", launch_file_name, package_name).into())
}

fn get_ros_workspace_paths() -> Vec<PathBuf> {
    let mut paths = vec![];
    
    // Add ROS system installation based on ROS_DISTRO environment variable
    if let Ok(ros_distro) = env::var("ROS_DISTRO") {
        paths.push(PathBuf::from(format!("/opt/ros/{}", ros_distro)));
    }
    
    // Add current working directory and parent directories (common workspace locations)
    if let Ok(current_dir) = env::current_dir() {
        paths.push(current_dir.clone());
        
        // Check parent directories for common workspace patterns
        let mut parent = current_dir.clone();
        for _ in 0..5 { // Search up to 5 levels up
            if let Some(p) = parent.parent() {
                parent = p.to_path_buf();
                
                // Look for workspace indicators
                if parent.join("src").exists() || 
                   parent.join("install").exists() ||
                   parent.join("devel").exists() {
                    paths.push(parent.clone());
                }
            } else {
                break;
            }
        }
    }
    
    // Add paths from ROS environment variables
    if let Ok(colcon_prefix_path) = env::var("COLCON_PREFIX_PATH") {
        for path in colcon_prefix_path.split(':') {
            let path_buf = PathBuf::from(path);
            if let Some(parent) = path_buf.parent() {
                paths.push(parent.to_path_buf());
            }
        }
    }
    
    if let Ok(ament_prefix_path) = env::var("AMENT_PREFIX_PATH") {
        for path in ament_prefix_path.split(':') {
            let path_buf = PathBuf::from(path);
            if let Some(parent) = path_buf.parent() {
                paths.push(parent.to_path_buf());
            }
        }
    }
    
    // Remove duplicates and return
    paths.sort();
    paths.dedup();
    paths
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let package_name = matches.get_one::<String>("package_name").unwrap();
    let launch_file_name = matches.get_one::<String>("launch_file_name").unwrap();
    
    // Find the actual launch file
    let launch_file_path = find_launch_file(package_name, launch_file_name).await?;
    
    println!("Launching: {}", launch_file_path.display());
    
    // Determine how to execute the launch file based on its extension
    let mut cmd = if let Some(extension) = launch_file_path.extension() {
        match extension.to_str() {
            Some("py") => {
                let mut command = Command::new("python3");
                command.arg(&launch_file_path);
                command
            },
            Some("xml") | Some("launch") => {
                // For XML launch files, we still need to use ros2 launch
                let mut command = Command::new("ros2");
                command.args(&["launch", &launch_file_path.to_string_lossy()]);
                command
            },
            _ => {
                return Err(format!("Unsupported launch file type: {}", launch_file_path.display()).into());
            }
        }
    } else {
        return Err(format!("Cannot determine launch file type: {}", launch_file_path.display()).into());
    };
    
    // Add launch arguments if provided
    if let Some(launch_arguments) = matches.get_one::<String>("launch_arguments") {
        // Parse and add arguments
        for arg in launch_arguments.split_whitespace() {
            cmd.arg(arg);
        }
    }
    
    // Set up stdio
    cmd.stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    // Apply launch options for ros2 launch
    if launch_file_path.extension().map(|e| e == "xml" || e == "launch").unwrap_or(false) {
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
    }
    
    let mut child = cmd.spawn()?;
    
    let stdout = child.stdout.take().unwrap();
    let mut reader = tokio::io::BufReader::new(stdout);

    let mut buffer = [0u8; 1024];
    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        let output = String::from_utf8_lossy(&buffer[0..n]);
        print!("{}", output);
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches));
}