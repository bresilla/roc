use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::commands::work::colcon_impl::{PackageMeta, BuildType, BuildConfig};
use super::environment_manager::EnvironmentManager;

pub struct BuildExecutor<'a> {
    config: &'a BuildConfig,
    install_paths: HashMap<String, PathBuf>,
    env_manager: EnvironmentManager,
}

/// Represents the state of a package during the build process
#[derive(Debug, Clone, PartialEq)]
pub enum PackageState {
    Pending,
    Building,
    Completed,
    Failed,
}

/// Thread-safe build state manager
pub struct BuildState {
    package_states: Arc<Mutex<HashMap<String, PackageState>>>,
    install_paths: Arc<Mutex<HashMap<String, PathBuf>>>,
    build_count: Arc<Mutex<(usize, usize)>>, // (successful, failed)
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
        
        // If parallel workers is 1, use sequential execution
        if self.config.parallel_workers <= 1 {
            return self.build_sequential(packages, build_order);
        }
        
        // Use parallel execution for multiple workers
        self.build_parallel(packages, build_order)
    }
    
    /// Sequential build (original implementation)
    fn build_sequential(
        &mut self,
        packages: &[PackageMeta],
        build_order: &[usize],
    ) -> Result<(), Box<dyn std::error::Error>> {
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
    
    /// Parallel build implementation
    fn build_parallel(
        &mut self,
        packages: &[PackageMeta],
        build_order: &[usize],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let build_state = BuildState {
            package_states: Arc::new(Mutex::new(HashMap::new())),
            install_paths: Arc::new(Mutex::new(HashMap::new())),
            build_count: Arc::new(Mutex::new((0, 0))),
        };
        
        // Initialize package states
        {
            let mut states = build_state.package_states.lock().unwrap();
            for &pkg_idx in build_order {
                let package = &packages[pkg_idx];
                states.insert(package.name.clone(), PackageState::Pending);
            }
        }
        
        // Create dependency graph for packages
        let mut pkg_dependencies: HashMap<String, HashSet<String>> = HashMap::new();
        for &pkg_idx in build_order {
            let package = &packages[pkg_idx];
            let deps: HashSet<String> = package.build_deps.iter()
                .filter(|dep| build_state.package_states.lock().unwrap().contains_key(*dep))
                .cloned()
                .collect();
            pkg_dependencies.insert(package.name.clone(), deps);
        }
        
        // Worker threads
        let mut handles = Vec::new();
        let packages_arc = Arc::new(packages.to_vec());
        let dependencies_arc = Arc::new(pkg_dependencies);
        let config_arc = Arc::new(self.config.clone());
        
        for worker_id in 0..self.config.parallel_workers {
            let build_state_clone = BuildState {
                package_states: Arc::clone(&build_state.package_states),
                install_paths: Arc::clone(&build_state.install_paths),
                build_count: Arc::clone(&build_state.build_count),
            };
            let packages_clone = Arc::clone(&packages_arc);
            let dependencies_clone = Arc::clone(&dependencies_arc);
            let config_clone = Arc::clone(&config_arc);
            
            let handle = thread::spawn(move || {
                Self::worker_thread(worker_id as usize, build_state_clone, packages_clone, dependencies_clone, config_clone)
            });
            handles.push(handle);
        }
        
        // Wait for all workers to complete
        for handle in handles {
            if let Err(e) = handle.join() {
                eprintln!("Worker thread panicked: {:?}", e);
            }
        }
        
        // Update our install paths with the shared state
        {
            let shared_paths = build_state.install_paths.lock().unwrap();
            self.install_paths.extend(shared_paths.clone());
        }
        
        // Print summary
        let (successful_builds, failed_builds) = {
            let counts = build_state.build_count.lock().unwrap();
            *counts
        };
        
        println!("\nBuild Summary:");
        println!("  {} packages succeeded", successful_builds);
        if failed_builds > 0 {
            println!("  {} packages failed", failed_builds);
            
            if !self.config.continue_on_error {
                return Err("Some packages failed to build".into());
            }
        }
        
        // Generate environment setup scripts
        self.generate_setup_scripts(packages)?;
        
        Ok(())
    }
    
    /// Worker thread that builds packages when their dependencies are satisfied
    fn worker_thread(
        worker_id: usize,
        build_state: BuildState,
        packages: Arc<Vec<PackageMeta>>,
        dependencies: Arc<HashMap<String, HashSet<String>>>,
        config: Arc<BuildConfig>,
    ) -> Result<(), String> {
        let mut env_manager = EnvironmentManager::new(
            config.install_base.clone(),
            config.isolated,
        );
        
        loop {
            // Find a package that's ready to build
            let package_to_build = {
                let mut states = build_state.package_states.lock().unwrap();
                let mut ready_package = None;
                
                // Collect candidates first to avoid borrow conflicts
                let pending_packages: Vec<String> = states.iter()
                    .filter_map(|(name, state)| {
                        if *state == PackageState::Pending {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                for pkg_name in pending_packages {
                    // Check if all dependencies are completed
                    if let Some(deps) = dependencies.get(&pkg_name) {
                        let all_deps_ready = deps.iter().all(|dep| {
                            states.get(dep).map(|s| *s == PackageState::Completed).unwrap_or(true)
                        });
                        
                        if all_deps_ready {
                            states.insert(pkg_name.clone(), PackageState::Building);
                            ready_package = Some(pkg_name);
                            break;
                        }
                    }
                }
                ready_package
            };
            
            match package_to_build {
                Some(pkg_name) => {
                    // Find the package by name
                    let package = packages.iter().find(|p| p.name == pkg_name);
                    if let Some(package) = package {
                        println!("[Worker {}] Starting >>> {} ({:?})", worker_id, package.name, package.build_type);
                        let start_time = Instant::now();
                        
                        // Update environment for dependencies
                        Self::update_worker_environment(&mut env_manager, package, &build_state, &config)
                            .map_err(|e| format!("Environment setup failed: {}", e))?;
                        
                        // Build the package
                        let build_result = Self::build_package_with_env(package, &env_manager, &config)
                            .map_err(|e| format!("Build failed: {}", e));
                        
                        let duration = start_time.elapsed();
                        
                        match build_result {
                            Ok(_) => {
                                println!("[Worker {}] Finished <<< {} [{:.2}s]", worker_id, package.name, duration.as_secs_f64());
                                
                                // Record install path
                                let install_path = if config.merge_install {
                                    config.workspace_root.join("install")
                                } else {
                                    config.workspace_root.join("install").join(&package.name)
                                };
                                
                                {
                                    let mut paths = build_state.install_paths.lock().unwrap();
                                    paths.insert(package.name.clone(), install_path);
                                }
                                
                                // Mark as completed
                                {
                                    let mut states = build_state.package_states.lock().unwrap();
                                    states.insert(package.name.clone(), PackageState::Completed);
                                }
                                
                                // Update build count
                                {
                                    let mut counts = build_state.build_count.lock().unwrap();
                                    counts.0 += 1;
                                }
                            }
                            Err(e) => {
                                eprintln!("[Worker {}] Failed <<< {} - {}", worker_id, package.name, e);
                                
                                // Mark as failed
                                {
                                    let mut states = build_state.package_states.lock().unwrap();
                                    states.insert(package.name.clone(), PackageState::Failed);
                                }
                                
                                // Update build count
                                {
                                    let mut counts = build_state.build_count.lock().unwrap();
                                    counts.1 += 1;
                                }
                                
                                if !config.continue_on_error {
                                    return Err(format!("Build failed for package {}: {}", package.name, e));
                                }
                            }
                        }
                    }
                }
                None => {
                    // Check if all packages are done
                    let all_done = {
                        let states = build_state.package_states.lock().unwrap();
                        states.values().all(|state| {
                            *state == PackageState::Completed || *state == PackageState::Failed
                        })
                    };
                    
                    if all_done {
                        break;
                    }
                    
                    // Brief sleep to avoid busy waiting
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
        
        Ok(())
    }
    
    /// Update environment for a worker thread
    fn update_worker_environment(
        env_manager: &mut EnvironmentManager,
        package: &PackageMeta,
        build_state: &BuildState,
        _config: &BuildConfig,
    ) -> Result<(), String> {
        // Setup package environment
        env_manager.setup_package_environment(&package.name, &package.path)
            .map_err(|e| format!("Failed to setup package environment: {}", e))?;
        
        // Add dependencies' install paths to environment
        let install_paths = build_state.install_paths.lock().unwrap();
        for dep_name in &package.build_deps {
            if let Some(dep_path) = install_paths.get(dep_name) {
                env_manager.setup_package_environment(dep_name, dep_path)
                    .map_err(|e| format!("Failed to setup dependency environment: {}", e))?;
            }
        }
        
        Ok(())
    }
    
    /// Build a package with the given environment
    fn build_package_with_env(
        package: &PackageMeta,
        env_manager: &EnvironmentManager,
        config: &BuildConfig,
    ) -> Result<(), String> {
        match package.build_type {
            BuildType::AmentCmake | BuildType::Cmake => {
                Self::build_cmake_package_with_env(package, env_manager, config)
            }
            BuildType::AmentPython => {
                Self::build_python_package_with_env(package, env_manager, config)
            }
            BuildType::Other(ref build_type) => {
                Err(format!("Unsupported build type: {}", build_type))
            }
        }
    }
    
    /// Static version of build_cmake_package for use in worker threads
    fn build_cmake_package_with_env(
        package: &PackageMeta,
        env_manager: &EnvironmentManager,
        config: &BuildConfig,
    ) -> Result<(), String> {
        let build_dir = config.workspace_root.join("build").join(&package.name);
        let install_prefix = if config.merge_install {
            config.workspace_root.join("install")
        } else {
            config.workspace_root.join("install").join(&package.name)
        };
        
        fs::create_dir_all(&build_dir).map_err(|e| format!("Failed to create build directory: {}", e))?;
        
        // Configure
        let mut configure_cmd = Command::new("cmake");
        configure_cmd
            .arg("-S").arg(&package.path)
            .arg("-B").arg(&build_dir)
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_prefix.display()));
        
        if config.symlink_install {
            configure_cmd.arg("-DCMAKE_INSTALL_MODE=ABS_SYMLINK_FILES");
        }
        
        // Add user-provided cmake args
        configure_cmd.args(&config.cmake_args);
        
        // Set environment
        configure_cmd.envs(env_manager.get_env_vars());
        
        println!("  Configuring with CMake...");
        let configure_output = configure_cmd.output().map_err(|e| format!("Failed to run cmake configure: {}", e))?;
        if !configure_output.status.success() {
            println!("  ❌ CMake configure failed");
            println!("  stdout: {}", String::from_utf8_lossy(&configure_output.stdout));
            println!("  stderr: {}", String::from_utf8_lossy(&configure_output.stderr));
            return Err(format!(
                "CMake configure failed:\n{}",
                String::from_utf8_lossy(&configure_output.stderr)
            ));
        }
        println!("  ✅ CMake configure succeeded");
        
        // Build and install
        let mut build_cmd = Command::new("cmake");
        build_cmd
            .arg("--build").arg(&build_dir)
            .arg("--target");
        
        if let Some(ref target) = config.cmake_target {
            build_cmd.arg(target);
        } else {
            build_cmd.arg("install");
        }
        
        build_cmd
            .arg("--")
            .arg(format!("-j{}", config.parallel_workers));
        
        build_cmd.envs(env_manager.get_env_vars());
        
        println!("  Building and installing...");
        let build_output = build_cmd.output().map_err(|e| format!("Failed to run cmake build: {}", e))?;
        if !build_output.status.success() {
            println!("  ❌ CMake build failed");
            println!("  stdout: {}", String::from_utf8_lossy(&build_output.stdout));
            println!("  stderr: {}", String::from_utf8_lossy(&build_output.stderr));
            return Err(format!(
                "CMake build failed:\n{}",
                String::from_utf8_lossy(&build_output.stderr)
            ));
        }
        println!("  ✅ Build and install succeeded");
        
        Ok(())
    }
    
    /// Static version of build_python_package for use in worker threads
    fn build_python_package_with_env(
        package: &PackageMeta,
        env_manager: &EnvironmentManager,
        config: &BuildConfig,
    ) -> Result<(), String> {
        let build_dir = config.workspace_root.join("build").join(&package.name);
        let install_prefix = if config.merge_install {
            config.workspace_root.join("install")
        } else {
            config.workspace_root.join("install").join(&package.name)
        };
        
        fs::create_dir_all(&build_dir).map_err(|e| format!("Failed to create build directory: {}", e))?;
        
        // Build
        let build_output = Command::new("python3")
            .arg("setup.py")
            .arg("build")
            .arg("--build-base").arg(&build_dir)
            .current_dir(&package.path)
            .envs(env_manager.get_env_vars())
            .output()
            .map_err(|e| format!("Failed to run python build: {}", e))?;
            
        if !build_output.status.success() {
            return Err(format!(
                "Python build failed:\n{}",
                String::from_utf8_lossy(&build_output.stderr)
            ));
        }
        
        // Install
        let install_output = Command::new("python3")
            .arg("setup.py")
            .arg("install")
            .arg("--prefix").arg("")
            .arg("--root").arg(&install_prefix)
            .current_dir(&package.path)
            .envs(env_manager.get_env_vars())
            .output()
            .map_err(|e| format!("Failed to run python install: {}", e))?;
            
        if !install_output.status.success() {
            return Err(format!(
                "Python install failed:\n{}",
                String::from_utf8_lossy(&install_output.stderr)
            ));
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
}
