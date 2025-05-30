use clap::ArgMatches;
use std::{fs, env};
use std::path::PathBuf;
use walkdir::WalkDir;
use std::collections::HashSet;
use crate::utils::{get_ros_workspace_paths, is_executable};

/// Handle internal dynamic completion (_complete)
pub fn handle(matches: ArgMatches) {
    let command = matches.get_one::<String>("command").unwrap();
    let sub = matches.get_one::<String>("subcommand");
    let subsub = matches.get_one::<String>("subsubcommand");
    let pos = matches.get_one::<String>("position").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    let current_args: Vec<String> = matches.get_many::<String>("current_args")
        .map(|values| values.cloned().collect())
        .unwrap_or_default();

    let sub_clean = sub.map(|s| s.as_str()).filter(|s| !s.is_empty());
    let subsub_clean = subsub.map(|s| s.as_str()).filter(|s| !s.is_empty());

    match (command.as_str(), sub_clean, subsub_clean, pos) {
        // roc launch completion
        ("launch", None, None, 1) => {
            for package in find_packages_with_launch_files() {
                println!("{}", package);
            }
        }
        ("launch", None, None, 2) => {
            let package_filter = current_args.get(0).map(|s| s.as_str());
            for launch_file in find_launch_files_for_package(package_filter) {
                if let Some((_, file)) = launch_file.split_once(':') {
                    println!("{}", file);
                }
            }
        }
        
        // roc run completion
        ("run", None, None, 1) => {
            for package in find_packages() {
                println!("{}", package);
            }
        }
        ("run", None, None, 2) => {
            let package_filter = current_args.get(0).map(|s| s.as_str());
            for executable in find_executables_for_package(package_filter) {
                if let Some((_, name)) = executable.split_once(':') {
                    println!("{}", name);
                }
            }
        }
        
        // roc topic completion
        ("topic", None, None, 1) => {
            // Complete topic subcommands
            let subcommands = ["echo", "info", "list", "pub", "bw", "delay", "find", "hz", "type"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("topic", Some("echo" | "info" | "bw" | "delay" | "hz" | "type"), None, 1) => {
            // Complete topic names
            for topic in find_topics() {
                println!("{}", topic);
            }
        }
        ("topic", Some("pub"), None, 1) => {
            // Complete topic names for publishing
            for topic in find_topics() {
                println!("{}", topic);
            }
        }
        ("topic", Some("pub"), None, 2) => {
            // Complete message types
            for msg_type in find_message_types() {
                println!("{}", msg_type);
            }
        }
        ("topic", Some("find"), None, 1) => {
            // Complete message types for find
            for msg_type in find_message_types() {
                println!("{}", msg_type);
            }
        }
        
        // roc service completion
        ("service", None, None, 1) => {
            let subcommands = ["call", "find", "list", "type"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("service", Some("call" | "type"), None, 1) => {
            for service in find_services() {
                println!("{}", service);
            }
        }
        ("service", Some("find"), None, 1) => {
            for service_type in find_service_types() {
                println!("{}", service_type);
            }
        }
        
        // roc param completion
        ("param", None, None, 1) => {
            let subcommands = ["get", "set", "list", "describe", "remove", "export", "import"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("param", Some("get" | "set" | "describe" | "remove"), None, 1) => {
            for param in find_parameters() {
                println!("{}", param);
            }
        }
        
        // roc node completion
        ("node", None, None, 1) => {
            let subcommands = ["list", "info"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("node", Some("info"), None, 1) => {
            for node in find_nodes() {
                println!("{}", node);
            }
        }
        
        // roc action completion
        ("action", None, None, 1) => {
            let subcommands = ["list", "info", "goal"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("action", Some("info" | "goal"), None, 1) => {
            for action in find_actions() {
                println!("{}", action);
            }
        }
        
        // roc interface completion
        ("interface", None, None, 1) => {
            let subcommands = ["list", "show", "package", "model"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("interface", Some("show"), None, 1) => {
            for interface in find_interfaces() {
                println!("{}", interface);
            }
        }
        ("interface", Some("package"), None, 1) => {
            for package in find_packages() {
                println!("{}", package);
            }
        }
        
        // roc bag completion
        ("bag", None, None, 1) => {
            let subcommands = ["record", "play", "info", "list"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("bag", Some("play" | "info"), None, 1) => {
            for bag in find_bag_files() {
                println!("{}", bag);
            }
        }
        
        // roc work completion
        ("work", None, None, 1) => {
            let subcommands = ["build", "create", "info", "list"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("work", Some("build"), None, 1) => {
            for package in find_packages() {
                println!("{}", package);
            }
        }
        
        // roc frame completion
        ("frame", None, None, 1) => {
            let subcommands = ["list", "echo", "info", "pub"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        ("frame", Some("echo" | "info"), None, 1) => {
            for frame in find_frames() {
                println!("{}", frame);
            }
        }
        
        // roc daemon completion
        ("daemon", None, None, 1) => {
            let subcommands = ["start", "stop", "status"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        
        // roc middleware completion
        ("middleware", None, None, 1) => {
            let subcommands = ["get", "set", "list"];
            for subcommand in subcommands {
                println!("{}", subcommand);
            }
        }
        
        _ => {
            // No completions for other positions/commands
        }
    }
}

/// Scan ROS workspaces for launch files
fn find_launch_files() -> Vec<String> {
    let mut launch_files = HashSet::new();
    
    // Look in ROS system installation first
    if let Ok(distro) = env::var("ROS_DISTRO") {
        let ros_path = PathBuf::from(format!("/opt/ros/{}/share", distro));
        if ros_path.exists() {
            for entry in WalkDir::new(&ros_path)
                .follow_links(true)
                .max_depth(3) // share/package/launch/file.launch.py
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    // Check if the file is in a launch directory
                    if path.to_string_lossy().contains("/launch/") {
                        // Extract package name from path like /opt/ros/humble/share/package_name/launch/file.launch.py
                        if let Some(share_idx) = path.to_string_lossy().find("/share/") {
                            let after_share = &path.to_string_lossy()[share_idx + 7..];
                            if let Some(next_slash) = after_share.find('/') {
                                let package_name = &after_share[..next_slash];
                                if let Some(file_name) = path.file_name() {
                                    launch_files.insert(format!("{}:{}", package_name, file_name.to_string_lossy()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Also look in workspace directories
    for workspace_path in get_ros_workspace_paths() {
        if workspace_path.exists() && !workspace_path.to_string_lossy().contains("/opt/ros/") {
            for entry in WalkDir::new(&workspace_path)
                .follow_links(true)
                .max_depth(6)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    // Check if the file is in a launch directory
                    if path.to_string_lossy().contains("/launch/") {
                        // Try to find package name by looking for package.xml
                        let mut current_path = path.parent();
                        while let Some(dir) = current_path {
                            if let Some(pkg_name) = find_package_name(&dir.to_path_buf()) {
                                if let Some(file_name) = path.file_name() {
                                    launch_files.insert(format!("{}:{}", pkg_name, file_name.to_string_lossy()));
                                }
                                break;
                            }
                            current_path = dir.parent();
                        }
                    }
                }
            }
        }
    }
    
    launch_files.into_iter().collect()
}

/// Get packages that have launch files
fn find_packages_with_launch_files() -> Vec<String> {
    let launch_files = find_launch_files();
    let mut packages = HashSet::new();
    
    for launch_file in launch_files {
        if let Some((package, _)) = launch_file.split_once(':') {
            packages.insert(package.to_string());
        }
    }
    
    packages.into_iter().collect()
}

/// Get launch files for a specific package (or all if no package specified)
fn find_launch_files_for_package(package_filter: Option<&str>) -> Vec<String> {
    let all_launch_files = find_launch_files();
    
    if let Some(package) = package_filter {
        all_launch_files.into_iter()
            .filter(|launch_file| {
                if let Some((pkg, _)) = launch_file.split_once(':') {
                    pkg == package
                } else {
                    false
                }
            })
            .collect()
    } else {
        all_launch_files
    }
}

/// Get executables for a specific package (or all if no package specified)
fn find_executables_for_package(package_filter: Option<&str>) -> Vec<String> {
    let all_executables = find_executables();
    
    if let Some(package) = package_filter {
        all_executables.into_iter()
            .filter(|executable| {
                if let Some((pkg, _)) = executable.split_once(':') {
                    pkg == package
                } else {
                    false
                }
            })
            .collect()
    } else {
        all_executables
    }
}

/// Scan ROS workspaces for executables
fn find_executables() -> Vec<String> {
    let mut executables = HashSet::new();
    
    // Look in ROS system installation first
    if let Ok(distro) = env::var("ROS_DISTRO") {
        // Check both lib and bin directories in ROS system installation
        let ros_lib_path = PathBuf::from(format!("/opt/ros/{}/lib", distro));
        let ros_bin_path = PathBuf::from(format!("/opt/ros/{}/bin", distro));
        
        // Scan lib directory for package-specific executables
        if ros_lib_path.exists() {
            for entry in WalkDir::new(&ros_lib_path)
                .follow_links(true)
                .max_depth(2) // lib/package_name/executable
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if is_executable(path) {
                        // Extract package name from path like /opt/ros/humble/lib/package_name/executable
                        if let Some(lib_idx) = path.to_string_lossy().find("/lib/") {
                            let after_lib = &path.to_string_lossy()[lib_idx + 5..];
                            if let Some(next_slash) = after_lib.find('/') {
                                let package_name = &after_lib[..next_slash];
                                if let Some(exec_name) = path.file_name() {
                                    executables.insert(format!("{}:{}", package_name, exec_name.to_string_lossy()));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Scan bin directory for general executables
        if ros_bin_path.exists() {
            for entry in WalkDir::new(&ros_bin_path)
                .follow_links(true)
                .max_depth(1) // bin/executable
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if is_executable(path) {
                        if let Some(exec_name) = path.file_name() {
                            // For bin executables, use "system" as package name
                            executables.insert(format!("system:{}", exec_name.to_string_lossy()));
                        }
                    }
                }
            }
        }
    }
    
    // Also look in workspace directories
    let workspace_paths = get_ros_workspace_paths();
    for workspace_path in workspace_paths {
        if workspace_path.exists() && !workspace_path.to_string_lossy().contains("/opt/ros/") {
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
                        if path.to_string_lossy().contains("/lib/") {
                            if let Some(parent) = path.parent() {
                                if let Some(pkg) = parent.file_name() {
                                    if is_executable(path) {
                                        executables.insert(format!("{}:{}", pkg.to_string_lossy(), path.file_name().unwrap().to_string_lossy()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
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
                            if let Some(pkg) = parent.file_name() {
                                if is_executable(path) {
                                    executables.insert(format!("{}:{}", pkg.to_string_lossy(), path.file_name().unwrap().to_string_lossy()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    executables.into_iter().collect()
}

/// Get available packages in the workspace
fn find_packages() -> Vec<String> {
    let mut packages = HashSet::new();
    
    // Look in ROS system installation first
    if let Ok(distro) = env::var("ROS_DISTRO") {
        let ros_share_path = PathBuf::from(format!("/opt/ros/{}/share", distro));
        if ros_share_path.exists() {
            for entry in WalkDir::new(&ros_share_path)
                .follow_links(true)
                .max_depth(2) // share/package_name/package.xml
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() && entry.file_name() == "package.xml" {
                    if let Some(parent) = entry.path().parent() {
                        if let Some(package_name) = parent.file_name() {
                            let pkg_name = package_name.to_string_lossy().to_string();
                            packages.insert(pkg_name);
                        }
                    }
                }
            }
        }
    }
    
    // Also look in workspace directories
    for workspace_path in get_ros_workspace_paths() {
        if workspace_path.exists() && !workspace_path.to_string_lossy().contains("/opt/ros/") {
            for entry in WalkDir::new(&workspace_path)
                .follow_links(true)
                .max_depth(6)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() && entry.file_name() == "package.xml" {
                    if let Some(parent) = entry.path().parent() {
                        if let Some(name) = find_package_name(&parent.to_path_buf()) {
                            packages.insert(name);
                        }
                    }
                }
            }
        }
    }
    
    packages.into_iter().collect()
}

/// Find ROS package name
fn find_package_name(dir: &PathBuf) -> Option<String> {
    let mut current = dir.clone();
    for _ in 0..10 {
        let xml = current.join("package.xml");
        if xml.exists() {
            if let Ok(content) = fs::read_to_string(&xml) {
                if let Some(start) = content.find("<name>") {
                    if let Some(end) = content[start + 6..].find("</name>") {
                        return Some(content[start + 6..start + 6 + end].trim().to_string());
                    }
                }
            }
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else { break; }
    }
    None
}

// Stub functions for ROS runtime completions
// These would typically query running ROS systems, but for now return empty lists

/// Find active ROS topics
fn find_topics() -> Vec<String> {
    // In a real implementation, this would run `ros2 topic list`
    vec![]
}

/// Find available message types
fn find_message_types() -> Vec<String> {
    // In a real implementation, this would run `ros2 interface list` and filter for msg types
    vec![]
}

/// Find active ROS services
fn find_services() -> Vec<String> {
    // In a real implementation, this would run `ros2 service list`
    vec![]
}

/// Find available service types
fn find_service_types() -> Vec<String> {
    // In a real implementation, this would run `ros2 interface list` and filter for srv types
    vec![]
}

/// Find ROS parameters
fn find_parameters() -> Vec<String> {
    // In a real implementation, this would run `ros2 param list`
    vec![]
}

/// Find active ROS nodes
fn find_nodes() -> Vec<String> {
    // In a real implementation, this would run `ros2 node list`
    vec![]
}

/// Find active ROS actions
fn find_actions() -> Vec<String> {
    // In a real implementation, this would run `ros2 action list`
    vec![]
}

/// Find available interfaces (msg/srv/action types)
fn find_interfaces() -> Vec<String> {
    // In a real implementation, this would run `ros2 interface list`
    vec![]
}

/// Find ROS bag files in current directory
fn find_bag_files() -> Vec<String> {
    // In a real implementation, this would scan for .db3 or bag files
    vec![]
}

/// Find TF frames
fn find_frames() -> Vec<String> {
    // In a real implementation, this would run `ros2 run tf2_tools view_frames.py` or similar
    vec![]
}
