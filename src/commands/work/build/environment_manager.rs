use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Manages environment variables and setup scripts for ROS2 workspace builds
pub struct EnvironmentManager {
    /// Current environment variables
    env_vars: HashMap<String, String>,
    /// Install prefix directory
    install_prefix: PathBuf,
    /// Whether we're using isolated installs
    isolated: bool,
}

impl EnvironmentManager {
    pub fn new(install_prefix: PathBuf, isolated: bool) -> Self {
        let mut env_vars = HashMap::new();

        // Initialize with current environment
        for (key, value) in env::vars() {
            env_vars.insert(key, value);
        }

        Self {
            env_vars,
            install_prefix,
            isolated,
        }
    }

    /// Update environment variables for a package build
    pub fn setup_package_environment(
        &mut self,
        package_name: &str,
        _package_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let install_dir = if self.isolated {
            self.install_prefix.join(package_name)
        } else {
            self.install_prefix.clone()
        };

        // Update CMAKE_PREFIX_PATH
        self.update_path_env("CMAKE_PREFIX_PATH", &install_dir);

        // Update AMENT_PREFIX_PATH
        self.update_path_env("AMENT_PREFIX_PATH", &install_dir);

        // Update PATH to include bin directories
        let bin_dir = install_dir.join("bin");
        if bin_dir.exists() {
            self.update_path_env("PATH", &bin_dir);
        }

        // Update library paths
        #[cfg(target_os = "linux")]
        {
            let lib_dir = install_dir.join("lib");
            if lib_dir.exists() {
                self.update_path_env("LD_LIBRARY_PATH", &lib_dir);
            }
        }

        #[cfg(target_os = "macos")]
        {
            let lib_dir = install_dir.join("lib");
            if lib_dir.exists() {
                self.update_path_env("DYLD_LIBRARY_PATH", &lib_dir);
            }
        }

        // Update Python path
        let python_lib_dirs = [
            install_dir
                .join("lib")
                .join("python3")
                .join("site-packages"),
            install_dir
                .join("local")
                .join("lib")
                .join("python3")
                .join("site-packages"),
        ];

        for python_dir in &python_lib_dirs {
            if python_dir.exists() {
                self.update_path_env("PYTHONPATH", python_dir);
            }
        }

        Ok(())
    }

    /// Update a PATH-like environment variable by prepending a new path
    fn update_path_env(&mut self, var_name: &str, new_path: &Path) {
        let separator = if cfg!(windows) { ";" } else { ":" };
        let new_path_str = new_path.to_string_lossy();

        if let Some(current) = self.env_vars.get(var_name) {
            // Check if path is already in the variable
            let paths: Vec<&str> = current.split(separator).collect();
            if !paths.contains(&new_path_str.as_ref()) {
                let updated = format!("{}{}{}", new_path_str, separator, current);
                self.env_vars.insert(var_name.to_string(), updated);
            }
        } else {
            self.env_vars
                .insert(var_name.to_string(), new_path_str.to_string());
        }
    }

    /// Check if an environment variable is relevant for ROS workspaces
    fn is_ros_relevant_env_var(key: &str) -> bool {
        match key {
            // Core ROS2 environment variables
            "CMAKE_PREFIX_PATH" | "AMENT_PREFIX_PATH" | "COLCON_PREFIX_PATH" => true,

            // System library paths
            "PATH" | "LD_LIBRARY_PATH" | "DYLD_LIBRARY_PATH" => true,

            // Python paths
            "PYTHONPATH" => true,

            // ROS-specific variables
            key if key.starts_with("ROS_") => true,
            key if key.starts_with("AMENT_") => true,
            key if key.starts_with("COLCON_") => true,
            key if key.starts_with("RCUTILS_") => true,
            key if key.starts_with("RMW_") => true,
            key if key.starts_with("FASTRTPS_") => true,
            key if key.starts_with("CYCLONE_") => true,

            // Build-related variables
            "PKG_CONFIG_PATH" | "CMAKE_MODULE_PATH" => true,

            // Workspace sourcing marker
            "ROC_WORKSPACE_SOURCED" => true,

            // Ignore system and private variables
            _ => false,
        }
    }

    /// Get current environment variables as a HashMap
    pub fn get_env_vars(&self) -> &HashMap<String, String> {
        &self.env_vars
    }

    /// Generate setup script for the workspace
    pub fn generate_setup_script(
        &self,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let script_content = if cfg!(windows) {
            self.generate_batch_script()
        } else {
            self.generate_bash_script()
        };

        let mut file = fs::File::create(output_path)?;
        file.write_all(script_content.as_bytes())?;

        // Make script executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(output_path, perms)?;
        }

