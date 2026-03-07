use crate::commands::cli::handle_boxed_command_result;
use crate::commands::work::build::dependency_graph;
use crate::commands::work::build::environment_manager::EnvironmentManager;
use crate::commands::work::build::{BuildType, PackageMeta};
use crate::shared::package_discovery::{DiscoveryConfig, discover_packages};
use clap::ArgMatches;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
struct TestConfig {
    base_paths: Vec<PathBuf>,
    packages_select: Option<Vec<String>>,
    packages_ignore: Option<Vec<String>>,
    packages_up_to: Option<Vec<String>>,
    continue_on_error: bool,
    merge_install: bool,
    ctest_args: Vec<String>,
    pytest_args: Vec<String>,
    workspace_root: PathBuf,
    install_base: PathBuf,
    build_base: PathBuf,
    log_base: PathBuf,
    isolated: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_default();
        Self {
            base_paths: vec![PathBuf::from("src")],
            packages_select: None,
            packages_ignore: None,
            packages_up_to: None,
            continue_on_error: true,
            merge_install: false,
            ctest_args: Vec::new(),
            pytest_args: Vec::new(),
            workspace_root: workspace_root.clone(),
            install_base: workspace_root.join("install"),
            build_base: workspace_root.join("build"),
            log_base: workspace_root.join("log"),
            isolated: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestState {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
struct TestRecord {
    status: TestState,
    duration_ms: u128,
    error: Option<String>,
}

struct WorkspaceTester {
    config: TestConfig,
    packages: Vec<PackageMeta>,
    test_order: Vec<usize>,
    install_paths: HashMap<String, PathBuf>,
}

impl WorkspaceTester {
    fn new(config: TestConfig) -> Self {
        Self {
            config,
            packages: Vec::new(),
            test_order: Vec::new(),
            install_paths: HashMap::new(),
        }
    }

    fn discover_packages(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        self.packages =
            discover_packages(&discovery_config).map_err(|e| -> Box<dyn std::error::Error> {
                Box::new(std::io::Error::other(e.to_string()))
            })?;

        if self.packages.is_empty() {
            return Err("No ROS packages found in the selected base paths".into());
        }

        self.apply_package_filters();

        println!(
            "{} {} {}",
            "Discovered".bright_cyan().bold(),
            self.packages.len().to_string().bright_white().bold(),
            "packages for testing".bright_cyan().bold()
        );

        Ok(())
    }

    fn resolve_dependencies(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.test_order = dependency_graph::topological_sort(&self.packages)?;

        println!("{}", "Test order".bright_cyan().bold());
        for &idx in &self.test_order {
            println!("  {}", self.packages[idx].name.bright_white());
        }

        Ok(())
    }

    fn run_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(self.config.log_base.join("latest_test"))?;

        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        let mut records = HashMap::new();

        for &pkg_idx in &self.test_order {
            let package = &self.packages[pkg_idx];
            let start_time = Instant::now();
            println!(
                "{} {} {}",
                "Testing >>>".bright_cyan().bold(),
                package.name.bright_white().bold(),
                format!("({:?})", package.build_type).bright_black()
            );

            let mut env_manager =
                EnvironmentManager::new(self.config.install_base.clone(), self.config.isolated);
            env_manager.setup_package_environment(&package.name, &package.path)?;
            for dep_name in package.build_order_deps() {
                if let Some(dep_path) = self
                    .install_paths
                    .get(&dep_name)
                    .cloned()
                    .or_else(|| Self::existing_install_path(&self.config, &dep_name))
                {
                    env_manager.setup_package_environment(&dep_name, &dep_path)?;
                }
            }

            let result = match package.build_type {
                BuildType::AmentCmake | BuildType::Cmake => {
                    self.test_cmake_package(package, &env_manager)
                }
                BuildType::AmentPython => self.test_python_package(package, &env_manager),
                BuildType::Other(ref build_type) => Ok(TestRecord {
                    status: TestState::Skipped,
                    duration_ms: 0,
                    error: Some(format!("Unsupported build type for testing: {build_type}")),
                }),
            };

            let record = match result {
                Ok(mut record) => {
                    record.duration_ms = start_time.elapsed().as_millis();
                    record
                }
                Err(error) => TestRecord {
                    status: TestState::Failed,
                    duration_ms: start_time.elapsed().as_millis(),
                    error: Some(error.to_string()),
                },
            };

            self.install_paths.insert(
                package.name.clone(),
                Self::existing_install_path(&self.config, &package.name)
                    .unwrap_or_else(|| self.config.install_base.join(&package.name)),
            );

            match record.status {
                TestState::Passed => {
                    passed += 1;
                    println!(
                        "{} {} {}",
                        "Passed <<<".bright_green().bold(),
                        package.name.bright_white().bold(),
                        format!("[{:.2}s]", record.duration_ms as f64 / 1000.0).bright_black()
                    );
                }
                TestState::Skipped => {
                    skipped += 1;
                    println!(
                        "{} {} {}",
                        "Skipped <<<".bright_yellow().bold(),
                        package.name.bright_white().bold(),
                        record
                            .error
                            .as_deref()
                            .unwrap_or("No runnable tests detected")
                            .bright_black()
                    );
                }
                TestState::Failed => {
                    failed += 1;
                    eprintln!(
                        "{} {} - {}",
                        "Failed <<<".bright_red().bold(),
                        package.name.bright_white().bold(),
                        record
                            .error
                            .as_deref()
                            .unwrap_or("unknown failure")
                            .bright_white()
                    );
                    if !self.config.continue_on_error {
                        records.insert(package.name.clone(), record.clone());
                        Self::write_package_state(&self.config, &package.name, &record)?;
                        self.write_test_summary(&records)?;
                        return Err(format!("Tests failed for package {}", package.name).into());
                    }
                }
            }

            records.insert(package.name.clone(), record.clone());
            Self::write_package_state(&self.config, &package.name, &record)?;
        }

        println!("\n{}", "Test Summary".bright_cyan().bold());
        println!(
            "  {} {}",
            passed.to_string().bright_green().bold(),
            "packages passed".bright_green()
        );
        if skipped > 0 {
            println!(
                "  {} {}",
                skipped.to_string().bright_yellow().bold(),
                "packages skipped".bright_yellow()
            );
        }
        if failed > 0 {
            println!(
                "  {} {}",
                failed.to_string().bright_red().bold(),
                "packages failed".bright_red()
            );
        }

        self.write_test_summary(&records)?;

        if failed > 0 {
            return Err("Some package tests failed".into());
        }

        Ok(())
    }

    fn test_cmake_package(
        &self,
        package: &PackageMeta,
        env_manager: &EnvironmentManager,
    ) -> Result<TestRecord, Box<dyn std::error::Error>> {
        let build_dir = self.config.build_base.join(&package.name);
        if !build_dir.exists() {
            return Ok(TestRecord {
                status: TestState::Skipped,
                duration_ms: 0,
                error: Some(format!(
                    "Build directory missing at {}; run 'roc work build' first",
                    build_dir.display()
                )),
            });
        }

        let mut command = Command::new("ctest");
        command.args(Self::ctest_args(&build_dir, &self.config.ctest_args));
        command.envs(env_manager.get_env_vars());
        Self::run_command_checked(
            command,
            "CTest",
            &Self::phase_log_path(&self.config, &package.name, "ctest"),
        )?;

        Ok(TestRecord {
            status: TestState::Passed,
            duration_ms: 0,
            error: None,
        })
    }

    fn test_python_package(
        &self,
        package: &PackageMeta,
        env_manager: &EnvironmentManager,
    ) -> Result<TestRecord, Box<dyn std::error::Error>> {
        if !Self::has_python_tests(&package.path) {
            return Ok(TestRecord {
                status: TestState::Skipped,
                duration_ms: 0,
                error: Some("No pytest-compatible tests detected".to_string()),
            });
        }

        let mut command = Command::new("python3");
        let build_dir = self.config.build_base.join(&package.name);
        let pytest_xml = build_dir.join("pytest.xml");
        let pytest_addopts = format!(
            "--tb=short --junit-xml={} --junit-prefix={} -o cache_dir={}",
            pytest_xml.display(),
            package.name,
            build_dir.join(".pytest_cache").display()
        );
        command
            .args(Self::pytest_args(&self.config.pytest_args))
            .current_dir(&package.path)
            .envs(env_manager.get_env_vars())
            .env("PYTEST_ADDOPTS", pytest_addopts)
            .env("PYTHONDONTWRITEBYTECODE", "1");
        Self::run_command_checked(
            command,
            "Pytest",
            &Self::phase_log_path(&self.config, &package.name, "pytest"),
        )?;

        Ok(TestRecord {
            status: TestState::Passed,
            duration_ms: 0,
            error: None,
        })
    }

    fn ctest_args(build_dir: &Path, extra_args: &[String]) -> Vec<String> {
        let mut args = vec![
            "--test-dir".to_string(),
            build_dir.display().to_string(),
            "--output-on-failure".to_string(),
        ];
        args.extend(extra_args.iter().cloned());
        args
    }

    fn pytest_args(extra_args: &[String]) -> Vec<String> {
        let mut args = vec!["-m".to_string(), "pytest".to_string()];
        args.extend(extra_args.iter().cloned());
        args
    }

    fn has_python_tests(package_path: &Path) -> bool {
        package_path.join("tests").exists()
            || package_path.join("pytest.ini").is_file()
            || package_path.join("pyproject.toml").is_file()
            || package_path.join("setup.cfg").is_file()
    }

    fn existing_install_path(config: &TestConfig, package_name: &str) -> Option<PathBuf> {
        if config.merge_install {
            let share_dir = config.install_base.join("share").join(package_name);
            if share_dir.exists() {
                return Some(config.install_base.clone());
            }
            None
        } else {
            let package_prefix = config.install_base.join(package_name);
            package_prefix.exists().then_some(package_prefix)
        }
    }

    fn phase_log_path(config: &TestConfig, package_name: &str, phase: &str) -> PathBuf {
        config
            .log_base
            .join("latest_test")
            .join(package_name)
            .join(format!("{phase}.log"))
    }

    fn write_package_state(
        config: &TestConfig,
        package_name: &str,
        record: &TestRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let package_dir = config.log_base.join("latest_test").join(package_name);
        fs::create_dir_all(&package_dir)?;
        let status = match record.status {
            TestState::Passed => "passed",
            TestState::Failed => "failed",
            TestState::Skipped => "skipped",
        };
        let mut contents = format!("status={status}\nduration_ms={}\n", record.duration_ms);
        if let Some(error) = &record.error {
            contents.push_str(&format!("error={error}\n"));
        }
        fs::write(package_dir.join("status.txt"), contents)?;
        Ok(())
    }

    fn write_test_summary(
        &self,
        records: &HashMap<String, TestRecord>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut lines = vec!["roc work test summary".to_string()];
        let mut names = records.keys().cloned().collect::<Vec<_>>();
        names.sort();
        for name in names {
            if let Some(record) = records.get(&name) {
                let status = match record.status {
                    TestState::Passed => "passed",
                    TestState::Failed => "failed",
                    TestState::Skipped => "skipped",
                };
                let mut line = format!(
                    "{name}: status={status} duration_ms={} log=log/latest_test/{name}",
                    record.duration_ms
                );
                if let Some(error) = &record.error {
                    line.push_str(&format!(" error={error}"));
                }
                lines.push(line);
            }
        }
        fs::create_dir_all(self.config.log_base.join("latest_test"))?;
        fs::write(
            self.config
                .log_base
                .join("latest_test")
                .join("test_summary.log"),
            lines.join("\n"),
        )?;
        Ok(())
    }

    fn run_command_checked(
        mut command: Command,
        description: &str,
        log_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let output = command.output()?;
        let rendered_command = format!("{command:?}");
        let log_contents = format!(
            "command={rendered_command}\nexit_code={}\n\nstdout:\n{}\n\nstderr:\n{}\n",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::write(log_path, log_contents)?;

        if output.status.success() {
            return Ok(());
        }

        Err(format!("{description} failed; see {}", log_path.display()).into())
    }

    fn apply_package_filters(&mut self) {
        let mut selected_names: Option<HashSet<String>> = None;

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

        if let Some(selected_names) = selected_names {
            self.packages
                .retain(|pkg| selected_names.contains(&pkg.name));
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

    fn add_dependencies_recursive(&self, pkg_name: &str, packages_to_test: &mut HashSet<String>) {
        if let Some(pkg) = self.packages.iter().find(|p| p.name == pkg_name) {
            for dep in pkg.build_order_deps() {
                if !packages_to_test.contains(&dep) && self.packages.iter().any(|p| p.name == dep) {
                    packages_to_test.insert(dep.clone());
                    self.add_dependencies_recursive(&dep, packages_to_test);
                }
            }
        }
    }
}

fn config_from_matches(matches: &ArgMatches) -> Result<TestConfig, Box<dyn std::error::Error>> {
    let mut config = TestConfig::default();
    let mut user_provided_base_paths = false;

    if let Some(base_paths) = matches.get_many::<String>("base_paths") {
        user_provided_base_paths = true;
        config.base_paths = base_paths.map(PathBuf::from).collect();
    }

    if let Some(build_base) = matches.get_one::<String>("build_base") {
        config.build_base = PathBuf::from(build_base);
    }

    if let Some(install_base) = matches.get_one::<String>("install_base") {
        config.install_base = PathBuf::from(install_base);
    }

    if let Some(log_base) = matches.get_one::<String>("log_base") {
        config.log_base = PathBuf::from(log_base);
    }

    if let Some(packages) = matches.get_many::<String>("packages_select") {
        config.packages_select = Some(packages.map(|pkg| pkg.to_string()).collect());
    }

    if let Some(packages) = matches.get_many::<String>("packages_ignore") {
        config.packages_ignore = Some(packages.map(|pkg| pkg.to_string()).collect());
    }

    if let Some(packages) = matches.get_many::<String>("packages_up_to") {
        config.packages_up_to = Some(packages.map(|pkg| pkg.to_string()).collect());
    }

    if let Some(args) = matches.get_many::<String>("ctest_args") {
        config.ctest_args = args.map(|arg| arg.to_string()).collect();
    }

    if let Some(args) = matches.get_many::<String>("pytest_args") {
        config.pytest_args = args.map(|arg| arg.to_string()).collect();
    }

    config.continue_on_error = config.continue_on_error || matches.get_flag("continue_on_error");
    config.merge_install = matches.get_flag("merge_install");
    config.isolated = !config.merge_install;
    config.workspace_root = std::env::current_dir()?;

    if !config.build_base.is_absolute() {
        config.build_base = config.workspace_root.join(&config.build_base);
    }
    if !config.install_base.is_absolute() {
        config.install_base = config.workspace_root.join(&config.install_base);
    }
    if !config.log_base.is_absolute() {
        config.log_base = config.workspace_root.join(&config.log_base);
    }

    if !user_provided_base_paths && !config.workspace_root.join("src").exists() {
        config.base_paths = vec![config.workspace_root.clone()];
        println!(
            "{}",
            "No 'src' directory found; scanning workspace root for packages".bright_yellow()
        );
    }

    Ok(config)
}

fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = config_from_matches(&matches)?;

    println!("{}", "Testing ROS2 workspace with roc".bright_cyan().bold());
    println!(
        "{} {}",
        "Workspace:".bright_blue().bold(),
        config.workspace_root.display().to_string().bright_white()
    );

    let mut tester = WorkspaceTester::new(config);
    tester.discover_packages()?;
    tester.resolve_dependencies()?;
    tester.run_tests()?;

    println!("\n{}", "Tests completed successfully".bright_green().bold());
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_boxed_command_result(run_command(matches));
}

#[cfg(test)]
mod tests {
    use super::{TestConfig, WorkspaceTester, config_from_matches};
    use crate::commands::work::build::{BuildType, PackageMeta};
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
    fn config_from_matches_uses_custom_base_directories() {
        let matches = crate::arguments::work::cmd()
            .try_get_matches_from([
                "work",
                "test",
                "--build-base",
                "out/build-tree",
                "--install-base",
                "out/install-tree",
                "--log-base",
                "out/log-tree",
                "--ctest-args",
                "-R",
                "demo",
            ])
            .unwrap();
        let (_, submatches) = matches.subcommand().unwrap();

        let config = config_from_matches(submatches).unwrap();
        assert!(config.build_base.ends_with("out/build-tree"));
        assert!(config.install_base.ends_with("out/install-tree"));
        assert!(config.log_base.ends_with("out/log-tree"));
        assert_eq!(
            config.ctest_args,
            vec!["-R".to_string(), "demo".to_string()]
        );
    }

    #[test]
    fn package_filters_respect_packages_up_to() {
        let temp = tempdir().unwrap();
        let mut config = TestConfig::default();
        config.workspace_root = temp.path().to_path_buf();
        config.packages_up_to = Some(vec!["consumer".to_string()]);
        let mut tester = WorkspaceTester::new(config);
        tester.packages = vec![
            pkg("base", &[]),
            pkg("consumer", &["base"]),
            pkg("leaf", &[]),
        ];

        tester.apply_package_filters();

        let selected = tester
            .packages
            .iter()
            .map(|pkg| pkg.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec!["base", "consumer"]);
    }

    #[test]
    fn pytest_args_prefix_python_module_invocation() {
        assert_eq!(
            WorkspaceTester::pytest_args(&["-q".to_string()]),
            vec!["-m".to_string(), "pytest".to_string(), "-q".to_string()]
        );
    }

    #[test]
    fn test_config_defaults_to_continue_on_error() {
        let config = TestConfig::default();
        assert!(config.continue_on_error);
    }
}
