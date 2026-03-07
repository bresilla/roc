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
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

struct CommandRunLog {
    command: String,
    cwd: Option<PathBuf>,
    stdout: String,
    stderr: String,
    exit_code: i32,
}

struct WorkspaceTester {
    config: TestConfig,
    packages: Vec<PackageMeta>,
    test_order: Vec<usize>,
    install_paths: HashMap<String, PathBuf>,
    run_log_dir: PathBuf,
    event_log: Vec<String>,
    logger_log: Vec<String>,
}

impl WorkspaceTester {
    fn new(config: TestConfig) -> Self {
        let run_log_dir = Self::test_run_log_dir(&config.log_base);
        Self {
            config,
            packages: Vec::new(),
            test_order: Vec::new(),
            install_paths: HashMap::new(),
            run_log_dir,
            event_log: Vec::new(),
            logger_log: Vec::new(),
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
        fs::create_dir_all(&self.run_log_dir)?;

        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        let mut records = HashMap::new();

        let test_order = self.test_order.clone();
        for pkg_idx in test_order {
            let package = self.packages[pkg_idx].clone();
            let start_time = Instant::now();
            self.event_log.push(format!("JobStarted {}", package.name));
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
                    self.test_cmake_package(&package, &env_manager)
                }
                BuildType::AmentPython => self.test_python_package(&package, &env_manager),
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
                        self.write_package_state(&package.name, &record)?;
                        self.write_colcon_test_result(&package, &record)?;
                        self.event_log
                            .push(format!("JobEnded {} failed", package.name));
                        self.write_test_summary(&records)?;
                        self.write_top_level_logs()?;
                        return Err(format!("Tests failed for package {}", package.name).into());
                    }
                }
            }

            records.insert(package.name.clone(), record.clone());
            self.write_package_state(&package.name, &record)?;
            self.write_colcon_test_result(&package, &record)?;
            self.event_log.push(format!(
                "JobEnded {} {}",
                package.name,
                match record.status {
                    TestState::Passed => "passed",
                    TestState::Failed => "failed",
                    TestState::Skipped => "skipped",
                }
            ));
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
        self.write_top_level_logs()?;

        if failed > 0 {
            return Err("Some package tests failed".into());
        }

        Ok(())
    }

    fn test_cmake_package(
        &mut self,
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
        let latest_log = Self::phase_log_path(&self.config, &package.name, "ctest");
        let run_log = self.run_log_dir.join(&package.name).join("ctest.log");
        let command_log = Self::run_command_checked(command, "CTest", &latest_log)?;
        Self::mirror_log_file(&latest_log, &run_log)?;
        self.write_package_log_bundle(package, "ctest", &command_log)?;
        if command_log.exit_code != 0 {
            return Err(format!("CTest failed; see {}", latest_log.display()).into());
        }

        Ok(TestRecord {
            status: TestState::Passed,
            duration_ms: 0,
            error: None,
        })
    }

    fn test_python_package(
        &mut self,
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
        let latest_log = Self::phase_log_path(&self.config, &package.name, "pytest");
        let run_log = self.run_log_dir.join(&package.name).join("pytest.log");
        let command_log = Self::run_command_checked(command, "Pytest", &latest_log)?;
        Self::mirror_log_file(&latest_log, &run_log)?;
        self.write_package_log_bundle(package, "pytest", &command_log)?;
        if command_log.exit_code != 0 {
            return Err(format!("Pytest failed; see {}", latest_log.display()).into());
        }

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
        &self,
        package_name: &str,
        record: &TestRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let status = match record.status {
            TestState::Passed => "passed",
            TestState::Failed => "failed",
            TestState::Skipped => "skipped",
        };
        let mut contents = format!("status={status}\nduration_ms={}\n", record.duration_ms);
        if let Some(error) = &record.error {
            contents.push_str(&format!("error={error}\n"));
        }
        for package_dir in [
            self.config.log_base.join("latest_test").join(package_name),
            self.run_log_dir.join(package_name),
        ] {
            fs::create_dir_all(&package_dir)?;
            fs::write(package_dir.join("status.txt"), &contents)?;
        }
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
        let summary = lines.join("\n");
        fs::create_dir_all(self.config.log_base.join("latest_test"))?;
        fs::write(
            self.config
                .log_base
                .join("latest_test")
                .join("test_summary.log"),
            &summary,
        )?;
        fs::write(self.run_log_dir.join("test_summary.log"), summary)?;
        Ok(())
    }

    fn run_command_checked(
        mut command: Command,
        _description: &str,
        log_path: &Path,
    ) -> Result<CommandRunLog, Box<dyn std::error::Error>> {
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let cwd = command.get_current_dir().map(PathBuf::from);
        let output = command.output()?;
        let rendered_command = format!("{command:?}");
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let log_contents = format!(
            "command={rendered_command}\nexit_code={}\n\nstdout:\n{}\n\nstderr:\n{}\n",
            exit_code, stdout, stderr
        );
        fs::write(log_path, log_contents)?;

        let command_run = CommandRunLog {
            command: rendered_command,
            cwd,
            stdout,
            stderr,
            exit_code,
        };

        Ok(command_run)
    }

    fn write_colcon_test_result(
        &self,
        package: &PackageMeta,
        record: &TestRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.build_base.join(&package.name);
        if !build_dir.exists() {
            return Ok(());
        }

        let rc = match record.status {
            TestState::Passed | TestState::Skipped => "0",
            TestState::Failed => "1",
        };
        fs::write(build_dir.join("colcon_test.rc"), format!("{rc}\n"))?;
        Ok(())
    }

    fn test_run_log_dir(log_base: &Path) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        log_base.join(format!("test_{timestamp}"))
    }

    fn mirror_log_file(
        source: &Path,
        destination: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        if source.exists() {
            fs::copy(source, destination)?;
        }
        Ok(())
    }

    fn write_package_log_bundle(
        &mut self,
        package: &PackageMeta,
        phase: &str,
        command_run: &CommandRunLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let package_dir = package.path.display().to_string();
        let command_log = format!(
            "Invoking command in '{}': {}\nInvoked command in '{}' returned '{}': {}\n",
            package_dir,
            command_run.command,
            package_dir,
            command_run.exit_code,
            command_run.command
        );
        let combined = if command_run.stderr.is_empty() {
            command_run.stdout.clone()
        } else if command_run.stdout.is_empty() {
            command_run.stderr.clone()
        } else {
            format!("{}{}", command_run.stdout, command_run.stderr)
        };

        for package_log_dir in [
            self.config.log_base.join("latest_test").join(&package.name),
            self.run_log_dir.join(&package.name),
        ] {
            fs::create_dir_all(&package_log_dir)?;
            fs::write(package_log_dir.join("command.log"), &command_log)?;
            fs::write(package_log_dir.join("stdout.log"), &command_run.stdout)?;
            fs::write(package_log_dir.join("stderr.log"), &command_run.stderr)?;
            fs::write(package_log_dir.join("stdout_stderr.log"), &combined)?;
            fs::write(package_log_dir.join("streams.log"), &combined)?;
            let phase_log = self
                .config
                .log_base
                .join("latest_test")
                .join(&package.name)
                .join(format!("{phase}.log"));
            if phase_log.exists()
                && package_log_dir != phase_log.parent().unwrap_or(&package_log_dir)
            {
                fs::copy(&phase_log, package_log_dir.join(format!("{phase}.log")))?;
            }
        }

        let cwd = command_run
            .cwd
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| package_dir.clone());
        self.event_log.push(format!(
            "Command {} cwd={} exit_code={}",
            package.name, cwd, command_run.exit_code
        ));
        self.logger_log.push(format!(
            "[{}] {} exit_code={}",
            package.name, phase, command_run.exit_code
        ));
        self.logger_log.push(combined);
        Ok(())
    }

    fn write_top_level_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let events = self.event_log.join("\n");
        let logger = self.logger_log.join("\n");
        for log_dir in [&self.run_log_dir, &self.config.log_base.join("latest_test")] {
            fs::create_dir_all(log_dir)?;
            fs::write(log_dir.join("events.log"), &events)?;
            fs::write(log_dir.join("logger_all.log"), &logger)?;
        }
        Ok(())
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
