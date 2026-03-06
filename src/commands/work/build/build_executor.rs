use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    build_records: Arc<Mutex<HashMap<String, BuildRecord>>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct InstalledArtifacts {
    bin_dirs: Vec<PathBuf>,
    lib_dirs: Vec<PathBuf>,
    python_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DsvOperation {
    PrependNonDuplicate { name: String, value: String },
    PrependNonDuplicateIfExists { name: String, value: String },
    Set { name: String, value: String },
    SetIfUnset { name: String, value: String },
    Source { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BuildRecord {
    status: PackageState,
    duration_ms: u128,
    error: Option<String>,
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
        let mut build_records = HashMap::new();

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

                    build_records.insert(
                        package.name.clone(),
                        BuildRecord {
                            status: PackageState::Completed,
                            duration_ms: duration.as_millis(),
                            error: None,
                        },
                    );
                    Self::write_package_state(
                        self.config,
                        &package.name,
                        &BuildRecord {
                            status: PackageState::Completed,
                            duration_ms: duration.as_millis(),
                            error: None,
                        },
                    )?;
                    successful_builds += 1;
                }
                Err(e) => {
                    let duration = start_time.elapsed();
                    eprintln!(
                        "{} {} - {}",
                        "Failed <<<".bright_red().bold(),
                        package.name.bright_white().bold(),
                        e.to_string().bright_white()
                    );
                    build_records.insert(
                        package.name.clone(),
                        BuildRecord {
                            status: PackageState::Failed,
                            duration_ms: duration.as_millis(),
                            error: Some(e.to_string()),
                        },
                    );
                    Self::write_package_state(
                        self.config,
                        &package.name,
                        &BuildRecord {
                            status: PackageState::Failed,
                            duration_ms: duration.as_millis(),
                            error: Some(e.to_string()),
                        },
                    )?;
                    failed_builds += 1;

                    if !self.config.continue_on_error {
                        self.write_build_summary(packages, &build_records)?;
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

        self.write_build_summary(packages, &build_records)?;

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
            build_records: Arc::new(Mutex::new(HashMap::new())),
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
                .filter(|dep| build_state.package_states.lock().unwrap().contains_key(dep))
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
                build_records: Arc::clone(&build_state.build_records),
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
        {
            let records = build_state.build_records.lock().unwrap();
            self.write_build_summary(packages, &records)?;
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
                                {
                                    let mut records = build_state.build_records.lock().unwrap();
                                    let record = BuildRecord {
                                        status: PackageState::Completed,
                                        duration_ms: duration.as_millis(),
                                        error: None,
                                    };
                                    records.insert(package.name.clone(), record.clone());
                                    Self::write_package_state(&config, &package.name, &record)
                                        .map_err(|e| {
                                            format!(
                                                "Failed to persist build state for {}: {}",
                                                package.name, e
                                            )
                                        })?;
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
                                {
                                    let mut records = build_state.build_records.lock().unwrap();
                                    let record = BuildRecord {
                                        status: PackageState::Failed,
                                        duration_ms: duration.as_millis(),
                                        error: Some(e.to_string()),
                                    };
                                    records.insert(package.name.clone(), record.clone());
                                    Self::write_package_state(&config, &package.name, &record)
                                        .map_err(|e| {
                                            format!(
                                                "Failed to persist build state for {}: {}",
                                                package.name, e
                                            )
                                        })?;
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

        let mut configure_cmd = Command::new("cmake");
        configure_cmd.args(Self::cmake_configure_args(
            &package.path,
            &build_dir,
            &install_prefix,
            config.symlink_install,
            &config.cmake_args,
        ));

        configure_cmd.envs(env_manager.get_env_vars());

        println!("  {}", "Configuring with CMake...".bright_blue());
        Self::run_command_checked(
            configure_cmd,
            "CMake configure",
            &Self::phase_log_path(config, &package.name, "configure"),
        )?;
        println!("  {}", "CMake configure succeeded".bright_green());

        let mut build_cmd = Command::new("cmake");
        build_cmd.args(Self::cmake_build_args(
            &build_dir,
            config.parallel_workers,
            config.cmake_target.as_deref(),
        ));
        build_cmd.envs(env_manager.get_env_vars());

        println!("  {}", "Building with CMake...".bright_blue());
        Self::run_command_checked(
            build_cmd,
            "CMake build",
            &Self::phase_log_path(config, &package.name, "build"),
        )?;
        println!("  {}", "CMake build succeeded".bright_green());

        let mut install_cmd = Command::new("cmake");
        install_cmd.args(Self::cmake_install_args(&build_dir, &install_prefix));
        install_cmd.envs(env_manager.get_env_vars());

        println!("  {}", "Installing with CMake...".bright_blue());
        Self::run_command_checked(
            install_cmd,
            "CMake install",
            &Self::phase_log_path(config, &package.name, "install"),
        )?;
        println!("  {}", "CMake install succeeded".bright_green());

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

        let mut build_cmd = Command::new("python3");
        build_cmd
            .args(Self::python_build_args(&build_dir))
            .current_dir(&package.path)
            .envs(env_manager.get_env_vars());
        Self::run_command_checked(
            build_cmd,
            "Python build",
            &Self::phase_log_path(config, &package.name, "build"),
        )?;

        let mut install_cmd = Command::new("python3");
        install_cmd
            .args(Self::python_install_args(&build_dir, &install_prefix))
            .current_dir(&package.path)
            .envs(env_manager.get_env_vars());
        Self::run_command_checked(
            install_cmd,
            "Python install",
            &Self::phase_log_path(config, &package.name, "install"),
        )?;

        Self::normalize_python_install_layout(&install_prefix)
            .map_err(|e| format!("Failed to normalize Python install layout: {}", e))?;

        if config.symlink_install {
            Self::apply_python_symlink_install(package, &build_dir, &install_prefix)
                .map_err(|e| format!("Failed to apply symlink install: {}", e))?;
        }

        Ok(())
    }

    fn apply_python_symlink_install(
        package: &PackageMeta,
        _build_dir: &Path,
        install_prefix: &Path,
    ) -> Result<(), io::Error> {
        let source_package_dir = package.path.join(&package.name);
        let install_package_dir =
            Self::find_python_package_install_dir(install_prefix, &package.name);

        if let Some(install_dir) = install_package_dir {
            Self::replace_with_symlink(&install_dir, &source_package_dir)?;
        }

        let source_resource_marker = package.path.join("resource").join(&package.name);
        let install_resource_marker = install_prefix
            .join("share")
            .join("ament_index")
            .join("resource_index")
            .join("packages")
            .join(&package.name);
        if source_resource_marker.exists() && install_resource_marker.exists() {
            Self::replace_with_symlink(&install_resource_marker, &source_resource_marker)?;
        }

        let source_package_xml = package.path.join("package.xml");
        let install_package_xml = install_prefix
            .join("share")
            .join(&package.name)
            .join("package.xml");
        if source_package_xml.exists() && install_package_xml.exists() {
            Self::replace_with_symlink(&install_package_xml, &source_package_xml)?;
        }

        Ok(())
    }

    fn normalize_python_install_layout(install_prefix: &Path) -> Result<(), io::Error> {
        let local_lib_dir = install_prefix.join("local").join("lib");
        if local_lib_dir.is_dir() {
            let entries = match fs::read_dir(&local_lib_dir) {
                Ok(entries) => entries,
                Err(_) => return Ok(()),
            };

            for entry in entries.flatten() {
                let local_python_dir = entry.path();
                if !local_python_dir.is_dir() {
                    continue;
                }
                let Some(dir_name) = local_python_dir.file_name().and_then(|name| name.to_str())
                else {
                    continue;
                };
                if !dir_name.starts_with("python") {
                    continue;
                }

                let dist_packages = local_python_dir.join("dist-packages");
                if !dist_packages.is_dir() {
                    continue;
                }

                let site_packages = install_prefix
                    .join("lib")
                    .join(dir_name)
                    .join("site-packages");
                Self::merge_directory_contents(&dist_packages, &site_packages)?;
                fs::remove_dir_all(&dist_packages)?;
                Self::remove_empty_parent_chain(&local_python_dir, &install_prefix.join("local"))?;
            }
        }

        let local_share_dir = install_prefix.join("local").join("share");
        if local_share_dir.is_dir() {
            let share_dir = install_prefix.join("share");
            Self::merge_directory_contents(&local_share_dir, &share_dir)?;
            fs::remove_dir_all(&local_share_dir)?;
            Self::remove_empty_parent_chain(&install_prefix.join("local"), &install_prefix)?;
        }

        Ok(())
    }

    fn merge_directory_contents(source: &Path, destination: &Path) -> Result<(), io::Error> {
        fs::create_dir_all(destination)?;

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());

            if destination_path.exists() {
                let source_meta = fs::metadata(&source_path)?;
                let destination_meta = fs::metadata(&destination_path)?;
                if source_meta.is_dir() && destination_meta.is_dir() {
                    Self::merge_directory_contents(&source_path, &destination_path)?;
                    fs::remove_dir_all(&source_path)?;
                } else {
                    fs::remove_file(&destination_path)?;
                    fs::rename(&source_path, &destination_path)?;
                }
            } else {
                fs::rename(&source_path, &destination_path)?;
            }
        }

        Ok(())
    }

    fn remove_empty_parent_chain(path: &Path, stop_at: &Path) -> Result<(), io::Error> {
        let mut current = path.to_path_buf();
        while current.starts_with(stop_at) && current != stop_at {
            if fs::read_dir(&current)?.next().is_some() {
                break;
            }
            fs::remove_dir(&current)?;
            let Some(parent) = current.parent() else {
                break;
            };
            current = parent.to_path_buf();
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
        self.generate_workspace_helper_scripts(&install_dir)?;
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
            let package_ps1 = package_share_dir.join("package.ps1");
            let package_dsv = package_share_dir.join("package.dsv");
            let local_setup_sh = package_share_dir.join("local_setup.sh");
            let local_setup_bash = package_share_dir.join("local_setup.bash");
            let local_setup_zsh = package_share_dir.join("local_setup.zsh");
            let hook_dir = package_share_dir.join("hook");
            fs::create_dir_all(&hook_dir)?;

            self.generate_package_hook_files(package, pkg_install_path, &hook_dir)?;

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
            fs::write(&package_zsh, self.render_shell_wrapper("zsh", &package_sh))?;
            fs::write(
                &package_ps1,
                self.render_package_ps1(&package.name, pkg_install_path),
            )?;
            fs::write(
                &package_dsv,
                self.render_package_dsv(
                    &package.name,
                    &self.package_hook_names(package, pkg_install_path),
                ),
            )?;

            Self::make_executable_if_unix(&local_setup_sh)?;
            Self::make_executable_if_unix(&local_setup_bash)?;
            Self::make_executable_if_unix(&local_setup_zsh)?;
            Self::make_executable_if_unix(&package_sh)?;
            Self::make_executable_if_unix(&package_bash)?;
            Self::make_executable_if_unix(&package_zsh)?;
            Self::make_executable_if_unix(&package_ps1)?;
        }

        Ok(())
    }

    fn generate_package_metadata_files(
        &self,
        packages: &[PackageMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let built_packages: HashSet<&str> = self
            .install_paths
            .keys()
            .map(|name| name.as_str())
            .collect();
        for package in packages {
            if !self.install_paths.contains_key(&package.name) {
                continue;
            }

            let runtime_deps = package
                .runtime_deps()
                .into_iter()
                .filter(|dep| built_packages.contains(dep.as_str()))
                .collect::<Vec<_>>();
            let metadata_path = self
                .package_metadata_root(&package.name)
                .join(&package.name);
            fs::create_dir_all(metadata_path.parent().unwrap())?;
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
        let local_setup_ps1 = install_dir.join("local_setup.ps1");
        let setup_sh = install_dir.join("setup.sh");
        let setup_bash = install_dir.join("setup.bash");
        let setup_zsh = install_dir.join("setup.zsh");
        let setup_ps1 = install_dir.join("setup.ps1");

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
        fs::write(
            &local_setup_ps1,
            self.render_workspace_local_setup_ps1(install_dir, packages),
        )?;
        fs::write(&setup_sh, self.render_workspace_setup_sh(install_dir))?;
        fs::write(&setup_bash, self.render_shell_wrapper("bash", &setup_sh))?;
        fs::write(&setup_zsh, self.render_shell_wrapper("zsh", &setup_sh))?;
        fs::write(&setup_ps1, self.render_workspace_setup_ps1(install_dir))?;

        Self::make_executable_if_unix(&local_setup_sh)?;
        Self::make_executable_if_unix(&local_setup_bash)?;
        Self::make_executable_if_unix(&local_setup_zsh)?;
        Self::make_executable_if_unix(&local_setup_ps1)?;
        Self::make_executable_if_unix(&setup_sh)?;
        Self::make_executable_if_unix(&setup_bash)?;
        Self::make_executable_if_unix(&setup_zsh)?;
        Self::make_executable_if_unix(&setup_ps1)?;

        Ok(())
    }

    fn generate_workspace_helper_scripts(
        &self,
        install_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(install_dir)?;
        fs::write(
            install_dir.join("_local_setup_util_sh.py"),
            self.render_workspace_helper_script(false),
        )?;
        fs::write(
            install_dir.join("_local_setup_util_ps1.py"),
            self.render_workspace_helper_script(true),
        )?;
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
        Self::write_colcon_install_layout(install_dir, self.config.merge_install)?;

        Ok(())
    }

    fn write_colcon_ignore(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(dir.join("COLCON_IGNORE"), "")?;
        Ok(())
    }

    fn write_colcon_install_layout(
        install_dir: &Path,
        merge_install: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let layout = if merge_install { "merged" } else { "isolated" };
        fs::write(install_dir.join(".colcon_install_layout"), layout)?;
        Ok(())
    }

    fn package_log_dir(config: &BuildConfig, package_name: &str) -> PathBuf {
        config.log_base.join("latest").join(package_name)
    }

    fn phase_log_path(config: &BuildConfig, package_name: &str, phase_name: &str) -> PathBuf {
        Self::package_log_dir(config, package_name).join(format!("{phase_name}.log"))
    }

    fn package_state_path(config: &BuildConfig, package_name: &str) -> PathBuf {
        Self::package_log_dir(config, package_name).join("status.txt")
    }

    fn write_build_summary(
        &self,
        packages: &[PackageMeta],
        records: &HashMap<String, BuildRecord>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut summary = String::from("# roc build summary\n");
        for package in packages {
            let Some(record) = records.get(&package.name) else {
                continue;
            };

            let status = match record.status {
                PackageState::Pending => "pending",
                PackageState::Building => "building",
                PackageState::Completed => "completed",
                PackageState::Failed => "failed",
            };
            summary.push_str(&format!(
                "{package}: status={status} duration_ms={duration}\n",
                package = package.name,
                duration = record.duration_ms,
            ));
            summary.push_str(&format!(
                "logs={log_dir}\n",
                log_dir = Self::package_log_dir(self.config, &package.name).display()
            ));
            if let Some(error) = &record.error {
                summary.push_str(&format!("error={error}\n"));
            }
            summary.push('\n');
        }

        fs::write(
            self.config
                .log_base
                .join("latest")
                .join("build_summary.log"),
            summary,
        )?;
        Ok(())
    }

    fn write_package_state(
        config: &BuildConfig,
        package_name: &str,
        record: &BuildRecord,
    ) -> Result<(), io::Error> {
        let status = match record.status {
            PackageState::Pending => "pending",
            PackageState::Building => "building",
            PackageState::Completed => "completed",
            PackageState::Failed => "failed",
        };
        let mut content = format!(
            "status={status}\nduration_ms={duration}\n",
            duration = record.duration_ms
        );
        if let Some(error) = &record.error {
            content.push_str(&format!("error={error}\n"));
        }

        let state_path = Self::package_state_path(config, package_name);
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(state_path, content)
    }

    fn render_local_setup_sh(&self, package_name: &str, prefix: &Path) -> String {
        let prefix_str = prefix.display();
        let artifacts = Self::scan_installed_artifacts(prefix);
        let hook_exports = self.render_package_hook_exports(package_name, prefix);
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

{path_exports}
{hook_exports}
"#,
            path_exports = Self::render_artifact_exports(&artifacts),
            hook_exports = hook_exports,
        )
    }

    fn render_package_sh(&self, package_name: &str, prefix: &Path) -> String {
        let local_setup_path = prefix
            .join("share")
            .join(package_name)
            .join("local_setup.sh");
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

    fn render_package_ps1(&self, package_name: &str, prefix: &Path) -> String {
        let hook_names = self.package_hook_names_for_ps1_path(package_name, prefix);
        let mut content = format!(
            r#"# Generated package setup script for package {package_name}

$env:COLCON_CURRENT_PREFIX="{prefix}"

"#,
            prefix = prefix.display(),
        );
        for hook_name in hook_names {
            let hook_path = prefix
                .join("share")
                .join(package_name)
                .join("hook")
                .join(hook_name);
            content.push_str(&format!(
                "if (Test-Path \"{hook}\") {{\n    . \"{hook}\"\n}}\n",
                hook = hook_path.display()
            ));
        }
        content.push_str("Remove-Item Env:\\COLCON_CURRENT_PREFIX -ErrorAction SilentlyContinue\n");
        content
    }

    fn render_package_dsv(&self, package_name: &str, hook_names: &[String]) -> String {
        let mut content = String::new();
        for hook_name in hook_names {
            if hook_name.ends_with(".ps1")
                || hook_name.ends_with(".dsv")
                || hook_name.ends_with(".sh")
            {
                content.push_str(&format!("source;share/{package_name}/hook/{hook_name}\n"));
            }
        }
        content
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

    fn generate_package_hook_files(
        &self,
        package: &PackageMeta,
        prefix: &Path,
        hook_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let artifacts = Self::scan_installed_artifacts(prefix);
        fs::write(
            hook_dir.join("ament_prefix_path.dsv"),
            "prepend-non-duplicate;AMENT_PREFIX_PATH;\n",
        )?;
        fs::write(
            hook_dir.join("ament_prefix_path.sh"),
            "# generated by roc\n\n_colcon_prepend_unique_value AMENT_PREFIX_PATH \"$COLCON_CURRENT_PREFIX\"\n",
        )?;
        fs::write(
            hook_dir.join("ament_prefix_path.ps1"),
            "# generated by roc\n\ncolcon_prepend_unique_value AMENT_PREFIX_PATH \"$env:COLCON_CURRENT_PREFIX\"\n",
        )?;

        if let Some(python_dir) = artifacts.python_dirs.first() {
            if let Ok(relative_python_dir) = python_dir.strip_prefix(prefix) {
                let rel = relative_python_dir.to_string_lossy().replace('\\', "/");
                fs::write(
                    hook_dir.join("pythonpath.dsv"),
                    format!("prepend-non-duplicate;PYTHONPATH;{rel}\n"),
                )?;
                fs::write(
                    hook_dir.join("pythonpath.sh"),
                    format!(
                        "# generated by roc\n\n_colcon_prepend_unique_value PYTHONPATH \"$COLCON_CURRENT_PREFIX/{rel}\"\n"
                    ),
                )?;
                fs::write(
                    hook_dir.join("pythonpath.ps1"),
                    format!(
                        "# generated by roc\n\ncolcon_prepend_unique_value PYTHONPATH \"$env:COLCON_CURRENT_PREFIX\\{}\"\n",
                        rel.replace('/', "\\")
                    ),
                )?;
            }
        }

        if matches!(package.build_type, BuildType::AmentCmake | BuildType::Cmake) {
            fs::write(
                hook_dir.join("cmake_prefix_path.dsv"),
                "prepend-non-duplicate;CMAKE_PREFIX_PATH;\n",
            )?;
            fs::write(
                hook_dir.join("cmake_prefix_path.sh"),
                "# generated by roc\n\n_colcon_prepend_unique_value CMAKE_PREFIX_PATH \"$COLCON_CURRENT_PREFIX\"\n",
            )?;
            fs::write(
                hook_dir.join("cmake_prefix_path.ps1"),
                "# generated by roc\n\ncolcon_prepend_unique_value CMAKE_PREFIX_PATH \"$env:COLCON_CURRENT_PREFIX\"\n",
            )?;
        }

        Ok(())
    }

    fn package_hook_names(&self, package: &PackageMeta, prefix: &Path) -> Vec<String> {
        let artifacts = Self::scan_installed_artifacts(prefix);
        let mut names = Vec::new();
        if !artifacts.python_dirs.is_empty() {
            names.extend([
                "pythonpath.ps1".to_string(),
                "pythonpath.dsv".to_string(),
                "pythonpath.sh".to_string(),
            ]);
        }
        names.extend([
            "ament_prefix_path.ps1".to_string(),
            "ament_prefix_path.dsv".to_string(),
            "ament_prefix_path.sh".to_string(),
        ]);
        if matches!(package.build_type, BuildType::AmentCmake | BuildType::Cmake) {
            names.extend([
                "cmake_prefix_path.ps1".to_string(),
                "cmake_prefix_path.dsv".to_string(),
                "cmake_prefix_path.sh".to_string(),
            ]);
        }
        names
    }

    fn package_hook_names_for_ps1_path(&self, package_name: &str, prefix: &Path) -> Vec<String> {
        let hook_dir = prefix.join("share").join(package_name).join("hook");
        let mut names = match fs::read_dir(hook_dir) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .filter(|name| name.ends_with(".ps1"))
                .collect::<Vec<_>>(),
            Err(_) => Vec::new(),
        };
        names.sort();
        names
    }

    fn render_workspace_helper_script(&self, powershell: bool) -> String {
        let invoke = if powershell {
            "_colcon_prefix_powershell_source_script"
        } else {
            "_colcon_prefix_sh_source_script"
        };
        format!(
            r#"#!/usr/bin/env python3
import os
import sys
from pathlib import Path


def read_packages(prefix):
    packages = {{}}
    root_index = prefix / "share" / "colcon-core" / "packages"
    if root_index.is_dir():
        for path in root_index.iterdir():
            if path.is_file() and not path.name.startswith('.'):
                packages[path.name] = parse_dependencies(path.read_text())

    for child in prefix.iterdir():
        package_index = child / "share" / "colcon-core" / "packages" / child.name
        if package_index.is_file():
            packages[child.name] = parse_dependencies(package_index.read_text())
    return packages


def parse_dependencies(text):
    deps = []
    for item in text.replace(os.pathsep, "\n").splitlines():
        item = item.strip()
        if item:
            deps.append(item)
    return deps


def topo(packages):
    ordered = []
    visited = set()

    def visit(name):
        if name in visited:
            return
        visited.add(name)
        for dep in packages.get(name, []):
            if dep in packages:
                visit(dep)
        ordered.append(name)

    for name in sorted(packages):
        visit(name)
    return ordered


def main():
    prefix = Path(__file__).resolve().parent
    ext = "ps1" if len(sys.argv) > 1 and sys.argv[1] == "ps1" else "sh"
    packages = read_packages(prefix)
    for package in topo(packages):
        package_prefix = prefix / package
        if not package_prefix.is_dir():
            package_prefix = prefix
        script = package_prefix / "share" / package / f"package.{{ext}}"
        if script.is_file():
            if ext == "ps1":
                print(f'$env:COLCON_CURRENT_PREFIX="{{{{package_prefix}}}}"')
                print(f'{invoke} "{{{{script}}}}"')
            else:
                print(f'COLCON_CURRENT_PREFIX="{{{{package_prefix}}}}" {invoke} "{{{{script}}}}"')


if __name__ == "__main__":
    main()
"#
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

    fn render_workspace_local_setup_ps1(
        &self,
        install_dir: &Path,
        packages: &[PackageMeta],
    ) -> String {
        let helper_path = install_dir.join("_local_setup_util_ps1.py");
        let mut content = format!(
            r#"# Generated workspace local setup script

$env:COLCON_CURRENT_PREFIX="{install_prefix}"

function colcon_prepend_unique_value {{
    param(
        [string]$EnvVarName,
        [string]$Value
    )

    $pathSeparator = [System.IO.Path]::PathSeparator
    $currentValue = [Environment]::GetEnvironmentVariable($EnvVarName, "Process")
    $entries = @()
    if ($currentValue) {{
        $entries = $currentValue.Split($pathSeparator) | Where-Object {{ $_ }}
    }}
    if ($entries -notcontains $Value) {{
        $entries = @($Value) + $entries
    }}
    [Environment]::SetEnvironmentVariable($EnvVarName, ($entries -join $pathSeparator), "Process")
}}

function _colcon_prefix_powershell_source_script {{
    param([string]$ScriptPath)
    if (Test-Path $ScriptPath) {{
        . $ScriptPath
    }}
}}

"#,
            install_prefix = install_dir.display()
        );

        if helper_path.exists() {
            content.push_str(&format!(
                "$_colcon_ordered_scripts = & python3 \"{helper}\" ps1\nforeach ($_colcon_script in $_colcon_ordered_scripts) {{\n    Invoke-Expression $_colcon_script\n}}\n",
                helper = helper_path.display()
            ));
        } else {
            for package in packages {
                let Some(package_prefix) = self.install_paths.get(&package.name) else {
                    continue;
                };
                let package_script = package_prefix
                    .join("share")
                    .join(&package.name)
                    .join("package.ps1");
                content.push_str(&format!(
                    "if (Test-Path \"{package_script}\") {{\n    $env:COLCON_CURRENT_PREFIX=\"{package_prefix}\"\n    . \"{package_script}\"\n}}\n",
                    package_script = package_script.display(),
                    package_prefix = package_prefix.display()
                ));
            }
        }

        content.push_str(
            "Remove-Item Env:\\COLCON_CURRENT_PREFIX -ErrorAction SilentlyContinue\n",
        );
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

_colcon_normalize_path_list() {{
    var_name="$1"
    eval current_value="\${{${{var_name}}:-}}"
    if [ -z "$current_value" ]; then
        return
    fi

    old_ifs="$IFS"
    IFS=':'
    normalized=""
    for entry in $current_value; do
        if [ -z "$entry" ]; then
            continue
        fi
        if [ -z "$normalized" ]; then
            normalized="$entry"
        else
            normalized="$normalized:$entry"
        fi
    done
    IFS="$old_ifs"
    unset old_ifs

    if [ -n "$normalized" ]; then
        eval export "$var_name=$normalized"
    else
        unset "$var_name"
    fi
}}

_colcon_workspace_prefix="{install_prefix}"
_colcon_previous_prefixes="${{COLCON_PREFIX_PATH:-}}"
_colcon_normalize_path_list COLCON_PREFIX_PATH
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
_colcon_normalize_path_list COLCON_PREFIX_PATH
unset _colcon_workspace_prefix
unset _colcon_previous_prefixes
unset _colcon_prefix
"#,
            install_prefix = install_dir.display(),
            local_setup = install_dir.join("local_setup.sh").display(),
        )
    }

    fn render_workspace_setup_ps1(&self, install_dir: &Path) -> String {
        format!(
            r#"# Generated workspace setup script

function colcon_prepend_unique_value {{
    param(
        [string]$EnvVarName,
        [string]$Value
    )

    $pathSeparator = [System.IO.Path]::PathSeparator
    $currentValue = [Environment]::GetEnvironmentVariable($EnvVarName, "Process")
    $entries = @()
    if ($currentValue) {{
        $entries = $currentValue.Split($pathSeparator) | Where-Object {{ $_ }}
    }}
    if ($entries -notcontains $Value) {{
        $entries = @($Value) + $entries
    }}
    [Environment]::SetEnvironmentVariable($EnvVarName, ($entries -join $pathSeparator), "Process")
}}

function colcon_normalize_path_list {{
    param([string]$EnvVarName)

    $pathSeparator = [System.IO.Path]::PathSeparator
    $currentValue = [Environment]::GetEnvironmentVariable($EnvVarName, "Process")
    if (-not $currentValue) {{
        return
    }}

    $normalized = @()
    foreach ($entry in $currentValue.Split($pathSeparator)) {{
        if (-not [string]::IsNullOrWhiteSpace($entry)) {{
            $normalized += $entry
        }}
    }}

    if ($normalized.Count -gt 0) {{
        [Environment]::SetEnvironmentVariable($EnvVarName, ($normalized -join $pathSeparator), "Process")
    }} else {{
        Remove-Item "Env:$EnvVarName" -ErrorAction SilentlyContinue
    }}
}}

function _colcon_prefix_powershell_source_script {{
    param([string]$ScriptPath)
    if (Test-Path $ScriptPath) {{
        . $ScriptPath
    }}
}}

$_colcon_workspace_prefix = "{install_prefix}"
colcon_normalize_path_list COLCON_PREFIX_PATH
$_colcon_previous_prefixes = [Environment]::GetEnvironmentVariable("COLCON_PREFIX_PATH", "Process")
if ($_colcon_previous_prefixes) {{
    foreach ($_colcon_prefix in $_colcon_previous_prefixes.Split([System.IO.Path]::PathSeparator)) {{
        if (-not [string]::IsNullOrWhiteSpace($_colcon_prefix) -and $_colcon_prefix -ne $_colcon_workspace_prefix) {{
            _colcon_prefix_powershell_source_script (Join-Path $_colcon_prefix "local_setup.ps1")
        }}
    }}
}}

_colcon_prefix_powershell_source_script "{local_setup}"
colcon_prepend_unique_value COLCON_PREFIX_PATH $_colcon_workspace_prefix
colcon_normalize_path_list COLCON_PREFIX_PATH
Remove-Variable _colcon_workspace_prefix -ErrorAction SilentlyContinue
Remove-Variable _colcon_previous_prefixes -ErrorAction SilentlyContinue
Remove-Variable _colcon_prefix -ErrorAction SilentlyContinue
"#,
            install_prefix = install_dir.display(),
            local_setup = install_dir.join("local_setup.ps1").display(),
        )
    }

    fn package_metadata_root(&self, package_name: &str) -> PathBuf {
        if self.config.merge_install {
            self.config
                .install_base
                .join("share")
                .join("colcon-core")
                .join("packages")
        } else if let Some(package_prefix) = self.install_paths.get(package_name) {
            package_prefix.join("share").join("colcon-core").join("packages")
        } else {
            self.config
                .install_base
                .join(package_name)
                .join("share")
                .join("colcon-core")
                .join("packages")
        }
    }

    fn scan_installed_artifacts(prefix: &Path) -> InstalledArtifacts {
        let mut artifacts = InstalledArtifacts::default();

        let bin_dir = prefix.join("bin");
        if bin_dir.is_dir() {
            artifacts.bin_dirs.push(bin_dir);
        }

        let lib_dir = prefix.join("lib");
        if lib_dir.is_dir() {
            artifacts.lib_dirs.push(lib_dir.clone());
        }

        let local_lib_dir = prefix.join("local").join("lib");
        if local_lib_dir.is_dir() {
            artifacts.lib_dirs.push(local_lib_dir.clone());
        }

        for base in [&lib_dir, &local_lib_dir] {
            if !base.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) else {
                        continue;
                    };
                    if !dir_name.starts_with("python") {
                        continue;
                    }

                    let site_packages = path.join("site-packages");
                    if site_packages.is_dir() {
                        artifacts.python_dirs.push(site_packages);
                    }

                    let dist_packages = path.join("dist-packages");
                    if dist_packages.is_dir() {
                        artifacts.python_dirs.push(dist_packages);
                    }
                }
            }
        }

        artifacts
    }

    fn render_artifact_exports(artifacts: &InstalledArtifacts) -> String {
        let mut content = String::new();

        for bin_dir in &artifacts.bin_dirs {
            content.push_str(&format!(
                "if [ -d \"{path}\" ]; then\n    _colcon_prepend_unique_value PATH \"{path}\"\nfi\n\n",
                path = bin_dir.display()
            ));
        }

        for lib_dir in &artifacts.lib_dirs {
            content.push_str(&format!(
                "if [ -d \"{path}\" ]; then\n    _colcon_prepend_unique_value LD_LIBRARY_PATH \"{path}\"\nfi\n\n",
                path = lib_dir.display()
            ));
        }

        for python_dir in &artifacts.python_dirs {
            content.push_str(&format!(
                "if [ -d \"{path}\" ]; then\n    _colcon_prepend_unique_value PYTHONPATH \"{path}\"\nfi\n\n",
                path = python_dir.display()
            ));
        }

        content
    }

    fn render_package_hook_exports(&self, package_name: &str, prefix: &Path) -> String {
        let hook_dir = prefix.join("share").join(package_name).join("hook");
        if !hook_dir.is_dir() {
            return String::new();
        }

        let mut entries = match fs::read_dir(&hook_dir) {
            Ok(entries) => entries.filter_map(|entry| entry.ok()).collect::<Vec<_>>(),
            Err(_) => return String::new(),
        };
        entries.sort_by_key(|entry| entry.file_name());

        let mut content = String::new();
        for entry in entries {
            let path = entry.path();
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("dsv") => {
                    content.push_str(&self.render_dsv_file(&path, prefix));
                }
                Some("sh") => {
                    content.push_str(&format!(
                        "if [ -f \"{path}\" ]; then\n    . \"{path}\"\nfi\n\n",
                        path = path.display()
                    ));
                }
                _ => {}
            }
        }

        content
    }

    fn render_dsv_file(&self, dsv_path: &Path, prefix: &Path) -> String {
        let Ok(contents) = fs::read_to_string(dsv_path) else {
            return String::new();
        };

        let mut rendered = String::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some(operation) = Self::parse_dsv_operation(line) else {
                continue;
            };
            rendered.push_str(&Self::render_dsv_operation(&operation, prefix));
        }

        rendered
    }

    fn parse_dsv_operation(line: &str) -> Option<DsvOperation> {
        let mut parts = line.splitn(3, ';');
        let op = parts.next()?;
        let first = parts.next()?.to_string();
        let second = parts.next().unwrap_or_default().to_string();

        match op {
            "prepend-non-duplicate" => Some(DsvOperation::PrependNonDuplicate {
                name: first,
                value: second,
            }),
            "prepend-non-duplicate-if-exists" => Some(DsvOperation::PrependNonDuplicateIfExists {
                name: first,
                value: second,
            }),
            "set" => Some(DsvOperation::Set {
                name: first,
                value: second,
            }),
            "set-if-unset" => Some(DsvOperation::SetIfUnset {
                name: first,
                value: second,
            }),
            "source" => Some(DsvOperation::Source { path: first }),
            _ => None,
        }
    }

    fn render_dsv_operation(operation: &DsvOperation, prefix: &Path) -> String {
        match operation {
            DsvOperation::PrependNonDuplicate { name, value } => {
                let resolved = Self::resolve_dsv_value(prefix, value);
                format!(
                    "_colcon_prepend_unique_value {name} \"{resolved}\"\n",
                    name = name,
                    resolved = resolved.display()
                )
            }
            DsvOperation::PrependNonDuplicateIfExists { name, value } => {
                let resolved = Self::resolve_dsv_value(prefix, value);
                format!(
                    "if [ -e \"{resolved}\" ]; then\n    _colcon_prepend_unique_value {name} \"{resolved}\"\nfi\n",
                    name = name,
                    resolved = resolved.display()
                )
            }
            DsvOperation::Set { name, value } => {
                let resolved = Self::resolve_set_value(prefix, value);
                format!(
                    "export {name}=\"{resolved}\"\n",
                    name = name,
                    resolved = resolved
                )
            }
            DsvOperation::SetIfUnset { name, value } => {
                let resolved = Self::resolve_set_value(prefix, value);
                format!(
                    "if [ -z \"${{{name}:-}}\" ]; then\n    export {name}=\"{resolved}\"\nfi\n",
                    name = name,
                    resolved = resolved
                )
            }
            DsvOperation::Source { path } => {
                let resolved = Self::resolve_dsv_value(prefix, path);
                format!(
                    "if [ -f \"{resolved}\" ]; then\n    . \"{resolved}\"\nfi\n",
                    resolved = resolved.display()
                )
            }
        }
    }

    fn resolve_dsv_value(prefix: &Path, value: &str) -> PathBuf {
        if value.is_empty() {
            return prefix.to_path_buf();
        }

        let path = PathBuf::from(value);
        if path.is_absolute() {
            path
        } else {
            prefix.join(path)
        }
    }

    fn resolve_set_value(prefix: &Path, value: &str) -> String {
        let resolved = Self::resolve_dsv_value(prefix, value);
        if !Path::new(value).is_absolute() && resolved.exists() {
            resolved.display().to_string()
        } else {
            value.to_string()
        }
    }

    fn find_python_package_install_dir(
        install_prefix: &Path,
        package_name: &str,
    ) -> Option<PathBuf> {
        for base in [
            install_prefix.join("lib"),
            install_prefix.join("local").join("lib"),
        ] {
            if !base.is_dir() {
                continue;
            }

            let entries = match fs::read_dir(&base) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let python_dir = entry.path();
                if !python_dir.is_dir() {
                    continue;
                }
                let Some(dir_name) = python_dir.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !dir_name.starts_with("python") {
                    continue;
                }

                for site_dir_name in ["site-packages", "dist-packages"] {
                    let candidate = python_dir.join(site_dir_name).join(package_name);
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }

        None
    }

    fn replace_with_symlink(destination: &Path, source: &Path) -> Result<(), io::Error> {
        if !destination.exists() {
            return Ok(());
        }

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        let metadata = fs::symlink_metadata(destination)?;
        if metadata.file_type().is_symlink() {
            fs::remove_file(destination)?;
        } else if metadata.is_dir() {
            fs::remove_dir_all(destination)?;
        } else {
            fs::remove_file(destination)?;
        }

        Self::symlink_path(source, destination)
    }

    fn symlink_path(source: &Path, destination: &Path) -> Result<(), io::Error> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, destination)
        }
        #[cfg(windows)]
        {
            if source.is_dir() {
                std::os::windows::fs::symlink_dir(source, destination)
            } else {
                std::os::windows::fs::symlink_file(source, destination)
            }
        }
    }

    fn cmake_configure_args(
        package_path: &Path,
        build_dir: &Path,
        install_prefix: &Path,
        symlink_install: bool,
        cmake_args: &[String],
    ) -> Vec<String> {
        let mut args = vec![
            "-S".to_string(),
            package_path.display().to_string(),
            "-B".to_string(),
            build_dir.display().to_string(),
            format!("-DCMAKE_INSTALL_PREFIX={}", install_prefix.display()),
        ];
        if symlink_install {
            args.push("-DCMAKE_INSTALL_MODE=ABS_SYMLINK_FILES".to_string());
        }
        args.extend(cmake_args.iter().cloned());
        args
    }

    fn cmake_build_args(
        build_dir: &Path,
        parallel_workers: u32,
        cmake_target: Option<&str>,
    ) -> Vec<String> {
        let mut args = vec![
            "--build".to_string(),
            build_dir.display().to_string(),
            "--parallel".to_string(),
            parallel_workers.to_string(),
        ];
        if let Some(target) = cmake_target {
            args.push("--target".to_string());
            args.push(target.to_string());
        }
        args
    }

    fn cmake_install_args(build_dir: &Path, install_prefix: &Path) -> Vec<String> {
        vec![
            "--install".to_string(),
            build_dir.display().to_string(),
            "--prefix".to_string(),
            install_prefix.display().to_string(),
        ]
    }

    fn python_build_args(build_dir: &Path) -> Vec<String> {
        vec![
            "setup.py".to_string(),
            "build".to_string(),
            "--build-base".to_string(),
            build_dir.display().to_string(),
        ]
    }

    fn python_install_args(build_dir: &Path, install_prefix: &Path) -> Vec<String> {
        vec![
            "setup.py".to_string(),
            "install".to_string(),
            "--prefix".to_string(),
            "".to_string(),
            "--root".to_string(),
            install_prefix.display().to_string(),
            "--single-version-externally-managed".to_string(),
            "--record".to_string(),
            build_dir.join("install-record.txt").display().to_string(),
        ]
    }

    fn run_command_checked(
        mut command: Command,
        label: &str,
        log_path: &Path,
    ) -> Result<(), String> {
        let program = command.get_program().to_string_lossy().to_string();
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        let output = command
            .output()
            .map_err(|e| format!("{label} failed to start: {e}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let command_line = std::iter::once(program)
            .chain(args.into_iter())
            .collect::<Vec<_>>()
            .join(" ");
        Self::write_command_log(
            log_path,
            label,
            &command_line,
            output.status.code(),
            &stdout,
            &stderr,
        )
        .map_err(|e| format!("Failed to write {label} log: {e}"))?;
        if output.status.success() {
            return Ok(());
        }

        let mut message = format!("{label} failed.\nCommand: {command_line}");
        if !stdout.is_empty() {
            message.push_str(&format!("\nstdout:\n{stdout}"));
        }
        if !stderr.is_empty() {
            message.push_str(&format!("\nstderr:\n{stderr}"));
        }

        Err(message)
    }

    fn write_command_log(
        log_path: &Path,
        label: &str,
        command_line: &str,
        exit_code: Option<i32>,
        stdout: &str,
        stderr: &str,
    ) -> Result<(), io::Error> {
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut content = format!(
            "[{label}]\ncommand={command_line}\nexit_code={exit_code}\n\n",
            exit_code = exit_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".to_string())
        );
        if !stdout.is_empty() {
            content.push_str("[stdout]\n");
            content.push_str(stdout);
            content.push_str("\n\n");
        }
        if !stderr.is_empty() {
            content.push_str("[stderr]\n");
            content.push_str(stderr);
            content.push('\n');
        }

        fs::write(log_path, content)
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
    use super::{BuildExecutor, BuildRecord, DsvOperation, InstalledArtifacts, PackageState};
    use crate::commands::work::build::{BuildConfig, BuildType, PackageMeta};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
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
        assert_eq!(
            fs::read_to_string(config.install_base.join(".colcon_install_layout")).unwrap(),
            "isolated"
        );
        assert!(
            config
                .log_base
                .join("latest")
                .join("COLCON_IGNORE")
                .exists()
        );
    }

    #[test]
    fn generate_package_metadata_files_writes_isolated_runtime_dependencies_to_package_prefixes() {
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

        let metadata_root = config
            .install_base
            .join("consumer_pkg")
            .join("share/colcon-core/packages");
        let base_metadata_root = config
            .install_base
            .join("base_pkg")
            .join("share/colcon-core/packages");
        assert_eq!(
            std::fs::read_to_string(base_metadata_root.join("base_pkg")).unwrap(),
            ""
        );
        assert_eq!(
            std::fs::read_to_string(metadata_root.join("consumer_pkg")).unwrap(),
            "base_pkg"
        );
        assert!(!config
            .install_base
            .join("share/colcon-core/packages/consumer_pkg")
            .exists());
    }

    #[test]
    fn generate_package_metadata_files_writes_merged_runtime_dependencies_to_workspace_root() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");
        config.merge_install = true;
        config.isolated = false;

        let mut executor = BuildExecutor::new(&config);
        executor
            .install_paths
            .insert("base_pkg".to_string(), config.install_base.clone());
        executor
            .install_paths
            .insert("consumer_pkg".to_string(), config.install_base.clone());

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
                build_export_deps: Vec::new(),
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
        assert!(share_dir.join("package.ps1").exists());
        assert!(share_dir.join("package.dsv").exists());
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
    fn generate_package_setup_scripts_writes_standard_hook_files_for_python_packages() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");

        let mut executor = BuildExecutor::new(&config);
        let package_prefix = config.install_base.join("demo_python_pkg");
        fs::create_dir_all(package_prefix.join("lib/python3.12/site-packages")).unwrap();
        executor
            .install_paths
            .insert("demo_python_pkg".to_string(), package_prefix.clone());

        let packages = vec![PackageMeta {
            name: "demo_python_pkg".to_string(),
            path: PathBuf::from("/tmp/demo_python_pkg"),
            build_type: BuildType::AmentPython,
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

        let share_dir = package_prefix.join("share").join("demo_python_pkg");
        let hook_dir = share_dir.join("hook");
        assert!(hook_dir.join("ament_prefix_path.dsv").exists());
        assert!(hook_dir.join("ament_prefix_path.sh").exists());
        assert!(hook_dir.join("ament_prefix_path.ps1").exists());
        assert!(hook_dir.join("pythonpath.dsv").exists());
        assert!(hook_dir.join("pythonpath.sh").exists());
        assert!(hook_dir.join("pythonpath.ps1").exists());

        let package_dsv = fs::read_to_string(share_dir.join("package.dsv")).unwrap();
        assert!(package_dsv.contains("source;share/demo_python_pkg/hook/pythonpath.dsv"));
        assert!(package_dsv.contains("source;share/demo_python_pkg/hook/ament_prefix_path.dsv"));

        let package_ps1 = fs::read_to_string(share_dir.join("package.ps1")).unwrap();
        assert!(package_ps1.contains("pythonpath.ps1"));
        assert!(package_ps1.contains("ament_prefix_path.ps1"));
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

        executor
            .generate_workspace_helper_scripts(&config.install_base)
            .unwrap();
        executor
            .generate_workspace_setup_scripts(&config.install_base, &packages)
            .unwrap();

        assert!(config.install_base.join("local_setup.sh").exists());
        assert!(config.install_base.join("local_setup.bash").exists());
        assert!(config.install_base.join("local_setup.zsh").exists());
        assert!(config.install_base.join("local_setup.ps1").exists());
        assert!(config.install_base.join("setup.sh").exists());
        assert!(config.install_base.join("setup.bash").exists());
        assert!(config.install_base.join("setup.zsh").exists());
        assert!(config.install_base.join("setup.ps1").exists());

        let local_setup =
            std::fs::read_to_string(config.install_base.join("local_setup.sh")).unwrap();
        assert!(local_setup.contains("package.sh"));
        assert!(local_setup.contains("COLCON_CURRENT_PREFIX"));

        let local_setup_ps1 =
            std::fs::read_to_string(config.install_base.join("local_setup.ps1")).unwrap();
        assert!(local_setup_ps1.contains("colcon_prepend_unique_value"));
        assert!(local_setup_ps1.contains("_local_setup_util_ps1.py"));

        let setup_sh = std::fs::read_to_string(config.install_base.join("setup.sh")).unwrap();
        assert!(setup_sh.contains("COLCON_PREFIX_PATH"));
        assert!(setup_sh.contains("local_setup.sh"));
        assert!(setup_sh.contains("_colcon_normalize_path_list"));

        let setup_ps1 = std::fs::read_to_string(config.install_base.join("setup.ps1")).unwrap();
        assert!(setup_ps1.contains("local_setup.ps1"));
        assert!(setup_ps1.contains("colcon_normalize_path_list"));
        assert!(setup_ps1.contains("COLCON_PREFIX_PATH"));
    }

    #[test]
    fn workspace_setup_normalizes_colcon_prefix_path_without_trailing_separator() {
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

        fs::create_dir_all(package_prefix.join("share/demo_pkg")).unwrap();
        executor
            .generate_workspace_setup_scripts(&config.install_base, &packages)
            .unwrap();

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "COLCON_PREFIX_PATH='{prefix}:' . '{setup}' >/dev/null 2>&1; printf '%s' \"$COLCON_PREFIX_PATH\"",
                prefix = config.install_base.display(),
                setup = config.install_base.join("setup.sh").display(),
            ))
            .output()
            .unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            config.install_base.display().to_string()
        );
    }

    #[test]
    fn generate_workspace_helper_scripts_writes_local_setup_util_helpers() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");

        let executor = BuildExecutor::new(&config);
        fs::create_dir_all(&config.install_base).unwrap();
        executor
            .generate_workspace_helper_scripts(&config.install_base)
            .unwrap();

        let sh_helper =
            fs::read_to_string(config.install_base.join("_local_setup_util_sh.py")).unwrap();
        let ps1_helper =
            fs::read_to_string(config.install_base.join("_local_setup_util_ps1.py")).unwrap();
        assert!(sh_helper.contains("def read_packages(prefix):"));
        assert!(sh_helper.contains("_colcon_prefix_sh_source_script"));
        assert!(ps1_helper.contains("_colcon_prefix_powershell_source_script"));
    }

    #[test]
    fn scan_installed_artifacts_detects_dynamic_python_and_runtime_paths() {
        let temp = tempdir().unwrap();
        let prefix = temp.path().join("install-prefix");
        fs::create_dir_all(prefix.join("bin")).unwrap();
        fs::create_dir_all(prefix.join("lib")).unwrap();
        fs::create_dir_all(prefix.join("lib/python3.12/site-packages")).unwrap();
        fs::create_dir_all(prefix.join("local/lib/python3.11/dist-packages")).unwrap();

        let artifacts = BuildExecutor::scan_installed_artifacts(&prefix);

        assert_eq!(
            artifacts,
            InstalledArtifacts {
                bin_dirs: vec![prefix.join("bin")],
                lib_dirs: vec![prefix.join("lib"), prefix.join("local/lib")],
                python_dirs: vec![
                    prefix.join("lib/python3.12/site-packages"),
                    prefix.join("local/lib/python3.11/dist-packages"),
                ],
            }
        );
    }

    #[test]
    fn render_local_setup_sh_uses_detected_artifact_paths() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");

        let executor = BuildExecutor::new(&config);
        let prefix = config.install_base.join("demo_pkg");
        fs::create_dir_all(prefix.join("bin")).unwrap();
        fs::create_dir_all(prefix.join("lib/python3.12/site-packages")).unwrap();

        let script = executor.render_local_setup_sh("demo_pkg", &prefix);

        assert!(script.contains(&prefix.join("bin").display().to_string()));
        assert!(
            script.contains(
                &prefix
                    .join("lib/python3.12/site-packages")
                    .display()
                    .to_string()
            )
        );
        assert!(!script.contains("python3.10"));
    }

    #[test]
    fn parse_dsv_operation_supports_standard_colcon_operations() {
        assert_eq!(
            BuildExecutor::parse_dsv_operation("prepend-non-duplicate;PATH;bin"),
            Some(DsvOperation::PrependNonDuplicate {
                name: "PATH".to_string(),
                value: "bin".to_string(),
            })
        );
        assert_eq!(
            BuildExecutor::parse_dsv_operation("set-if-unset;RMW_IMPLEMENTATION;rmw_fastrtps_cpp"),
            Some(DsvOperation::SetIfUnset {
                name: "RMW_IMPLEMENTATION".to_string(),
                value: "rmw_fastrtps_cpp".to_string(),
            })
        );
        assert_eq!(
            BuildExecutor::parse_dsv_operation("source;share/demo_pkg/hook/custom.sh"),
            Some(DsvOperation::Source {
                path: "share/demo_pkg/hook/custom.sh".to_string(),
            })
        );
    }

    #[test]
    fn render_local_setup_sh_includes_hook_scripts_and_dsv_effects() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.install_base = workspace_root.join("install");

        let executor = BuildExecutor::new(&config);
        let prefix = config.install_base.join("demo_pkg");
        let hook_dir = prefix.join("share/demo_pkg/hook");
        fs::create_dir_all(&hook_dir).unwrap();
        fs::write(
            hook_dir.join("10-python.dsv"),
            "prepend-non-duplicate;PYTHONPATH;lib/python3.12/site-packages\nset-if-unset;RMW_IMPLEMENTATION;rmw_fastrtps_cpp\n",
        )
        .unwrap();
        fs::write(hook_dir.join("20-extra.sh"), "export DEMO_HOOK=1\n").unwrap();
        fs::create_dir_all(prefix.join("lib/python3.12/site-packages")).unwrap();

        let script = executor.render_local_setup_sh("demo_pkg", &prefix);

        assert!(script.contains("PYTHONPATH"));
        assert!(script.contains("rmw_fastrtps_cpp"));
        assert!(script.contains("20-extra.sh"));
    }

    #[test]
    fn apply_python_symlink_install_replaces_installed_python_artifacts_with_symlinks() {
        let temp = tempdir().unwrap();
        let package_root = temp.path().join("src/demo_python_pkg");
        let source_module_dir = package_root.join("demo_python_pkg");
        let source_resource_marker = package_root.join("resource/demo_python_pkg");
        let source_package_xml = package_root.join("package.xml");
        fs::create_dir_all(&source_module_dir).unwrap();
        fs::create_dir_all(source_resource_marker.parent().unwrap()).unwrap();
        fs::write(source_module_dir.join("__init__.py"), "# source module\n").unwrap();
        fs::write(&source_resource_marker, "").unwrap();
        fs::write(&source_package_xml, "<package />\n").unwrap();

        let install_prefix = temp.path().join("install/demo_python_pkg");
        let installed_module_dir =
            install_prefix.join("lib/python3.12/site-packages/demo_python_pkg");
        let installed_resource_marker =
            install_prefix.join("share/ament_index/resource_index/packages/demo_python_pkg");
        let installed_package_xml = install_prefix.join("share/demo_python_pkg/package.xml");
        fs::create_dir_all(installed_module_dir.parent().unwrap()).unwrap();
        fs::create_dir_all(installed_resource_marker.parent().unwrap()).unwrap();
        fs::create_dir_all(installed_package_xml.parent().unwrap()).unwrap();
        fs::create_dir_all(&installed_module_dir).unwrap();
        fs::write(
            installed_module_dir.join("__init__.py"),
            "# installed module\n",
        )
        .unwrap();
        fs::write(&installed_resource_marker, "").unwrap();
        fs::write(&installed_package_xml, "<package />\n").unwrap();

        let package = PackageMeta {
            name: "demo_python_pkg".to_string(),
            path: package_root.clone(),
            build_type: BuildType::AmentPython,
            version: "0.1.0".to_string(),
            description: "demo".to_string(),
            maintainers: vec!["Fixture".to_string()],
            depend_deps: Vec::new(),
            build_deps: Vec::new(),
            buildtool_deps: Vec::new(),
            build_export_deps: Vec::new(),
            exec_deps: Vec::new(),
            test_deps: Vec::new(),
        };

        BuildExecutor::apply_python_symlink_install(&package, temp.path(), &install_prefix)
            .unwrap();

        let module_meta = fs::symlink_metadata(&installed_module_dir).unwrap();
        let marker_meta = fs::symlink_metadata(&installed_resource_marker).unwrap();
        let xml_meta = fs::symlink_metadata(&installed_package_xml).unwrap();
        assert!(module_meta.file_type().is_symlink());
        assert!(marker_meta.file_type().is_symlink());
        assert!(xml_meta.file_type().is_symlink());
    }

    #[test]
    fn normalize_python_install_layout_moves_dist_packages_into_site_packages() {
        let temp = tempdir().unwrap();
        let install_prefix = temp.path().join("install/demo_python_pkg");
        let local_package_dir =
            install_prefix.join("local/lib/python3.12/dist-packages/demo_python_pkg");
        let local_egg_info_dir = install_prefix
            .join("local/lib/python3.12/dist-packages/demo_python_pkg-0.1.0-py3.12.egg-info");

        fs::create_dir_all(&local_package_dir).unwrap();
        fs::create_dir_all(&local_egg_info_dir).unwrap();
        fs::write(local_package_dir.join("__init__.py"), "# module\n").unwrap();
        fs::write(local_egg_info_dir.join("PKG-INFO"), "metadata\n").unwrap();

        BuildExecutor::normalize_python_install_layout(&install_prefix).unwrap();

        let site_packages = install_prefix.join("lib/python3.12/site-packages");
        assert!(site_packages.join("demo_python_pkg/__init__.py").exists());
        assert!(
            site_packages
                .join("demo_python_pkg-0.1.0-py3.12.egg-info/PKG-INFO")
                .exists()
        );
        assert!(
            !install_prefix
                .join("local/lib/python3.12/dist-packages")
                .exists()
        );
    }

    #[test]
    fn normalize_python_install_layout_moves_local_share_into_standard_share() {
        let temp = tempdir().unwrap();
        let install_prefix = temp.path().join("install/demo_python_pkg");
        let local_marker =
            install_prefix.join("local/share/ament_index/resource_index/packages/demo_python_pkg");
        let local_package_xml = install_prefix.join("local/share/demo_python_pkg/package.xml");

        fs::create_dir_all(local_marker.parent().unwrap()).unwrap();
        fs::create_dir_all(local_package_xml.parent().unwrap()).unwrap();
        fs::write(&local_marker, "").unwrap();
        fs::write(&local_package_xml, "<package />\n").unwrap();

        BuildExecutor::normalize_python_install_layout(&install_prefix).unwrap();

        assert!(
            install_prefix
                .join("share/ament_index/resource_index/packages/demo_python_pkg")
                .exists()
        );
        assert!(
            install_prefix
                .join("share/demo_python_pkg/package.xml")
                .exists()
        );
        assert!(!install_prefix.join("local/share").exists());
    }

    #[test]
    fn cmake_command_args_are_split_into_configure_build_and_install_phases() {
        let package_path = PathBuf::from("/ws/src/demo_pkg");
        let build_dir = PathBuf::from("/ws/build/demo_pkg");
        let install_prefix = PathBuf::from("/ws/install/demo_pkg");

        let configure = BuildExecutor::cmake_configure_args(
            &package_path,
            &build_dir,
            &install_prefix,
            true,
            &["-DCMAKE_BUILD_TYPE=RelWithDebInfo".to_string()],
        );
        let build = BuildExecutor::cmake_build_args(&build_dir, 8, Some("demo_target"));
        let install = BuildExecutor::cmake_install_args(&build_dir, &install_prefix);

        assert!(configure.contains(&"-S".to_string()));
        assert!(configure.contains(&"-B".to_string()));
        assert!(configure.contains(&"-DCMAKE_INSTALL_MODE=ABS_SYMLINK_FILES".to_string()));
        assert!(configure.contains(&"-DCMAKE_BUILD_TYPE=RelWithDebInfo".to_string()));
        assert_eq!(
            build,
            vec![
                "--build",
                "/ws/build/demo_pkg",
                "--parallel",
                "8",
                "--target",
                "demo_target"
            ]
        );
        assert_eq!(
            install,
            vec![
                "--install",
                "/ws/build/demo_pkg",
                "--prefix",
                "/ws/install/demo_pkg"
            ]
        );
    }

    #[test]
    fn python_install_args_request_single_version_install_record() {
        let build_dir = PathBuf::from("/ws/build/demo_python_pkg");
        let install_prefix = PathBuf::from("/ws/install/demo_python_pkg");

        let args = BuildExecutor::python_install_args(&build_dir, &install_prefix);

        assert_eq!(args[0], "setup.py");
        assert!(args.contains(&"--single-version-externally-managed".to_string()));
        assert!(args.contains(&"--record".to_string()));
        assert!(args.contains(&"/ws/build/demo_python_pkg/install-record.txt".to_string()));
    }

    #[test]
    fn run_command_checked_writes_phase_log() {
        let temp = tempdir().unwrap();
        let log_path = temp.path().join("latest/demo_pkg/build.log");

        let mut command = Command::new("sh");
        command.args(["-c", "printf 'hello stdout'; printf 'oops stderr' >&2"]);

        BuildExecutor::run_command_checked(command, "Fixture command", &log_path).unwrap();

        let log = fs::read_to_string(log_path).unwrap();
        assert!(log.contains("[Fixture command]"));
        assert!(log.contains("exit_code=0"));
        assert!(log.contains("[stdout]"));
        assert!(log.contains("hello stdout"));
        assert!(log.contains("[stderr]"));
        assert!(log.contains("oops stderr"));
    }

    #[test]
    fn write_build_summary_persists_package_statuses_and_log_paths() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.log_base = workspace_root.join("log");
        fs::create_dir_all(config.log_base.join("latest")).unwrap();

        let executor = BuildExecutor::new(&config);
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
        let records = HashMap::from([(
            "demo_pkg".to_string(),
            BuildRecord {
                status: PackageState::Completed,
                duration_ms: 42,
                error: None,
            },
        )]);

        executor.write_build_summary(&packages, &records).unwrap();

        let summary = fs::read_to_string(config.log_base.join("latest/build_summary.log")).unwrap();
        assert!(summary.contains("demo_pkg: status=completed duration_ms=42"));
        assert!(summary.contains("logs="));
        assert!(summary.contains("log/latest/demo_pkg"));
    }

    #[test]
    fn write_package_state_persists_machine_readable_status() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().to_path_buf();
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.log_base = workspace_root.join("log");

        BuildExecutor::write_package_state(
            &config,
            "demo_pkg",
            &BuildRecord {
                status: PackageState::Failed,
                duration_ms: 12,
                error: Some("compile failed".to_string()),
            },
        )
        .unwrap();

        let status =
            fs::read_to_string(config.log_base.join("latest/demo_pkg/status.txt")).unwrap();
        assert!(status.contains("status=failed"));
        assert!(status.contains("duration_ms=12"));
        assert!(status.contains("error=compile failed"));
    }
}
