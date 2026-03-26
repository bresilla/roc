// Colcon replacement implementation
// This module provides functionality to replace colcon build for ROS 2 workspaces

pub mod build_executor;
pub mod command;
pub mod dependency_graph;
pub mod environment_manager;

#[cfg(test)]
mod compatibility_tests;

// Re-export the handle function for easier access
pub use command::handle;

use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::shared::package_discovery::{
    discover_packages_with_diagnostics, DiscoveryConfig, DiscoveryResult,
};
pub use crate::shared::package_discovery::{BuildType, Package as PackageMeta};

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub base_paths: Vec<PathBuf>,
    pub packages_select: Option<Vec<String>>,
    pub packages_ignore: Option<Vec<String>>,
    pub packages_up_to: Option<Vec<String>>,
    pub packages_select_build_failed: bool,
    pub packages_select_build_finished: bool,
    pub packages_skip_build_finished: bool,
    pub packages_skip_build_failed: bool,
    pub parallel_workers: u32,
    pub merge_install: bool,
    pub symlink_install: bool,
    pub cmake_args: Vec<String>,
    pub cmake_target: Option<String>,
    pub continue_on_error: bool,
    pub strict_discovery: bool,
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
            packages_select_build_failed: false,
            packages_select_build_finished: false,
            packages_skip_build_finished: false,
            packages_skip_build_failed: false,
            parallel_workers: num_cpus::get() as u32,
            merge_install: false,
            symlink_install: false,
            cmake_args: Vec::new(),
            cmake_target: None,
            continue_on_error: false,
            strict_discovery: false,
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

        let DiscoveryResult {
            packages,
            diagnostics,
        } = discover_packages_with_diagnostics(&discovery_config).map_err(
            |e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::other(e.to_string())) },
        )?;
        self.packages = packages;

        self.report_discovery_diagnostics(&diagnostics)?;

        if self.packages.is_empty() {
            return Err("No ROS packages found in the selected base paths".into());
        }

        self.validate_requested_packages()?;

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

    fn report_discovery_diagnostics(
        &self,
        diagnostics: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if diagnostics.is_empty() {
            return Ok(());
        }

        if self.config.strict_discovery {
            let details = diagnostics
                .iter()
                .map(|line| format!("- {line}"))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(format!("Strict discovery failed:\n{details}").into());
        }

        for diagnostic in diagnostics {
            eprintln!(
                "{} {}",
                "Discovery warning:".bright_yellow().bold(),
                diagnostic
            );
        }
        Ok(())
    }

    fn validate_requested_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.strict_discovery {
            return Ok(());
        }

        let discovered_names = self
            .packages
            .iter()
            .map(|pkg| pkg.name.as_str())
            .collect::<HashSet<_>>();
        let mut missing = Vec::new();

        if let Some(selected) = &self.config.packages_select {
            for package in selected {
                if !discovered_names.contains(package.as_str()) {
                    missing.push(package.clone());
                }
            }
        }

        if let Some(up_to) = &self.config.packages_up_to {
            for package in up_to {
                if !discovered_names.contains(package.as_str()) {
                    missing.push(package.clone());
                }
            }
        }

        missing.sort();
        missing.dedup();
        if missing.is_empty() {
            return Ok(());
        }

        Err(format!(
            "Strict discovery failed: requested packages not found: {}",
            missing.join(", ")
        )
        .into())
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
        let mut selected_names: Option<HashSet<String>> = None;
        let previous_state = self.load_previous_build_state();

        if let Some(selected) = &self.config.packages_select {
            let names = selected.iter().cloned().collect::<HashSet<_>>();
            Self::intersect_selected_names(&mut selected_names, names);
        }

        if let Some(up_to) = &self.config.packages_up_to {
            let mut names = HashSet::new();
            for target in up_to {
                if let Some(pkg) = self.packages.iter().find(|p| &p.name == target) {
                    names.insert(pkg.name.clone());
                    self.add_dependencies_recursive(&pkg.name, &mut names);
                }
            }
            Self::intersect_selected_names(&mut selected_names, names);
        }

        if self.config.packages_select_build_failed {
            let names = previous_state
                .iter()
                .filter_map(|(name, status)| {
                    if status == "failed" {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();
            Self::intersect_selected_names(&mut selected_names, names);
        }

        if self.config.packages_select_build_finished {
            let names = previous_state
                .iter()
                .filter_map(|(name, status)| {
                    if status == "completed" {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();
            Self::intersect_selected_names(&mut selected_names, names);
        }

        if let Some(selected_names) = selected_names {
            self.packages
                .retain(|pkg| selected_names.contains(&pkg.name));
        }

        if self.config.packages_skip_build_finished {
            self.packages.retain(|pkg| {
                previous_state
                    .get(&pkg.name)
                    .map(|status| status != "completed")
                    .unwrap_or(true)
            });
        }

        if self.config.packages_skip_build_failed {
            self.packages.retain(|pkg| {
                previous_state
                    .get(&pkg.name)
                    .map(|status| status != "failed")
                    .unwrap_or(true)
            });
        }

        if let Some(ignored) = &self.config.packages_ignore {
            self.packages.retain(|pkg| !ignored.contains(&pkg.name));
        }
    }

    fn intersect_selected_names(
        selected_names: &mut Option<HashSet<String>>,
        new_names: HashSet<String>,
    ) {
        match selected_names {
            Some(existing) => existing.retain(|name| new_names.contains(name)),
            None => *selected_names = Some(new_names),
        }
    }

    fn add_dependencies_recursive(&self, pkg_name: &str, packages_to_build: &mut HashSet<String>) {
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

    fn load_previous_build_state(&self) -> HashMap<String, String> {
        if let Some(state) = self.load_previous_build_state_file() {
            return state;
        }

        let state_root = self.config.log_base.join("latest");
        let entries = match fs::read_dir(state_root) {
            Ok(entries) => entries,
            Err(_) => return HashMap::new(),
        };

        let mut state = HashMap::new();
        for entry in entries.flatten() {
            let package_dir = entry.path();
            if !package_dir.is_dir() {
                continue;
            }

            let package_name = match package_dir.file_name().and_then(|name| name.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            let state_file = package_dir.join("status.txt");
            let contents = match fs::read_to_string(state_file) {
                Ok(contents) => contents,
                Err(_) => continue,
            };

            for line in contents.lines() {
                if let Some(status) = line.strip_prefix("status=") {
                    state.insert(package_name.clone(), status.to_string());
                    break;
                }
            }
        }

        state
    }

    fn load_previous_build_state_file(&self) -> Option<HashMap<String, String>> {
        let state_path = self
            .config
            .log_base
            .join("latest")
            .join("workspace_state.txt");
        let contents = fs::read_to_string(state_path).ok()?;
        let mut lines = contents.lines();
        let workspace_root = lines.next()?.strip_prefix("workspace_root=")?;
        if workspace_root != self.config.workspace_root.to_string_lossy() {
            return Some(HashMap::new());
        }

        let mut state = HashMap::new();
        for line in lines {
            let Some((package_part, status_part)) = line.split_once('\t') else {
                continue;
            };
            let Some(package_name) = package_part.strip_prefix("package=") else {
                continue;
            };
            let Some(status) = status_part.strip_prefix("status=") else {
                continue;
            };
            state.insert(package_name.to_string(), status.to_string());
        }

        Some(state)
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildConfig, BuildType, ColconBuilder, PackageMeta};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn pkg(name: &str, deps: &[&str]) -> PackageMeta {
        PackageMeta {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            build_type: BuildType::AmentCmake,
            version: "0.1.0".to_string(),
            description: "fixture".to_string(),
            maintainers: vec!["Fixture".to_string()],
            depend_deps: Vec::new(),
            build_deps: deps.iter().map(|dep| dep.to_string()).collect(),
            buildtool_deps: Vec::new(),
            build_export_deps: Vec::new(),
            exec_deps: Vec::new(),
            test_deps: Vec::new(),
        }
    }

    #[test]
    fn package_filters_intersect_selection_modes() {
        let temp = tempdir().unwrap();
        let mut config = BuildConfig::default();
        config.log_base = temp.path().join("log");
        config.packages_select = Some(vec!["consumer".to_string(), "leaf".to_string()]);
        config.packages_up_to = Some(vec!["consumer".to_string()]);
        config.packages_select_build_failed = true;
        fs::create_dir_all(config.log_base.join("latest/consumer")).unwrap();
        fs::write(
            config.log_base.join("latest/consumer/status.txt"),
            "status=failed\n",
        )
        .unwrap();
        fs::create_dir_all(config.log_base.join("latest/leaf")).unwrap();
        fs::write(
            config.log_base.join("latest/leaf/status.txt"),
            "status=failed\n",
        )
        .unwrap();

        let mut builder = ColconBuilder::new(config);
        builder.packages = vec![
            pkg("base", &[]),
            pkg("consumer", &["base"]),
            pkg("leaf", &[]),
        ];

        builder.apply_package_filters();

        let selected = builder
            .packages
            .iter()
            .map(|pkg| pkg.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec!["consumer"]);
    }

    #[test]
    fn package_filters_can_skip_previously_completed_packages() {
        let temp = tempdir().unwrap();
        let mut config = BuildConfig::default();
        config.log_base = temp.path().join("log");
        config.packages_skip_build_finished = true;
        fs::create_dir_all(config.log_base.join("latest/base")).unwrap();
        fs::write(
            config.log_base.join("latest/base/status.txt"),
            "status=completed\n",
        )
        .unwrap();
        fs::create_dir_all(config.log_base.join("latest/consumer")).unwrap();
        fs::write(
            config.log_base.join("latest/consumer/status.txt"),
            "status=failed\n",
        )
        .unwrap();

        let mut builder = ColconBuilder::new(config);
        builder.packages = vec![pkg("base", &[]), pkg("consumer", &[]), pkg("fresh", &[])];

        builder.apply_package_filters();

        let selected = builder
            .packages
            .iter()
            .map(|pkg| pkg.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec!["consumer", "fresh"]);
    }

    #[test]
    fn package_filters_can_select_only_previously_completed_packages() {
        let temp = tempdir().unwrap();
        let mut config = BuildConfig::default();
        config.log_base = temp.path().join("log");
        config.packages_select_build_finished = true;
        fs::create_dir_all(config.log_base.join("latest/base")).unwrap();
        fs::write(
            config.log_base.join("latest/base/status.txt"),
            "status=completed\n",
        )
        .unwrap();
        fs::create_dir_all(config.log_base.join("latest/consumer")).unwrap();
        fs::write(
            config.log_base.join("latest/consumer/status.txt"),
            "status=failed\n",
        )
        .unwrap();

        let mut builder = ColconBuilder::new(config);
        builder.packages = vec![pkg("base", &[]), pkg("consumer", &[]), pkg("fresh", &[])];

        builder.apply_package_filters();

        let selected = builder
            .packages
            .iter()
            .map(|pkg| pkg.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec!["base"]);
    }

    #[test]
    fn package_filters_can_skip_previously_failed_packages() {
        let temp = tempdir().unwrap();
        let mut config = BuildConfig::default();
        config.log_base = temp.path().join("log");
        config.packages_skip_build_failed = true;
        fs::create_dir_all(config.log_base.join("latest/base")).unwrap();
        fs::write(
            config.log_base.join("latest/base/status.txt"),
            "status=completed\n",
        )
        .unwrap();
        fs::create_dir_all(config.log_base.join("latest/consumer")).unwrap();
        fs::write(
            config.log_base.join("latest/consumer/status.txt"),
            "status=failed\n",
        )
        .unwrap();

        let mut builder = ColconBuilder::new(config);
        builder.packages = vec![pkg("base", &[]), pkg("consumer", &[]), pkg("fresh", &[])];

        builder.apply_package_filters();

        let selected = builder
            .packages
            .iter()
            .map(|pkg| pkg.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec!["base", "fresh"]);
    }

    #[test]
    fn strict_discovery_rejects_missing_requested_packages() {
        let mut builder = ColconBuilder::new(BuildConfig {
            strict_discovery: true,
            packages_select: Some(vec!["missing_pkg".to_string()]),
            ..BuildConfig::default()
        });
        builder.packages = vec![pkg("demo_pkg", &[])];

        let error = builder.validate_requested_packages().unwrap_err();
        assert!(error
            .to_string()
            .contains("requested packages not found: missing_pkg"));
    }

    #[test]
    fn permissive_discovery_allows_missing_requested_packages() {
        let mut builder = ColconBuilder::new(BuildConfig {
            strict_discovery: false,
            packages_select: Some(vec!["missing_pkg".to_string()]),
            ..BuildConfig::default()
        });
        builder.packages = vec![pkg("demo_pkg", &[])];

        builder.validate_requested_packages().unwrap();
    }

    #[test]
    fn load_previous_build_state_prefers_workspace_state_file_for_matching_root() {
        let temp = tempdir().unwrap();
        let workspace_root = temp.path().join("ws");
        let mut config = BuildConfig::default();
        config.workspace_root = workspace_root.clone();
        config.log_base = temp.path().join("log");
        fs::create_dir_all(config.log_base.join("latest")).unwrap();
        fs::write(
            config.log_base.join("latest/workspace_state.txt"),
            format!(
                "workspace_root={}\npackage=base\tstatus=completed\npackage=consumer\tstatus=failed\n",
                workspace_root.display()
            ),
        )
        .unwrap();

        let builder = ColconBuilder::new(config);
        let state = builder.load_previous_build_state();

        assert_eq!(state.get("base").map(String::as_str), Some("completed"));
        assert_eq!(state.get("consumer").map(String::as_str), Some("failed"));
    }

    #[test]
    fn load_previous_build_state_ignores_workspace_state_from_different_root() {
        let temp = tempdir().unwrap();
        let mut config = BuildConfig::default();
        config.workspace_root = temp.path().join("current_ws");
        config.log_base = temp.path().join("log");
        fs::create_dir_all(config.log_base.join("latest/base")).unwrap();
        fs::write(
            config.log_base.join("latest/base/status.txt"),
            "status=completed\n",
        )
        .unwrap();
        fs::write(
            config.log_base.join("latest/workspace_state.txt"),
            format!(
                "workspace_root={}\npackage=base\tstatus=failed\n",
                temp.path().join("other_ws").display()
            ),
        )
        .unwrap();

        let builder = ColconBuilder::new(config);
        let state = builder.load_previous_build_state();

        assert!(state.is_empty());
    }

    #[test]
    fn load_previous_build_state_falls_back_to_status_files_when_state_file_missing() {
        let temp = tempdir().unwrap();
        let mut config = BuildConfig::default();
        config.workspace_root = temp.path().join("ws");
        config.log_base = temp.path().join("log");
        fs::create_dir_all(config.log_base.join("latest/base")).unwrap();
        fs::write(
            config.log_base.join("latest/base/status.txt"),
            "status=completed\n",
        )
        .unwrap();

        let builder = ColconBuilder::new(config);
        let state = builder.load_previous_build_state();

        assert_eq!(state.get("base").map(String::as_str), Some("completed"));
    }
}
