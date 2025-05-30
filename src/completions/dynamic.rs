use clap::ArgMatches;
use std::{fs, env};
use std::path::PathBuf;
use walkdir::WalkDir;
use std::collections::HashSet;

/// Handle internal dynamic completion (_complete)
pub fn handle(matches: ArgMatches) {
    let command = matches.get_one::<String>("command").unwrap();
    let sub = matches.get_one::<String>("subcommand");
    let pos = matches.get_one::<String>("position").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);

    match (command.as_str(), sub.map(|s| s.as_str()).filter(|s| !s.is_empty()), pos) {
        ("launch", None, 1) => {
            for package in find_packages() {
                println!("{}", package);
            }
        }
        ("launch", None, 2) => {
            for launch_file in find_launch_files() {
                if let Some((_, file)) = launch_file.split_once(':') {
                    println!("{}", file);
                }
            }
        }
        ("run", None, 1) => {
            for package in find_packages() {
                println!("{}", package);
            }
        }
        ("run", None, 2) => {
            for executable in find_executables() {
                if let Some((_, name)) = executable.split_once(':') {
                    println!("{}", name);
                }
            }
        }
        _ => {
            // No completions for other positions
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
                    if let Some(ext) = path.extension() {
                        if (ext == "py" || ext == "launch" || ext == "xml") 
                            && (path.to_string_lossy().contains("/launch/")
                                || path.to_string_lossy().contains("launch.py")
                                || path.to_string_lossy().contains("launch.xml")) {
                            
                            // Extract package name from path like /opt/ros/humble/share/package_name/launch/file.launch.py
                            if let Some(share_idx) = path.to_string_lossy().find("/share/") {
                                let after_share = &path.to_string_lossy()[share_idx + 7..];
                                if let Some(next_slash) = after_share.find('/') {
                                    let package_name = &after_share[..next_slash];
                                    if let Some(stem) = path.file_stem() {
                                        launch_files.insert(format!("{}:{}", package_name, stem.to_string_lossy()));
                                    }
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
                    if let Some(ext) = path.extension() {
                        if (ext == "py" || ext == "launch" || ext == "xml")
                            && (path.to_string_lossy().contains("/launch/")
                                || path.to_string_lossy().contains("launch.py")
                                || path.to_string_lossy().contains("launch.xml")) {
                            
                            // Try to find package name by looking for package.xml
                            let mut current_path = path.parent();
                            while let Some(dir) = current_path {
                                if let Some(pkg_name) = find_package_name(&dir.to_path_buf()) {
                                    if let Some(stem) = path.file_stem() {
                                        launch_files.insert(format!("{}:{}", pkg_name, stem.to_string_lossy()));
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
    }
    
    launch_files.into_iter().collect()
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

/// Common ROS workspace paths
fn get_ros_workspace_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(distro) = env::var("ROS_DISTRO") {
        paths.push(PathBuf::from(format!("/opt/ros/{}", distro)));
    }
    if let Ok(dir) = env::current_dir() {
        paths.push(dir.clone());
        let mut p = dir.clone();
        for _ in 0..5 {
            if let Some(parent) = p.parent() {
                p = parent.to_path_buf();
                if p.join("src").exists() || p.join("install").exists() || p.join("devel").exists() {
                    paths.push(p.clone());
                }
            }
        }
    }
    if let Ok(prefix) = env::var("COLCON_PREFIX_PATH") {
        for part in prefix.split(':') {
            if let Some(parent) = PathBuf::from(part).parent() {
                paths.push(parent.to_path_buf());
            }
        }
    }
    if let Ok(prefix) = env::var("AMENT_PREFIX_PATH") {
        for part in prefix.split(':') {
            if let Some(parent) = PathBuf::from(part).parent() {
                paths.push(parent.to_path_buf());
            }
        }
    }
    paths.sort(); paths.dedup();
    paths
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

/// Check executable bit or extension
fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            return meta.permissions().mode() & 0o111 != 0;
        }
    }
    #[cfg(not(unix))] {
        if let Some(ext) = path.extension() {
            let e = ext.to_string_lossy().to_lowercase();
            return matches!(e.as_str(), "exe" | "bat" | "cmd");
        }
    }
    false
}
