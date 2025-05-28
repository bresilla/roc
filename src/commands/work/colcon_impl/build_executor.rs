use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use std::time::Instant;

use crate::commands::work::colcon_impl::{PackageMeta, BuildType, BuildConfig};
use super::environment_manager::EnvironmentManager;

pub struct BuildExecutor<'a> {
    config: &'a BuildConfig,
    install_paths: HashMap<String, PathBuf>,
    env_manager: EnvironmentManager,
}

impl<'a> BuildExecutor<'a> {
    pub fn new(config: &'a BuildConfig) -> Self {
        let env_manager = EnvironmentManager::new(
            config.install_base.clone(),
            config.isolated
        );
        
        Self {
            config,
            install_paths: HashMap::new(),
            env_manager,
        }
    }
    
    pub fn build_all(
        &mut self,
        packages: &[PackageMeta],
        build_order: &[usize],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create necessary directories
        self.create_workspace_directories()?;
        
        let mut successful_builds = 0;
        let mut failed_builds = 0;
        
        for &pkg_idx in build_order {
            let package = &packages[pkg_idx];
            
            println!("Starting >>> {} ({:?})", package.name, package.build_type);
            let start_time = Instant::now();
            
            // Update environment for this package's dependencies
            self.update_environment_for_package(package)?;
            
            match self.build_package(package) {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    println!("Finished <<< {} [{:.2}s]", package.name, duration.as_secs_f64());
                    
                    // Record install path for environment setup
                    let install_path = if self.config.merge_install {
                        self.config.workspace_root.join("install")
                    } else {
                        self.config.workspace_root.join("install").join(&package.name)
                    };
                    self.install_paths.insert(package.name.clone(), install_path.clone());
                    
                    // Update environment for subsequent packages
                    self.add_package_to_environment(&package.name, &install_path)?;
                    
                    successful_builds += 1;
                }
                Err(e) => {
                    eprintln!("Failed <<< {} - {}", package.name, e);
                    failed_builds += 1;
                    
                    if !self.config.continue_on_error {
                        return Err(format!("Build failed for package {}: {}", package.name, e).into());
                    }
                }
            }
        }
        
        println!("\nBuild Summary:");
        println!("  {} packages succeeded", successful_builds);
        if failed_builds > 0 {
            println!("  {} packages failed", failed_builds);
        }
        
        // Generate environment setup scripts
        self.generate_setup_scripts(packages)?;
        
