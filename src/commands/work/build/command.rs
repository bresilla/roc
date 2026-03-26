use clap::ArgMatches;
use colored::Colorize;
use std::path::PathBuf;

use crate::commands::work::build::{BuildConfig, ColconBuilder};

fn config_from_matches(matches: &ArgMatches) -> Result<BuildConfig, Box<dyn std::error::Error>> {
    let mut config = BuildConfig::default();
    let mut user_provided_base_paths = false;

    // Parse command line arguments
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
        config.packages_select = Some(packages.map(|s| s.to_string()).collect());
    }

    if let Some(packages) = matches.get_many::<String>("packages_ignore") {
        config.packages_ignore = Some(packages.map(|s| s.to_string()).collect());
    }

    if let Some(packages) = matches.get_many::<String>("packages_up_to") {
        config.packages_up_to = Some(packages.map(|s| s.to_string()).collect());
    }

    config.packages_select_build_failed = matches.get_flag("packages_select_build_failed");
    config.packages_select_build_finished = matches.get_flag("packages_select_build_finished");
    config.packages_skip_build_finished = matches.get_flag("packages_skip_build_finished");
    config.packages_skip_build_failed = matches.get_flag("packages_skip_build_failed");

    if let Some(workers) = matches.get_one::<u32>("parallel_workers") {
        config.parallel_workers = *workers;
    }

    config.merge_install = matches.get_flag("merge_install");
    config.symlink_install = matches.get_flag("symlink_install");
    config.continue_on_error = matches.get_flag("continue_on_error");
    config.strict_discovery = matches.get_flag("strict_discovery");

    if let Some(cmake_args) = matches.get_many::<String>("cmake_args") {
        config.cmake_args = cmake_args.map(|s| s.to_string()).collect();
    }

    if let Some(target) = matches.get_one::<String>("cmake_target") {
        config.cmake_target = Some(target.to_string());
    }

    // Set workspace root to current directory
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

    // Colcon-compatible fallback: if the default `src` path does not exist,
    // discover packages from the workspace root.
    if !user_provided_base_paths && !config.workspace_root.join("src").exists() {
        config.base_paths = vec![config.workspace_root.clone()];
        println!(
            "{}",
            "No 'src' directory found; scanning workspace root for packages".bright_yellow()
        );
    }

    // Update isolated mode based on merge_install flag
    config.isolated = !config.merge_install;

    Ok(config)
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = config_from_matches(&matches)?;

    println!(
        "{}",
        "Building ROS2 workspace with roc (colcon replacement)"
            .bright_cyan()
            .bold()
    );
    println!(
        "{} {}",
        "Workspace:".bright_blue().bold(),
        config.workspace_root.display().to_string().bright_white()
    );

    // Create and run the builder
    let mut builder = ColconBuilder::new(config);

    // Discover packages
    builder.discover_packages()?;
    builder.validate_build_preconditions()?;

    // Resolve dependencies and create build order
    builder.resolve_dependencies()?;

    // Build all packages
    builder.build_packages()?;

    println!("\n{}", "Build completed successfully".bright_green().bold());
    println!("{}", "To use the workspace, run: ".bright_blue().bold());
    println!("  {}", "source install/setup.bash".bright_white());

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(run_command(matches)) {
        Ok(_) => {
            println!("{}", "Done".bright_green().bold());
        }
        Err(e) => {
            eprintln!(
                "{} {}",
                "Build failed:".bright_red().bold(),
                e.to_string().bright_white()
            );
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::config_from_matches;

    #[test]
    fn config_from_matches_uses_custom_base_directories() {
        let matches = crate::arguments::work::cmd()
            .try_get_matches_from([
                "work",
                "build",
                "--build-base",
                "out/build-tree",
                "--install-base",
                "out/install-tree",
                "--log-base",
                "out/log-tree",
            ])
            .unwrap();
        let (_, submatches) = matches.subcommand().unwrap();

        let config = config_from_matches(submatches).unwrap();

        assert!(config.build_base.ends_with("out/build-tree"));
        assert!(config.install_base.ends_with("out/install-tree"));
        assert!(config.log_base.ends_with("out/log-tree"));
    }

    #[test]
    fn config_from_matches_parses_build_state_selectors() {
        let matches = crate::arguments::work::cmd()
            .try_get_matches_from([
                "work",
                "build",
                "--packages-select-build-failed",
                "--packages-select-build-finished",
            ])
            .unwrap();
        let (_, submatches) = matches.subcommand().unwrap();

        let config = config_from_matches(submatches).unwrap();

        assert!(config.packages_select_build_failed);
        assert!(config.packages_select_build_finished);
    }

    #[test]
    fn config_from_matches_rejects_conflicting_build_state_selectors() {
        let result = crate::arguments::work::cmd().try_get_matches_from([
            "work",
            "build",
            "--packages-select-build-failed",
            "--packages-skip-build-finished",
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn config_from_matches_enables_strict_discovery() {
        let matches = crate::arguments::work::cmd()
            .try_get_matches_from(["work", "build", "--strict-discovery"])
            .unwrap();
        let (_, submatches) = matches.subcommand().unwrap();

        let config = config_from_matches(submatches).unwrap();

        assert!(config.strict_discovery);
    }
}
