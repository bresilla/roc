// Colcon replacement implementation
// This module provides functionality to replace colcon build for ROS 2 workspaces

pub mod command;
pub mod dependency_graph;
pub mod build_executor;
pub mod environment_manager;

#[cfg(test)]
mod compatibility_tests;

// Re-export the handle function for easier access
pub use command::handle;

use colored::Colorize;
use std::path::PathBuf;

pub use crate::shared::package_discovery::{BuildType, Package as PackageMeta};
use crate::shared::package_discovery::{discover_packages, DiscoveryConfig};

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
    pub log_base: PathBuf,
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
            log_base: workspace_root.join("log"),
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
        let discovery_config = DiscoveryConfig {
            base_paths: self.config.base_paths.clone(),
            include_hidden: false,
            max_depth: Some(10),
            exclude_patterns: vec![
                "build".to_string(),
                "install".to_string(),
                "log".to_string(),
                ".git".to_string(),
                ".vscode".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
            ],
        };

        self.packages = discover_packages(&discovery_config)
            .map_err(|e| -> Box<dyn std::error::Error> {
                Box::new(std::io::Error::other(e.to_string()))
            })?;

        if self.packages.is_empty() {
            return Err("No ROS packages found in the selected base paths".into());
        }
        
        // Apply package filters
        self.apply_package_filters();
        
        println!(
            "{} {} {}",
            "Discovered".bright_cyan().bold(),
            self.packages.len().to_string().bright_white().bold(),
            "packages".bright_cyan().bold()
        );
        for pkg in &self.packages {
            println!(
                "  {} {} {}",
                "-".bright_black(),
                pkg.name.bright_white().bold(),
                format!("({:?})", pkg.build_type).bright_black()
            );
        }
        
        Ok(())
    }

    pub fn resolve_dependencies(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.build_order = dependency_graph::topological_sort(&self.packages)?;
        
        println!("{}", "Build order".bright_cyan().bold());
        for &idx in &self.build_order {
            println!("  {}", self.packages[idx].name.bright_white());
        }
        
        Ok(())
    }

    pub fn build_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut executor = build_executor::BuildExecutor::new(&self.config);
        executor.build_all(&self.packages, &self.build_order)?;

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
            for dep in pkg.build_order_deps() {
                if !packages_to_build.contains(&dep) {
                    if self.packages.iter().any(|p| p.name == dep) {
                        packages_to_build.insert(dep.clone());
                        self.add_dependencies_recursive(&dep, packages_to_build);
                    }
                }
            }
        }
    }
}
