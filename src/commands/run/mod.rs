use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use std::env;
use std::os::unix::fs::PermissionsExt;
use walkdir::WalkDir;

async fn find_executable(package_name: &str, executable_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
                                    if path_str.contains(&format!("/{}/", package_name)) ||
                                       path_str.contains(&format!("/install/{}/", package_name)) {
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
    
    Err(format!("Executable '{}' not found in package '{}'", executable_name, package_name).into())
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

/// Check if a file is executable
fn is_executable(path: &std::path::Path) -> bool {
    if let Ok(metadata) = std::fs::metadata(path) {
        let permissions = metadata.permissions();
        permissions.mode() & 0o111 != 0
    } else {
        false
    }
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let package_name = matches.get_one::<String>("package_name").unwrap();
    let executable_name = matches.get_one::<String>("executable_name").unwrap();
    
    // Find the actual executable file
    let executable_path = find_executable(package_name, executable_name).await?;
    
    println!("Running: {}", executable_path.display());
    
    // Set up environment for ROS2
    let mut cmd = Command::new(&executable_path);
    
    // Add current environment, making sure ROS environment is preserved
    for (key, value) in env::vars() {
        cmd.env(key, value);
    }
    
    // Add executable arguments if provided
    if let Some(argv) = matches.get_one::<String>("argv") {
        // Parse and add arguments
        for arg in argv.split_whitespace() {
            cmd.arg(arg);
        }
    }
    
    // Apply prefix if provided
    if let Some(prefix) = matches.get_one::<String>("prefix") {
        // For prefix, we need to run the prefix command with our executable as an argument
        let prefix_parts: Vec<&str> = prefix.split_whitespace().collect();
        if !prefix_parts.is_empty() {
            let mut prefixed_cmd = Command::new(prefix_parts[0]);
            
            // Add prefix arguments
            for part in &prefix_parts[1..] {
                prefixed_cmd.arg(part);
            }
            
            // Add the executable and its arguments
            prefixed_cmd.arg(&executable_path);
            if let Some(argv) = matches.get_one::<String>("argv") {
                for arg in argv.split_whitespace() {
                    prefixed_cmd.arg(arg);
                }
            }
            
            cmd = prefixed_cmd;
        }
    }
    
    cmd.stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
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