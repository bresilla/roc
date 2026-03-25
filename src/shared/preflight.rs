use std::collections::HashMap;
use std::env;

fn path_contains_command(tool: &str, path_value: Option<&str>) -> bool {
    let Some(path_value) = path_value else {
        return false;
    };

    env::split_paths(path_value)
        .map(|dir| dir.join(tool))
        .any(|candidate| candidate.is_file())
}

fn has_ros_environment_vars(vars: &HashMap<String, String>) -> bool {
    [
        "AMENT_PREFIX_PATH",
        "COLCON_PREFIX_PATH",
        "CMAKE_PREFIX_PATH",
        "ROS_DISTRO",
    ]
    .iter()
    .any(|key| vars.get(*key).is_some_and(|value| !value.trim().is_empty()))
}

pub fn ensure_command_available(
    tool: &str,
    context: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if path_contains_command(tool, env::var("PATH").ok().as_deref()) {
        return Ok(());
    }

    Err(format!(
        "{context} requires `{tool}` on PATH. Install it or enter the environment that provides it."
    )
    .into())
}

pub fn ensure_ros_environment(context: &str) -> Result<(), Box<dyn std::error::Error>> {
    let vars = env::vars().collect::<HashMap<_, _>>();
    if has_ros_environment_vars(&vars) {
        return Ok(());
    }

    Err(format!(
        "{context} requires a sourced ROS environment. Expected one of AMENT_PREFIX_PATH, COLCON_PREFIX_PATH, CMAKE_PREFIX_PATH, or ROS_DISTRO to be set."
    )
    .into())
}

#[cfg(test)]
mod tests {
    use super::{has_ros_environment_vars, path_contains_command};
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn path_contains_command_detects_executable_names_on_path() {
        let temp = tempdir().unwrap();
        let tool_path = temp.path().join("ros2");
        fs::write(&tool_path, "#!/bin/sh\n").unwrap();

        assert!(path_contains_command(
            "ros2",
            Some(temp.path().to_string_lossy().as_ref())
        ));
        assert!(!path_contains_command(
            "ctest",
            Some(temp.path().to_string_lossy().as_ref())
        ));
    }

    #[test]
    fn has_ros_environment_vars_accepts_any_known_ros_marker() {
        let vars = HashMap::from([("ROS_DISTRO".to_string(), "jazzy".to_string())]);

        assert!(has_ros_environment_vars(&vars));
    }

    #[test]
    fn has_ros_environment_vars_rejects_empty_markers() {
        let vars = HashMap::from([
            ("ROS_DISTRO".to_string(), String::new()),
            ("AMENT_PREFIX_PATH".to_string(), "   ".to_string()),
        ]);

        assert!(!has_ros_environment_vars(&vars));
    }
}
