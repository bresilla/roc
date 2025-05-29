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

    match (command.as_str(), sub.map(|s| s.as_str()), pos) {
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
    let workspace_paths = get_ros_workspace_paths();

    for workspace_path in workspace_paths {
        if workspace_path.exists() {
            for entry in WalkDir::new(&workspace_path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "py" || ext == "launch" || ext == "xml" {
                            let path_str = path.to_string_lossy();
                            if path_str.contains("/launch/")
                                || path_str.contains("launch.py")
                                || path_str.contains("launch.xml")
                            {
                                if let Some(parent) = path.parent() {
                                    if let Some(pkg) = find_package_name(&parent.to_path_buf()) {
                                        if let Some(stem) = path.file_stem() {
                                            launch_files.insert(format!("{}:{}", pkg, stem.to_string_lossy()));
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
    launch_files.into_iter().collect()
}

/// Scan ROS workspaces for executables
fn find_executables() -> Vec<String> {
    let mut executables = HashSet::new();
    let workspace_paths = get_ros_workspace_paths();

    for workspace_path in workspace_paths {
        if workspace_path.exists() {
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
    for workspace_path in get_ros_workspace_paths() {
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
