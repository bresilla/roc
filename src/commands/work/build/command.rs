use clap::ArgMatches;
use std::path::PathBuf;

use crate::commands::cli::run_async_command;
use crate::commands::work::build::{BuildConfig, ColconBuilder};
use crate::ui::blocks;

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
        blocks::print_note("No 'src' directory found; scanning workspace root for packages");
    }

    // Update isolated mode based on merge_install flag
    config.isolated = !config.merge_install;

    Ok(config)
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = config_from_matches(&matches)?;
    let setup_path = config.install_base.join("setup.bash");

    blocks::print_section("Build");
    blocks::print_field("Workspace", config.workspace_root.display());
    blocks::print_field("Build Base", config.build_base.display());
    blocks::print_field("Install Base", config.install_base.display());
    blocks::print_field("Log Base", config.log_base.display());
    blocks::print_field(
        "Layout",
        if config.merge_install {
            "merged"
        } else {
            "isolated"
        },
    );
    blocks::print_field("Workers", config.parallel_workers);
    println!();

    // Create and run the builder
    let mut builder = ColconBuilder::new(config);

    // Discover packages
    builder.discover_packages()?;

    // Resolve dependencies and create build order
    builder.resolve_dependencies()?;

    // Build all packages
    builder.build_packages()?;

    println!();
    blocks::print_success("Build completed successfully");
    blocks::print_field("Setup", format!("source {}", setup_path.display()));

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    run_async_command(run_command(matches));
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
}
