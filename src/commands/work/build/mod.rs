// Colcon replacement implementation
// This module provides functionality to replace colcon build for ROS 2 workspaces

pub mod command;
pub mod package_discovery;
pub mod dependency_graph;
pub mod build_executor;
pub mod environment_manager;

// Re-export the handle function for easier access
pub use command::handle;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum BuildType {
    AmentCmake,
    AmentPython,
    Cmake,
    Other(String),
}

impl From<&str> for BuildType {
    fn from(s: &str) -> Self {
        match s {
            "ament_cmake" => BuildType::AmentCmake,
            "ament_python" => BuildType::AmentPython,
            "cmake" => BuildType::Cmake,
            other => BuildType::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageMeta {
    pub name: String,
    pub path: PathBuf,
    pub build_type: BuildType,
    pub version: String,
    pub description: String,
    pub maintainers: Vec<String>,
    pub build_deps: Vec<String>,
    pub buildtool_deps: Vec<String>,
    pub exec_deps: Vec<String>,
    pub test_deps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub base_paths: Vec<PathBuf>,
    pub packages_select: Option<Vec<String>>,
    pub packages_ignore: Option<Vec<String>>,
    pub packages_up_to: Option<Vec<String>>,
    pub parallel_workers: u32,
    pub merge_install: bool,
    pub symlink_install: bool,
    pub cmake_args: Vec<String>,
    pub cmake_target: Option<String>,
    pub continue_on_error: bool,
    pub workspace_root: PathBuf,
    pub install_base: PathBuf,
    pub build_base: PathBuf,
    pub isolated: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_default();
        Self {
            base_paths: vec![PathBuf::from("src")],
            packages_select: None,
            packages_ignore: None,
            packages_up_to: None,
            parallel_workers: num_cpus::get() as u32,
            merge_install: false,
            symlink_install: false,
            cmake_args: Vec::new(),
            cmake_target: None,
            continue_on_error: false,
            workspace_root: workspace_root.clone(),
            install_base: workspace_root.join("install"),
            build_base: workspace_root.join("build"),
            isolated: true,
        }
    }
}

pub struct ColconBuilder {
    config: BuildConfig,
    packages: Vec<PackageMeta>,
    build_order: Vec<usize>, // indices into packages vec
}

impl ColconBuilder {
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            packages: Vec::new(),
            build_order: Vec::new(),
        }
    }

    pub fn discover_packages(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.packages = package_discovery::discover_packages(&self.config.base_paths)?;
        
        // Apply package filters
        self.apply_package_filters();
        
        println!("Discovered {} packages", self.packages.len());
        for pkg in &self.packages {
            println!("  - {} ({})", pkg.name, format!("{:?}", pkg.build_type));
        }
        
        Ok(())
    }

    pub fn resolve_dependencies(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.build_order = dependency_graph::topological_sort(&self.packages)?;
        
        println!("Build order:");
        for &idx in &self.build_order {
            println!("  {}", self.packages[idx].name);
        }
        
        Ok(())
    }

    pub fn build_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut executor = build_executor::BuildExecutor::new(&self.config);
        executor.build_all(&self.packages, &self.build_order)?;
        
        // Generate setup scripts
        self.generate_setup_scripts()?;
        
        Ok(())
    }
    
    /// Generate setup scripts for the workspace
    pub fn generate_setup_scripts(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::commands::work::build::environment_manager::EnvironmentManager;
        
        let env_manager = EnvironmentManager::new(
            self.config.install_base.clone(),
            self.config.isolated
        );
        
        // Generate main setup script
        let setup_bash_path = self.config.install_base.join("setup.bash");
        env_manager.generate_setup_script(&setup_bash_path)?;
        
        println!("📝 Generated setup script: {}", setup_bash_path.display());
        
        Ok(())
    }

    fn apply_package_filters(&mut self) {
        // Apply packages_select filter
        if let Some(ref selected) = self.config.packages_select {
            self.packages.retain(|pkg| selected.contains(&pkg.name));
        }

        // Apply packages_ignore filter
        if let Some(ref ignored) = self.config.packages_ignore {
            self.packages.retain(|pkg| !ignored.contains(&pkg.name));
        }

        // Apply packages_up_to filter (build dependencies up to the specified packages)
        if let Some(ref up_to) = self.config.packages_up_to {
            let mut packages_to_build = std::collections::HashSet::new();
            
            // Add the target packages
            for target in up_to {
                if let Some(pkg) = self.packages.iter().find(|p| &p.name == target) {
                    packages_to_build.insert(pkg.name.clone());
                    // Add all dependencies recursively
                    self.add_dependencies_recursive(&pkg.name, &mut packages_to_build);
                }
            }
            
            self.packages.retain(|pkg| packages_to_build.contains(&pkg.name));
        }
    }

    fn add_dependencies_recursive(&self, pkg_name: &str, packages_to_build: &mut std::collections::HashSet<String>) {
        if let Some(pkg) = self.packages.iter().find(|p| &p.name == pkg_name) {
            for dep in &pkg.build_deps {
                if !packages_to_build.contains(dep) {
                    if self.packages.iter().any(|p| &p.name == dep) {
                        packages_to_build.insert(dep.clone());
                        self.add_dependencies_recursive(dep, packages_to_build);
                    }
                }
            }
        }
    }
}
