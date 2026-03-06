use std::env;
use std::path::PathBuf;

/// Get common ROS workspace paths for finding packages, launch files, and executables
pub fn get_ros_workspace_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Add ROS system installation based on ROS_DISTRO environment variable
    if let Ok(distro) = env::var("ROS_DISTRO") {
        paths.push(PathBuf::from(format!("/opt/ros/{}", distro)));
    }

    // Add current working directory and parent directories (common workspace locations)
    if let Ok(current_dir) = env::current_dir() {
        paths.push(current_dir.clone());

        // Check parent directories for common workspace patterns
        let mut parent = current_dir.clone();
        for _ in 0..5 {
            // Search up to 5 levels up
            if let Some(p) = parent.parent() {
                parent = p.to_path_buf();

                // Look for workspace indicators
                if parent.join("src").exists()
                    || parent.join("install").exists()
                    || parent.join("devel").exists()
                {
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

/// Check if a file is executable (Unix-specific implementation)
#[cfg(unix)]
pub fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = std::fs::metadata(path) {
        let permissions = metadata.permissions();
        permissions.mode() & 0o111 != 0
    } else {
        false
    }
}

/// Check if a file is executable (non-Unix fallback)
#[cfg(not(unix))]
pub fn is_executable(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let e = ext.to_string_lossy().to_lowercase();
        matches!(e.as_str(), "exe" | "bat" | "cmd")
    } else {
        false
    }
}
