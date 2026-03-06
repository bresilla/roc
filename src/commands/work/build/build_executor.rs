use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use super::environment_manager::EnvironmentManager;
use crate::commands::work::build::{BuildConfig, BuildType, PackageMeta};

pub struct BuildExecutor<'a> {
    config: &'a BuildConfig,
    install_paths: HashMap<String, PathBuf>,
    #[allow(dead_code)]
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
        let env_manager = EnvironmentManager::new(config.install_base.clone(), config.isolated);

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

        // If parallel workers is 1, use sequential execution with filtered environment
        // Note: We use build_sequential_filtered instead of build_sequential to avoid
        // environment accumulation issues that can cause CMake to hang
        if self.config.parallel_workers <= 1 {
            return self.build_sequential_filtered(packages, build_order);
        }

        // Use parallel execution for multiple workers
        self.build_parallel(packages, build_order)
    }

    // Note: Legacy sequential build method removed to clean up unused code.
    // The active implementation uses build_sequential_filtered for sequential builds.

    /// Sequential build using filtered environment (fixed version)
    fn build_sequential_filtered(
        &mut self,
        packages: &[PackageMeta],
        build_order: &[usize],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut successful_builds = 0;
        let mut failed_builds = 0;

        for &pkg_idx in build_order {
            let package = &packages[pkg_idx];

            println!(
                "{} {} {}",
                "Starting >>>".bright_cyan().bold(),
                package.name.bright_white().bold(),
                format!("({:?})", package.build_type).bright_black()
            );
            let start_time = Instant::now();

            // Create a fresh environment manager for this package (like parallel build does)
            let mut package_env_manager =
                EnvironmentManager::new(self.config.install_base.clone(), self.config.isolated);

            // Setup environment for this package
            package_env_manager.setup_package_environment(&package.name, &package.path)?;

            // Add dependencies' install paths to environment
            for dep_name in package.build_order_deps() {
                if let Some(dep_path) = self.install_paths.get(&dep_name) {
                    package_env_manager.setup_package_environment(&dep_name, dep_path)?;
                }
            }

            // Build using the clean environment (like parallel build does)
            let build_result: Result<(), Box<dyn std::error::Error>> = match package.build_type {
                BuildType::AmentCmake | BuildType::Cmake => {
                    Self::build_cmake_package_with_env(package, &package_env_manager, self.config)
                        .map_err(|e| e.into())
                }
                BuildType::AmentPython => {
                    Self::build_python_package_with_env(package, &package_env_manager, self.config)
                        .map_err(|e| e.into())
                }
                BuildType::Other(ref build_type) => {
                    Err(format!("Unsupported build type: {}", build_type).into())
                }
            };

            match build_result {
                Ok(_) => {
                    let duration = start_time.elapsed();
                    println!(
                        "{} {} {}",
                        "Finished <<<".bright_green().bold(),
                        package.name.bright_white().bold(),
                        format!("[{:.2}s]", duration.as_secs_f64()).bright_black()
                    );

                    // Record install path for environment setup
                    let install_path = if self.config.merge_install {
                        self.config.install_base.clone()
                    } else {
                        self.config.install_base.join(&package.name)
                    };
                    self.install_paths
                        .insert(package.name.clone(), install_path.clone());

                    successful_builds += 1;
                }
                Err(e) => {
                    eprintln!(
                        "{} {} - {}",
                        "Failed <<<".bright_red().bold(),
                        package.name.bright_white().bold(),
                        e.to_string().bright_white()
                    );
                    failed_builds += 1;

                    if !self.config.continue_on_error {
                        return Err(
                            format!("Build failed for package {}: {}", package.name, e).into()
                        );
                    }
                }
            }
        }

        println!("\n{}", "Build Summary".bright_cyan().bold());
        println!(
            "  {} {}",
            successful_builds.to_string().bright_green().bold(),
            "packages succeeded".bright_green()
        );
        if failed_builds > 0 {
            println!(
                "  {} {}",
                failed_builds.to_string().bright_red().bold(),
                "packages failed".bright_red()
            );
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
            let deps: HashSet<String> = package
                .build_order_deps()
                .into_iter()
                .filter(|dep| {
                    build_state
                        .package_states
                        .lock()
                        .unwrap()
                        .contains_key(dep)
                })
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
                Self::worker_thread(
                    worker_id as usize,
                    build_state_clone,
                    packages_clone,
                    dependencies_clone,
                    config_clone,
                )
            });
            handles.push(handle);
        }

        // Wait for all workers to complete
        for handle in handles {
            if let Err(e) = handle.join() {
                eprintln!("{} {:?}", "Worker thread panicked:".bright_red().bold(), e);
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

        println!("\n{}", "Build Summary".bright_cyan().bold());
        println!(
            "  {} {}",
            successful_builds.to_string().bright_green().bold(),
            "packages succeeded".bright_green()
        );
        if failed_builds > 0 {
            println!(
                "  {} {}",
                failed_builds.to_string().bright_red().bold(),
                "packages failed".bright_red()
            );

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
        let mut env_manager = EnvironmentManager::new(config.install_base.clone(), config.isolated);

        loop {
            // Find a package that's ready to build
            let package_to_build = {
                let mut states = build_state.package_states.lock().unwrap();
                let mut ready_package = None;

                // Collect candidates first to avoid borrow conflicts
                let pending_packages: Vec<String> = states
                    .iter()
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
                            states
                                .get(dep)
                                .map(|s| *s == PackageState::Completed)
                                .unwrap_or(true)
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
                        println!(
                            "{} {} {} {}",
                            format!("[Worker {}]", worker_id).bright_black(),
                            "Starting >>>".bright_cyan().bold(),
                            package.name.bright_white().bold(),
                            format!("({:?})", package.build_type).bright_black()
                        );
                        let start_time = Instant::now();

                        // Update environment for dependencies
                        Self::update_worker_environment(
                            &mut env_manager,
                            package,
                            &build_state,
                            &config,
                        )
                        .map_err(|e| format!("Environment setup failed: {}", e))?;

                        // Build the package
                        let build_result =
                            Self::build_package_with_env(package, &env_manager, &config)
                                .map_err(|e| format!("Build failed: {}", e));

                        let duration = start_time.elapsed();

                        match build_result {
                            Ok(_) => {
                                println!(
                                    "{} {} {} {}",
                                    format!("[Worker {}]", worker_id).bright_black(),
                                    "Finished <<<".bright_green().bold(),
                                    package.name.bright_white().bold(),
                                    format!("[{:.2}s]", duration.as_secs_f64()).bright_black()
                                );

                                // Record install path
                                let install_path = if config.merge_install {
                                    config.install_base.clone()
                                } else {
                                    config.install_base.join(&package.name)
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
                                eprintln!(
                                    "{} {} {} - {}",
                                    format!("[Worker {}]", worker_id).bright_black(),
                                    "Failed <<<".bright_red().bold(),
                                    package.name.bright_white().bold(),
                                    e.to_string().bright_white()
                                );

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
                                    return Err(format!(
                                        "Build failed for package {}: {}",
                                        package.name, e
                                    ));
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
        env_manager
            .setup_package_environment(&package.name, &package.path)
            .map_err(|e| format!("Failed to setup package environment: {}", e))?;

        // Add dependencies' install paths to environment
        let install_paths = build_state.install_paths.lock().unwrap();
        for dep_name in package.build_order_deps() {
            if let Some(dep_path) = install_paths.get(&dep_name) {
                env_manager
                    .setup_package_environment(&dep_name, dep_path)
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
        let build_dir = config.build_base.join(&package.name);
        let install_prefix = if config.merge_install {
            config.install_base.clone()
        } else {
            config.install_base.join(&package.name)
        };

        fs::create_dir_all(&build_dir)
            .map_err(|e| format!("Failed to create build directory: {}", e))?;

        // Configure
        let mut configure_cmd = Command::new("cmake");
        configure_cmd
            .arg("-S")
            .arg(&package.path)
            .arg("-B")
            .arg(&build_dir)
            .arg(format!(
                "-DCMAKE_INSTALL_PREFIX={}",
                install_prefix.display()
            ));

        if config.symlink_install {
            configure_cmd.arg("-DCMAKE_INSTALL_MODE=ABS_SYMLINK_FILES");
        }

        // Add user-provided cmake args
        configure_cmd.args(&config.cmake_args);

        // Set environment
        configure_cmd.envs(env_manager.get_env_vars());

        println!("  {}", "Configuring with CMake...".bright_blue());
        let configure_output = configure_cmd
            .output()
            .map_err(|e| format!("Failed to run cmake configure: {}", e))?;
        if !configure_output.status.success() {
            println!("  {}", "CMake configure failed".bright_red().bold());
            println!(
                "  stdout: {}",
                String::from_utf8_lossy(&configure_output.stdout)
            );
            println!(
                "  stderr: {}",
                String::from_utf8_lossy(&configure_output.stderr)
            );
            return Err(format!(
                "CMake configure failed:\n{}",
                String::from_utf8_lossy(&configure_output.stderr)
            ));
        }
        println!("  {}", "CMake configure succeeded".bright_green());

        // Build and install
        let mut build_cmd = Command::new("cmake");
        build_cmd.arg("--build").arg(&build_dir).arg("--target");

        if let Some(ref target) = config.cmake_target {
            build_cmd.arg(target);
        } else {
            build_cmd.arg("install");
        }

        build_cmd
            .arg("--")
            .arg(format!("-j{}", config.parallel_workers));

        build_cmd.envs(env_manager.get_env_vars());

        println!("  {}", "Building and installing...".bright_blue());
        let build_output = build_cmd
            .output()
            .map_err(|e| format!("Failed to run cmake build: {}", e))?;
        if !build_output.status.success() {
            println!("  {}", "CMake build failed".bright_red().bold());
            println!(
                "  stdout: {}",
                String::from_utf8_lossy(&build_output.stdout)
            );
            println!(
                "  stderr: {}",
                String::from_utf8_lossy(&build_output.stderr)
            );
            return Err(format!(
                "CMake build failed:\n{}",
                String::from_utf8_lossy(&build_output.stderr)
            ));
        }
        println!("  {}", "Build and install succeeded".bright_green());

        Ok(())
    }

    /// Static version of build_python_package for use in worker threads
    fn build_python_package_with_env(
        package: &PackageMeta,
        env_manager: &EnvironmentManager,
        config: &BuildConfig,
    ) -> Result<(), String> {
        let build_dir = config.build_base.join(&package.name);
        let install_prefix = if config.merge_install {
            config.install_base.clone()
        } else {
            config.install_base.join(&package.name)
        };

        fs::create_dir_all(&build_dir)
            .map_err(|e| format!("Failed to create build directory: {}", e))?;

        // Build
        let build_output = Command::new("python3")
            .arg("setup.py")
            .arg("build")
            .arg("--build-base")
            .arg(&build_dir)
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
            .arg("--prefix")
            .arg("")
            .arg("--root")
            .arg(&install_prefix)
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

    fn generate_setup_scripts(
        &self,
        packages: &[PackageMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.generate_package_metadata_files(packages)?;
        self.generate_package_setup_scripts(packages)?;

        let install_dir = self.config.install_base.clone();
        self.generate_workspace_setup_scripts(&install_dir, packages)?;

        Ok(())
    }

    fn generate_package_setup_scripts(
        &self,
        packages: &[PackageMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for package in packages {
            let Some(pkg_install_path) = self.install_paths.get(&package.name) else {
                continue;
            };

            let package_share_dir = pkg_install_path.join("share").join(&package.name);
            fs::create_dir_all(&package_share_dir)?;

            let package_sh = package_share_dir.join("package.sh");
            let package_bash = package_share_dir.join("package.bash");
            let package_zsh = package_share_dir.join("package.zsh");
            let local_setup_sh = package_share_dir.join("local_setup.sh");
            let local_setup_bash = package_share_dir.join("local_setup.bash");
            let local_setup_zsh = package_share_dir.join("local_setup.zsh");

            fs::write(
                &local_setup_sh,
                self.render_local_setup_sh(&package.name, pkg_install_path),
            )?;
            fs::write(
                &local_setup_bash,
                self.render_shell_wrapper("bash", &local_setup_sh),
            )?;
            fs::write(
                &local_setup_zsh,
                self.render_shell_wrapper("zsh", &local_setup_sh),
            )?;

            fs::write(
                &package_sh,
                self.render_package_sh(&package.name, pkg_install_path),
            )?;
            fs::write(
                &package_bash,
                self.render_shell_wrapper("bash", &package_sh),
            )?;
            fs::write(
                &package_zsh,
                self.render_shell_wrapper("zsh", &package_sh),
            )?;

            Self::make_executable_if_unix(&local_setup_sh)?;
            Self::make_executable_if_unix(&local_setup_bash)?;
            Self::make_executable_if_unix(&local_setup_zsh)?;
            Self::make_executable_if_unix(&package_sh)?;
            Self::make_executable_if_unix(&package_bash)?;
            Self::make_executable_if_unix(&package_zsh)?;
        }

        Ok(())
    }

    fn generate_package_metadata_files(
        &self,
        packages: &[PackageMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_dir = self
            .config
            .install_base
            .join("share")
            .join("colcon-core")
            .join("packages");
        fs::create_dir_all(&metadata_dir)?;

        let built_packages: HashSet<&str> = self.install_paths.keys().map(|name| name.as_str()).collect();
        for package in packages {
            if !self.install_paths.contains_key(&package.name) {
                continue;
            }

            let runtime_deps = package
                .runtime_deps()
                .into_iter()
                .filter(|dep| built_packages.contains(dep.as_str()))
                .collect::<Vec<_>>();
            let metadata_path = metadata_dir.join(&package.name);
            let metadata_contents = runtime_deps.join("\n");
            fs::write(metadata_path, metadata_contents)?;
        }

        Ok(())
    }

    fn generate_workspace_setup_scripts(
        &self,
        install_dir: &Path,
        packages: &[PackageMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(install_dir)?;

        let local_setup_sh = install_dir.join("local_setup.sh");
        let local_setup_bash = install_dir.join("local_setup.bash");
        let local_setup_zsh = install_dir.join("local_setup.zsh");
        let setup_sh = install_dir.join("setup.sh");
        let setup_bash = install_dir.join("setup.bash");
        let setup_zsh = install_dir.join("setup.zsh");

        fs::write(
            &local_setup_sh,
            self.render_workspace_local_setup_sh(install_dir, packages),
        )?;
        fs::write(
            &local_setup_bash,
            self.render_shell_wrapper("bash", &local_setup_sh),
        )?;
        fs::write(
            &local_setup_zsh,
            self.render_shell_wrapper("zsh", &local_setup_sh),
        )?;
        fs::write(&setup_sh, self.render_workspace_setup_sh(install_dir))?;
        fs::write(&setup_bash, self.render_shell_wrapper("bash", &setup_sh))?;
        fs::write(&setup_zsh, self.render_shell_wrapper("zsh", &setup_sh))?;

        Self::make_executable_if_unix(&local_setup_sh)?;
        Self::make_executable_if_unix(&local_setup_bash)?;
        Self::make_executable_if_unix(&local_setup_zsh)?;
        Self::make_executable_if_unix(&setup_sh)?;
        Self::make_executable_if_unix(&setup_bash)?;
        Self::make_executable_if_unix(&setup_zsh)?;

        Ok(())
    }

    fn create_workspace_directories(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = &self.config.build_base;
        let install_dir = &self.config.install_base;
        let log_dir = &self.config.log_base;
        let latest_log_dir = log_dir.join("latest");

        fs::create_dir_all(build_dir)?;
        fs::create_dir_all(install_dir)?;
        fs::create_dir_all(log_dir)?;
        fs::create_dir_all(&latest_log_dir)?;

        Self::write_colcon_ignore(build_dir)?;
        Self::write_colcon_ignore(install_dir)?;
        Self::write_colcon_ignore(log_dir)?;
        Self::write_colcon_ignore(&latest_log_dir)?;

        Ok(())
    }

    fn write_colcon_ignore(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(dir.join("COLCON_IGNORE"), "")?;
        Ok(())
    }

    fn render_local_setup_sh(&self, package_name: &str, prefix: &Path) -> String {
        let prefix_str = prefix.display();
        format!(
            r#"#!/bin/sh
# Generated local setup script for package {package_name}

_colcon_prepend_unique_value() {{
    var_name="$1"
    value="$2"

    eval current_value="\${{${{var_name}}:-}}"
    case ":$current_value:" in
        *":$value:"*) ;;
        "")
            eval export "$var_name=$value"
            ;;
        *)
            eval export "$var_name=$value:$current_value"
            ;;
    esac
}}

_colcon_prepend_unique_value CMAKE_PREFIX_PATH "{prefix_str}"
_colcon_prepend_unique_value AMENT_PREFIX_PATH "{prefix_str}"

if [ -d "{prefix_str}/bin" ]; then
    _colcon_prepend_unique_value PATH "{prefix_str}/bin"
fi

if [ -d "{prefix_str}/lib" ]; then
    _colcon_prepend_unique_value LD_LIBRARY_PATH "{prefix_str}/lib"
fi

if [ -d "{prefix_str}/lib/python3.10/site-packages" ]; then
    _colcon_prepend_unique_value PYTHONPATH "{prefix_str}/lib/python3.10/site-packages"
fi
"#
        )
    }

    fn render_package_sh(&self, package_name: &str, prefix: &Path) -> String {
        let local_setup_path = prefix.join("share").join(package_name).join("local_setup.sh");
        format!(
            r#"#!/bin/sh
# Generated package setup script for package {package_name}

_colcon_package_old_prefix="${{COLCON_CURRENT_PREFIX:-}}"
export COLCON_CURRENT_PREFIX="{prefix}"

if [ -f "{local_setup}" ]; then
    . "{local_setup}"
fi

if [ -n "$_colcon_package_old_prefix" ]; then
    export COLCON_CURRENT_PREFIX="$_colcon_package_old_prefix"
else
    unset COLCON_CURRENT_PREFIX
fi
unset _colcon_package_old_prefix
"#,
            prefix = prefix.display(),
            local_setup = local_setup_path.display(),
        )
    }

    fn render_shell_wrapper(&self, shell_name: &str, target_path: &Path) -> String {
        let shebang = match shell_name {
            "sh" => "#!/bin/sh",
            "bash" => "#!/bin/bash",
            "zsh" => "#!/bin/zsh",
            _ => "#!/bin/sh",
        };

        format!(
            r#"{shebang}
. "{target}"
"#,
            target = target_path.display(),
        )
    }

    fn render_workspace_local_setup_sh(
        &self,
        install_dir: &Path,
        packages: &[PackageMeta],
    ) -> String {
        let mut content = format!(
            r#"#!/bin/sh
# Generated workspace local setup script

export COLCON_CURRENT_PREFIX="{install_prefix}"

"#,
            install_prefix = install_dir.display()
        );

        for package in packages {
            let Some(package_prefix) = self.install_paths.get(&package.name) else {
                continue;
            };
            let package_script = package_prefix
                .join("share")
                .join(&package.name)
                .join("package.sh");
            content.push_str(&format!(
                r#"if [ -f "{package_script}" ]; then
    . "{package_script}"
fi
"#,
                package_script = package_script.display()
            ));
        }

        content
    }

    fn render_workspace_setup_sh(&self, install_dir: &Path) -> String {
        format!(
            r#"#!/bin/sh
# Generated workspace setup script

_colcon_prefix_chain_source_script() {{
    script_path="$1"
    if [ -f "$script_path" ]; then
        . "$script_path"
    fi
}}

_colcon_prepend_unique_value() {{
    var_name="$1"
    value="$2"

    eval current_value="\${{${{var_name}}:-}}"
    case ":$current_value:" in
        *":$value:"*) ;;
        "")
            eval export "$var_name=$value"
            ;;
        *)
            eval export "$var_name=$value:$current_value"
            ;;
    esac
}}

_colcon_workspace_prefix="{install_prefix}"
_colcon_previous_prefixes="${{COLCON_PREFIX_PATH:-}}"

if [ -n "$_colcon_previous_prefixes" ]; then
    _colcon_old_ifs="$IFS"
    IFS=':'
    for _colcon_prefix in $_colcon_previous_prefixes; do
        if [ -n "$_colcon_prefix" ] && [ "$_colcon_prefix" != "$_colcon_workspace_prefix" ]; then
            _colcon_prefix_chain_source_script "$_colcon_prefix/local_setup.sh"
        fi
    done
    IFS="$_colcon_old_ifs"
    unset _colcon_old_ifs
fi

_colcon_prefix_chain_source_script "{local_setup}"
_colcon_prepend_unique_value COLCON_PREFIX_PATH "$_colcon_workspace_prefix"
unset _colcon_workspace_prefix
unset _colcon_previous_prefixes
unset _colcon_prefix
"#,
            install_prefix = install_dir.display(),
            local_setup = install_dir.join("local_setup.sh").display(),
        )
    }

    fn make_executable_if_unix(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    // Legacy unused methods removed to clean up compilation warnings.
    // The active implementation uses *_with_env methods for both sequential
    // and parallel builds to ensure proper environment isolation.
}

#[cfg(test)]
mod tests {
    use super::BuildExecutor;
    use crate::commands::work::build::{BuildConfig, BuildType, PackageMeta};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn create_workspace_directories_respects_custom_bases_and_ignore_markers() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.build_base = workspace_root.join("artifacts").join("build-tree");
        config.install_base = workspace_root.join("artifacts").join("install-tree");
        config.log_base = workspace_root.join("artifacts").join("log-tree");

        let executor = BuildExecutor::new(&config);
        executor.create_workspace_directories().unwrap();

        assert!(config.build_base.exists());
        assert!(config.install_base.exists());
        assert!(config.log_base.exists());
        assert!(config.log_base.join("latest").exists());
        assert!(config.build_base.join("COLCON_IGNORE").exists());
        assert!(config.install_base.join("COLCON_IGNORE").exists());
        assert!(config.log_base.join("COLCON_IGNORE").exists());
        assert!(config.log_base.join("latest").join("COLCON_IGNORE").exists());
    }

    #[test]
    fn generate_package_metadata_files_writes_workspace_runtime_dependencies() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");

        let mut executor = BuildExecutor::new(&config);
        executor
            .install_paths
            .insert("base_pkg".to_string(), config.install_base.join("base_pkg"));
        executor.install_paths.insert(
            "consumer_pkg".to_string(),
            config.install_base.join("consumer_pkg"),
        );

        let packages = vec![
            PackageMeta {
                name: "base_pkg".to_string(),
                path: PathBuf::from("/tmp/base_pkg"),
                build_type: BuildType::AmentCmake,
                version: "0.1.0".to_string(),
                description: "base".to_string(),
                maintainers: vec!["Fixture".to_string()],
                depend_deps: Vec::new(),
                build_deps: Vec::new(),
                buildtool_deps: Vec::new(),
                build_export_deps: Vec::new(),
                exec_deps: Vec::new(),
                test_deps: Vec::new(),
            },
            PackageMeta {
                name: "consumer_pkg".to_string(),
                path: PathBuf::from("/tmp/consumer_pkg"),
                build_type: BuildType::AmentCmake,
                version: "0.1.0".to_string(),
                description: "consumer".to_string(),
                maintainers: vec!["Fixture".to_string()],
                depend_deps: vec!["base_pkg".to_string()],
                build_deps: Vec::new(),
                buildtool_deps: Vec::new(),
                build_export_deps: vec!["external_pkg".to_string()],
                exec_deps: vec!["base_pkg".to_string()],
                test_deps: Vec::new(),
            },
        ];

        executor.generate_package_metadata_files(&packages).unwrap();

        let metadata_root = config.install_base.join("share/colcon-core/packages");
        assert_eq!(
            std::fs::read_to_string(metadata_root.join("base_pkg")).unwrap(),
            ""
        );
        assert_eq!(
            std::fs::read_to_string(metadata_root.join("consumer_pkg")).unwrap(),
            "base_pkg"
        );
    }

    #[test]
    fn generate_package_setup_scripts_writes_package_and_local_setup_files() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");

        let mut executor = BuildExecutor::new(&config);
        let package_prefix = config.install_base.join("demo_pkg");
        executor
            .install_paths
            .insert("demo_pkg".to_string(), package_prefix.clone());

        let packages = vec![PackageMeta {
            name: "demo_pkg".to_string(),
            path: PathBuf::from("/tmp/demo_pkg"),
            build_type: BuildType::AmentCmake,
            version: "0.1.0".to_string(),
            description: "demo".to_string(),
            maintainers: vec!["Fixture".to_string()],
            depend_deps: Vec::new(),
            build_deps: Vec::new(),
            buildtool_deps: Vec::new(),
            build_export_deps: Vec::new(),
            exec_deps: Vec::new(),
            test_deps: Vec::new(),
        }];

        executor.generate_package_setup_scripts(&packages).unwrap();

        let share_dir = package_prefix.join("share").join("demo_pkg");
        assert!(share_dir.join("package.sh").exists());
        assert!(share_dir.join("package.bash").exists());
        assert!(share_dir.join("package.zsh").exists());
        assert!(share_dir.join("local_setup.sh").exists());
        assert!(share_dir.join("local_setup.bash").exists());
        assert!(share_dir.join("local_setup.zsh").exists());

        let package_sh = std::fs::read_to_string(share_dir.join("package.sh")).unwrap();
        assert!(package_sh.contains("COLCON_CURRENT_PREFIX"));
        assert!(package_sh.contains("local_setup.sh"));

        let local_setup_sh = std::fs::read_to_string(share_dir.join("local_setup.sh")).unwrap();
        assert!(local_setup_sh.contains("CMAKE_PREFIX_PATH"));
        assert!(local_setup_sh.contains("AMENT_PREFIX_PATH"));
        assert!(local_setup_sh.contains(&package_prefix.display().to_string()));
    }

    #[test]
    fn generate_workspace_setup_scripts_writes_local_and_overlay_setup_variants() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");

        let mut executor = BuildExecutor::new(&config);
        let package_prefix = config.install_base.join("demo_pkg");
        executor
            .install_paths
            .insert("demo_pkg".to_string(), package_prefix.clone());

        let packages = vec![PackageMeta {
            name: "demo_pkg".to_string(),
            path: PathBuf::from("/tmp/demo_pkg"),
            build_type: BuildType::AmentCmake,
            version: "0.1.0".to_string(),
            description: "demo".to_string(),
            maintainers: vec!["Fixture".to_string()],
            depend_deps: Vec::new(),
            build_deps: Vec::new(),
            buildtool_deps: Vec::new(),
            build_export_deps: Vec::new(),
            exec_deps: Vec::new(),
            test_deps: Vec::new(),
        }];

        executor.generate_workspace_setup_scripts(&config.install_base, &packages).unwrap();

        assert!(config.install_base.join("local_setup.sh").exists());
        assert!(config.install_base.join("local_setup.bash").exists());
        assert!(config.install_base.join("local_setup.zsh").exists());
        assert!(config.install_base.join("setup.sh").exists());
        assert!(config.install_base.join("setup.bash").exists());
        assert!(config.install_base.join("setup.zsh").exists());

        let local_setup = std::fs::read_to_string(config.install_base.join("local_setup.sh")).unwrap();
        assert!(local_setup.contains("package.sh"));
        assert!(local_setup.contains("COLCON_CURRENT_PREFIX"));

        let setup_sh = std::fs::read_to_string(config.install_base.join("setup.sh")).unwrap();
        assert!(setup_sh.contains("COLCON_PREFIX_PATH"));
        assert!(setup_sh.contains("local_setup.sh"));
    }
}