        Ok(())
    }

    /// Generate bash setup script content
    fn generate_bash_script(&self) -> String {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# ROS2 workspace setup script generated by roc\n");
        script.push_str("# Source this file to setup your environment\n\n");

        // Function to safely update PATH-like variables
        script.push_str(
            r#"_roc_prepend_path() {
    local var_name="$1"
    local new_path="$2"
    
    if [ -z "${!var_name}" ]; then
        export "$var_name"="$new_path"
    else
        # Check if path is already present
        if [[ ":${!var_name}:" != *":$new_path:"* ]]; then
            export "$var_name"="$new_path:${!var_name}"
        fi
    fi
}

"#,
        );

        // Add environment variable exports with ROS-specific filtering
        for (key, value) in &self.env_vars {
            // Only export ROS-related and essential environment variables
            if Self::is_ros_relevant_env_var(key) {
                script.push_str(&format!("export {}=\"{}\"\n", key, value));
            }
        }

        script.push_str("\n# Mark workspace as sourced\n");
        script.push_str("export ROC_WORKSPACE_SOURCED=1\n");

        script
    }

    /// Generate Windows batch script content
    fn generate_batch_script(&self) -> String {
        let mut script = String::new();
        script.push_str("@echo off\n");
        script.push_str("REM ROS2 workspace setup script generated by roc\n");
        script.push_str("REM Call this file to setup your environment\n\n");

        // Add environment variable sets with ROS-specific filtering
        for (key, value) in &self.env_vars {
            // Only export ROS-related and essential environment variables
            if Self::is_ros_relevant_env_var(key) {
                script.push_str(&format!("set \"{}={}\"\n", key, value));
            }
        }

        script.push_str("\nREM Mark workspace as sourced\n");
        script.push_str("set \"ROC_WORKSPACE_SOURCED=1\"\n");

        script
    }

    /// Generate a local setup script for a specific package
    #[allow(dead_code)]
    pub fn generate_package_setup(
        &self,
        package_name: &str,
        package_install_dir: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let script_content = if cfg!(windows) {
            format!(
                "@echo off\n\
                 REM Setup script for package {}\n\
                 set \"CMAKE_PREFIX_PATH={};%CMAKE_PREFIX_PATH%\"\n\
                 set \"AMENT_PREFIX_PATH={};%AMENT_PREFIX_PATH%\"\n\
                 if exist \"{}\\bin\" set \"PATH={}\\bin;%PATH%\"\n\
                 if exist \"{}\\lib\" set \"PATH={}\\lib;%PATH%\"\n",
                package_name,
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display()
            )
        } else {
            format!(
                "#!/bin/bash\n\
                 # Setup script for package {}\n\
                 export CMAKE_PREFIX_PATH=\"{}:$CMAKE_PREFIX_PATH\"\n\
                 export AMENT_PREFIX_PATH=\"{}:$AMENT_PREFIX_PATH\"\n\
                 [ -d \"{}/bin\" ] && export PATH=\"{}/bin:$PATH\"\n\
                 [ -d \"{}/lib\" ] && export LD_LIBRARY_PATH=\"{}/lib:$LD_LIBRARY_PATH\"\n",
                package_name,
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display(),
                package_install_dir.display()
            )
        };

        Ok(script_content)
    }

    /// Reset environment to clean state
    #[allow(dead_code)]
    pub fn reset_environment(&mut self) {
        self.env_vars.clear();
        for (key, value) in env::vars() {
            self.env_vars.insert(key, value);
        }
    }

    /// Set a specific environment variable
    #[allow(dead_code)]
    pub fn set_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Get the value of an environment variable
    #[allow(dead_code)]
    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_environment_manager_creation() {
        let install_prefix = PathBuf::from("/tmp/install");
        let env_mgr = EnvironmentManager::new(install_prefix, true);
        assert!(!env_mgr.get_env_vars().is_empty());
    }

    #[test]
    fn test_path_update() {
        let install_prefix = PathBuf::from("/tmp/install");
        let mut env_mgr = EnvironmentManager::new(install_prefix, true);

        let test_path = PathBuf::from("/test/path");
        env_mgr.update_path_env("TEST_PATH", &test_path);

        assert_eq!(
            env_mgr.get_env_var("TEST_PATH"),
            Some(&"/test/path".to_string())
        );
    }
}
