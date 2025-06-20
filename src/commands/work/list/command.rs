use clap::ArgMatches;
use colored::*;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use crate::shared::package_discovery::{discover_packages, DiscoveryConfig, BuildType};

fn format_build_status(package_path: &PathBuf, build_base: &PathBuf, install_base: &PathBuf) -> String {
    let package_name = package_path.file_name().unwrap().to_string_lossy();
    let build_dir = build_base.join(&*package_name);
    let install_dir = install_base.join(&*package_name);
    
    if install_dir.exists() {
        "✓ Built".green().to_string()
    } else if build_dir.exists() {
        "⚠ Partial".yellow().to_string()
    } else {
        "✗ Not built".red().to_string()
    }
}

fn format_build_type(build_type: &BuildType) -> String {
    match build_type {
        BuildType::AmentCmake => "ament_cmake".blue().to_string(),
        BuildType::AmentPython => "ament_python".green().to_string(),
        BuildType::Cmake => "cmake".cyan().to_string(),
        BuildType::Other(s) => s.purple().to_string(),
    }
}

fn get_creation_time(package_path: &PathBuf) -> String {
    if let Ok(metadata) = fs::metadata(package_path.join("package.xml")) {
        if let Ok(created) = metadata.created() {
            if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                let datetime = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0);
                if let Some(dt) = datetime {
                    return dt.format("%Y-%m-%d %H:%M").to_string();
                }
            }
        }
    }
    "Unknown".to_string()
}

async fn run_command(matches: ArgMatches) -> Result<()> {
    let workspace_root = std::env::current_dir()?;
    let build_base = workspace_root.join("build");
    let install_base = workspace_root.join("install");
    
    // Use the new shared discovery system - more flexible than just /src
    let config = DiscoveryConfig {
        base_paths: vec![workspace_root.clone()],
        include_hidden: matches.get_flag("all"),
        max_depth: Some(10), // Reasonable depth for workspace
        exclude_patterns: vec![
            "build".to_string(),
            "install".to_string(), 
            "log".to_string(),
            ".git".to_string(),
            ".vscode".to_string(),
            "target".to_string(), // Rust build dir
            "node_modules".to_string(),
            "__pycache__".to_string(),
        ],
    };
    
    let packages = discover_packages(&config)?;
    
    if packages.is_empty() {
        println!("{}", "No ROS 2 packages found in the workspace.".yellow());
        return Ok(());
    }
    
    // Check if user wants count only
    if matches.get_flag("count_packages") {
        println!("{}", packages.len());
        return Ok(());
    }
    
    // Print header
    println!("{}", "ROS 2 Packages in Workspace".bright_cyan().bold());
    println!("{}", "=".repeat(80).bright_black());
    
    // Sort packages by name for consistent output
    let mut sorted_packages = packages.clone();
    sorted_packages.sort_by(|a, b| a.name.cmp(&b.name));
    
    for package in &sorted_packages {
        let status = format_build_status(&package.path, &build_base, &install_base);
        let build_type = format_build_type(&package.build_type);
        let created = get_creation_time(&package.path);
        
        println!(
            "{:<25} {:<15} {:<20} {}",
            package.name.bright_white().bold(),
            build_type,
            status,
            created.bright_black()
        );
    }
    
    println!();
    println!(
        "{} {} packages found",
        "Total:".bright_cyan(),
        packages.len().to_string().bright_white().bold()
    );
    
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(run_command(matches)) {
        eprintln!("{}: {}", "Error".red().bold(), e);
        std::process::exit(1);
    }
}
