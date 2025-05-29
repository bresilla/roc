use clap::ArgMatches;
use std::path::PathBuf;

use crate::commands::work::build::{ColconBuilder, BuildConfig};

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = BuildConfig::default();
    
    // Parse command line arguments
    if let Some(base_paths) = matches.get_many::<String>("base_paths") {
        config.base_paths = base_paths.map(PathBuf::from).collect();
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
    
    // Set build and install directories
    config.build_base = config.workspace_root.join("build");
    config.install_base = config.workspace_root.join("install");
    
    // Update isolated mode based on merge_install flag
    config.isolated = !config.merge_install;
    
    println!("🔧 Building ROS2 workspace with roc (colcon replacement)");
    println!("Workspace: {}", config.workspace_root.display());
    
    // Create and run the builder
    let mut builder = ColconBuilder::new(config);
    
    // Discover packages
    builder.discover_packages()?;
    
    // Resolve dependencies and create build order
    builder.resolve_dependencies()?;
    
    // Build all packages
    builder.build_packages()?;
    
    println!("\n✅ Build completed successfully!");
    println!("To use the workspace, run:");
    println!("  source install/setup.bash");
    
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(run_command(matches)) {
        Ok(_) => {
            println!("Build completed successfully!");
        }
        Err(e) => {
            eprintln!("❌ Build failed: {}", e);
            std::process::exit(1);
        }
    }
}