        Ok(())
    }
    
    fn create_workspace_directories(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.workspace_root.join("build");
        let install_dir = self.config.workspace_root.join("install");
        let log_dir = self.config.workspace_root.join("log");
        
        fs::create_dir_all(&build_dir)?;
        fs::create_dir_all(&install_dir)?;
        fs::create_dir_all(&log_dir)?;
        
        Ok(())
    }
    
    fn update_environment_for_package(&mut self, package: &PackageMeta) -> Result<(), Box<dyn std::error::Error>> {
        // Update environment for this package
        self.env_manager.setup_package_environment(&package.name, &package.path)?;
        
        // Add dependencies' install paths to environment
        let dep_paths: Vec<(String, PathBuf)> = package.build_deps.iter()
            .filter_map(|dep_name| {
                self.install_paths.get(dep_name).map(|path| (dep_name.clone(), path.clone()))
            })
            .collect();
            
        for (dep_name, dep_path) in dep_paths {
            self.add_package_to_environment(&dep_name, &dep_path)?;
        }
        
        Ok(())
    }
    
    fn add_package_to_environment(&mut self, package_name: &str, install_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.env_manager.setup_package_environment(package_name, install_path)?;
        Ok(())
    }
    
    fn build_package(&self, package: &PackageMeta) -> Result<(), Box<dyn std::error::Error>> {
        match package.build_type {
            BuildType::AmentCmake | BuildType::Cmake => self.build_cmake_package(package),
            BuildType::AmentPython => self.build_python_package(package),
            BuildType::Other(ref build_type) => {
                Err(format!("Unsupported build type: {}", build_type).into())
            }
        }
    }
    
    fn build_cmake_package(&self, package: &PackageMeta) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.workspace_root.join("build").join(&package.name);
        let install_prefix = if self.config.merge_install {
            self.config.workspace_root.join("install")
        } else {
            self.config.workspace_root.join("install").join(&package.name)
        };
        
        fs::create_dir_all(&build_dir)?;
        
        // Configure
        let mut configure_cmd = Command::new("cmake");
        configure_cmd
            .arg("-S").arg(&package.path)
            .arg("-B").arg(&build_dir)
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_prefix.display()));
        
        if self.config.symlink_install {
            configure_cmd.arg("-DCMAKE_INSTALL_MODE=ABS_SYMLINK_FILES");
        }
        
        // Add user-provided cmake args
        configure_cmd.args(&self.config.cmake_args);
        
        // Set environment
        configure_cmd.envs(self.env_manager.get_env_vars());
        
        println!("  Configuring with CMake...");
        let configure_output = configure_cmd.output()?;
        if !configure_output.status.success() {
            println!("  ❌ CMake configure failed");
            println!("  stdout: {}", String::from_utf8_lossy(&configure_output.stdout));
            println!("  stderr: {}", String::from_utf8_lossy(&configure_output.stderr));
            return Err(format!(
                "CMake configure failed:\n{}",
                String::from_utf8_lossy(&configure_output.stderr)
            ).into());
        }
        println!("  ✅ CMake configure succeeded");
        
        // Build and install
        let mut build_cmd = Command::new("cmake");
        build_cmd
            .arg("--build").arg(&build_dir)
            .arg("--target");
        
        if let Some(ref target) = self.config.cmake_target {
            build_cmd.arg(target);
        } else {
            build_cmd.arg("install");
        }
        
        build_cmd
            .arg("--")
            .arg(format!("-j{}", self.config.parallel_workers));
        
        build_cmd.envs(self.env_manager.get_env_vars());
        
        println!("  Building and installing...");
        let build_output = build_cmd.output()?;
        if !build_output.status.success() {
            println!("  ❌ CMake build failed");
            println!("  stdout: {}", String::from_utf8_lossy(&build_output.stdout));
            println!("  stderr: {}", String::from_utf8_lossy(&build_output.stderr));
            return Err(format!(
                "CMake build failed:\n{}",
                String::from_utf8_lossy(&build_output.stderr)
            ).into());
        }
        println!("  ✅ Build and install succeeded");
        
        Ok(())
    }
    
    fn build_python_package(&self, package: &PackageMeta) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.workspace_root.join("build").join(&package.name);
        let install_prefix = if self.config.merge_install {
            self.config.workspace_root.join("install")
        } else {
            self.config.workspace_root.join("install").join(&package.name)
        };
        
        fs::create_dir_all(&build_dir)?;
        
        // Build
        let build_output = Command::new("python3")
            .arg("setup.py")
            .arg("build")
            .arg("--build-base").arg(&build_dir)
            .current_dir(&package.path)
            .envs(self.env_manager.get_env_vars())
            .output()?;
            
        if !build_output.status.success() {
            return Err(format!(
                "Python build failed:\n{}",
                String::from_utf8_lossy(&build_output.stderr)
            ).into());
        }
        
        // Install
        let install_output = Command::new("python3")
            .arg("setup.py")
            .arg("install")
            .arg("--prefix").arg("")
            .arg("--root").arg(&install_prefix)
            .current_dir(&package.path)
            .envs(self.env_manager.get_env_vars())
            .output()?;
            
        if !install_output.status.success() {
            return Err(format!(
                "Python install failed:\n{}",
                String::from_utf8_lossy(&install_output.stderr)
            ).into());
        }
        
        Ok(())
    }
    
    fn generate_setup_scripts(&self, packages: &[PackageMeta]) -> Result<(), Box<dyn std::error::Error>> {
        let install_dir = self.config.workspace_root.join("install");
        
        if self.config.merge_install {
            self.generate_merged_setup_scripts(&install_dir)?;
        } else {
            self.generate_isolated_setup_scripts(&install_dir, packages)?;
        }
        
        Ok(())
    }
    
    fn generate_merged_setup_scripts(&self, install_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let setup_bash = install_dir.join("setup.bash");
        let setup_content = format!(r#"#!/bin/bash
# Generated by roc workspace build tool

# Source any parent workspaces
if [ -n "$COLCON_CURRENT_PREFIX" ]; then
    _colcon_current_prefix="$COLCON_CURRENT_PREFIX"
fi
export COLCON_CURRENT_PREFIX="{}"

# Add this workspace to environment
export CMAKE_PREFIX_PATH="$COLCON_CURRENT_PREFIX:${{CMAKE_PREFIX_PATH}}"
export AMENT_PREFIX_PATH="$COLCON_CURRENT_PREFIX:${{AMENT_PREFIX_PATH}}"

if [ -d "$COLCON_CURRENT_PREFIX/bin" ]; then
    export PATH="$COLCON_CURRENT_PREFIX/bin:${{PATH}}"
fi

if [ -d "$COLCON_CURRENT_PREFIX/lib" ]; then
    export LD_LIBRARY_PATH="$COLCON_CURRENT_PREFIX/lib:${{LD_LIBRARY_PATH}}"
fi

if [ -d "$COLCON_CURRENT_PREFIX/lib/python3.10/site-packages" ]; then
    export PYTHONPATH="$COLCON_CURRENT_PREFIX/lib/python3.10/site-packages:${{PYTHONPATH}}"
fi

# Restore previous prefix
if [ -n "$_colcon_current_prefix" ]; then
    export COLCON_CURRENT_PREFIX="$_colcon_current_prefix"
    unset _colcon_current_prefix
else
    unset COLCON_CURRENT_PREFIX
fi
"#, install_dir.display());
        
        fs::write(&setup_bash, setup_content)?;
        
        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&setup_bash)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&setup_bash, perms)?;
        }
        
        Ok(())
    }
    
    fn generate_isolated_setup_scripts(&self, install_dir: &Path, packages: &[PackageMeta]) -> Result<(), Box<dyn std::error::Error>> {
        // Generate individual package setup scripts
        for package in packages {
            if let Some(pkg_install_path) = self.install_paths.get(&package.name) {
                let package_dir = pkg_install_path.join("share").join(&package.name);
                fs::create_dir_all(&package_dir)?;
                
                let package_setup = package_dir.join("package.bash");
                let package_setup_content = format!(r#"#!/bin/bash
# Generated setup script for package {}

export CMAKE_PREFIX_PATH="{}:${{CMAKE_PREFIX_PATH}}"
export AMENT_PREFIX_PATH="{}:${{AMENT_PREFIX_PATH}}"

if [ -d "{}/bin" ]; then
    export PATH="{}/bin:${{PATH}}"
fi

if [ -d "{}/lib" ]; then
    export LD_LIBRARY_PATH="{}/lib:${{LD_LIBRARY_PATH}}"
fi
"#, 
                    package.name,
                    pkg_install_path.display(),
                    pkg_install_path.display(),
                    pkg_install_path.display(),
                    pkg_install_path.display(),
                    pkg_install_path.display(),
                    pkg_install_path.display()
                );
                
                fs::write(&package_setup, package_setup_content)?;
            }
        }
        
        // Generate workspace setup script
        let setup_bash = install_dir.join("setup.bash");
        let mut setup_content = String::from(r#"#!/bin/bash
# Generated by roc workspace build tool

if [ -n "$COLCON_CURRENT_PREFIX" ]; then
    _colcon_current_prefix="$COLCON_CURRENT_PREFIX"
fi
export COLCON_CURRENT_PREFIX="{}"

"#);
        
        // Source each package in dependency order
        for package in packages {
            if self.install_paths.contains_key(&package.name) {
                setup_content.push_str(&format!(
                    r#"if [ -f "$COLCON_CURRENT_PREFIX/{}/share/{}/package.bash" ]; then
    source "$COLCON_CURRENT_PREFIX/{}/share/{}/package.bash"
fi
"#,
                    package.name, package.name, package.name, package.name
                ));
            }
        }
        
        setup_content.push_str(r#"
# Restore previous prefix
if [ -n "$_colcon_current_prefix" ]; then
    export COLCON_CURRENT_PREFIX="$_colcon_current_prefix"
    unset _colcon_current_prefix
else
    unset COLCON_CURRENT_PREFIX
fi
"#);
        
        let final_content = setup_content.replace("{}", &install_dir.display().to_string());
        fs::write(&setup_bash, final_content)?;
        
        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&setup_bash)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&setup_bash, perms)?;
        }
        
        Ok(())
    }
}
