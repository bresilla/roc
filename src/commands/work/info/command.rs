use clap::ArgMatches;
use colored::*;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use crate::shared::package_discovery::{discover_packages, DiscoveryConfig, BuildType};
use roxmltree::Document;

fn format_build_type(build_type: &BuildType) -> String {
    match build_type {
        BuildType::AmentCmake => "ament_cmake".blue().to_string(),
        BuildType::AmentPython => "ament_python".green().to_string(),
        BuildType::Cmake => "cmake".cyan().to_string(),
        BuildType::Other(s) => s.purple().to_string(),
    }
}

fn print_section_header(title: &str) {
    println!("\n{}", title.bright_cyan().bold());
    println!("{}", "-".repeat(title.len()).bright_black());
}

fn print_dependencies(deps: &[String], title: &str) {
    if !deps.is_empty() {
        print_section_header(title);
        for dep in deps {
            println!("  • {}", dep.bright_white());
        }
    }
}

fn extract_licenses(xml_content: &str) -> Vec<String> {
    if let Ok(doc) = Document::parse(xml_content) {
        doc.root_element()
            .descendants()
            .filter(|n| n.has_tag_name("license"))
            .filter_map(|n| n.text())
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    }
}

fn extract_authors(xml_content: &str) -> Vec<String> {
    if let Ok(doc) = Document::parse(xml_content) {
        doc.root_element()
            .descendants()
            .filter(|n| n.has_tag_name("author"))
            .filter_map(|n| n.text())
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    }
}

fn extract_urls(xml_content: &str) -> Vec<(String, String)> {
    if let Ok(doc) = Document::parse(xml_content) {
        doc.root_element()
            .descendants()
            .filter(|n| n.has_tag_name("url"))
            .filter_map(|n| {
                let url_type = n.attribute("type").unwrap_or("website");
                n.text().map(|text| (url_type.to_string(), text.to_string()))
            })
            .collect()
    } else {
        Vec::new()
    }
}

async fn run_command(matches: ArgMatches) -> Result<()> {
    let package_name = matches.get_one::<String>("PACKAGE_NAME").unwrap();
    let show_xml = matches.get_flag("xml");
    
    let workspace_root = std::env::current_dir()?;
    let build_base = workspace_root.join("build");
    let install_base = workspace_root.join("install");
    
    // Use the new shared discovery system
    let config = DiscoveryConfig {
        base_paths: vec![workspace_root.clone()],
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
    
    let packages = discover_packages(&config)?;
    
    // Find the requested package
    let package = packages.iter()
        .find(|pkg| pkg.name == *package_name)
        .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in workspace", package_name))?;
    
    let package_xml_path = package.path.join("package.xml");
    
    if show_xml {
        // Just print the raw XML content
        let xml_content = fs::read_to_string(&package_xml_path)?;
        println!("{}", xml_content);
        return Ok(());
    }
    
    // Parse the XML for detailed information
    let xml_content = fs::read_to_string(&package_xml_path)?;
    
    // Print package header
    println!("{}", "Package Information".bright_cyan().bold());
    println!("{}", "=".repeat(50).bright_black());
    
    // Basic information
    println!("{}: {}", "Name".bright_yellow().bold(), package.name.bright_white().bold());
    println!("{}: {}", "Version".bright_yellow().bold(), package.version.bright_white());
    println!("{}: {}", "Build Type".bright_yellow().bold(), format_build_type(&package.build_type));
    println!("{}: {}", "Path".bright_yellow().bold(), package.path.display().to_string().bright_black());
    
    // Description
    if !package.description.is_empty() {
        println!("{}: {}", "Description".bright_yellow().bold(), package.description.bright_white());
    }
    
    // Maintainers
    if !package.maintainers.is_empty() {
        print_section_header("Maintainers");
        for maintainer in &package.maintainers {
            println!("  • {}", maintainer.bright_white());
        }
    }
    
    // Authors
    let authors = extract_authors(&xml_content);
    if !authors.is_empty() {
        print_section_header("Authors");
        for author in &authors {
            println!("  • {}", author.bright_white());
        }
    }
    
    // Licenses
    let licenses = extract_licenses(&xml_content);
    if !licenses.is_empty() {
        print_section_header("Licenses");
        for license in &licenses {
            println!("  • {}", license.bright_white());
        }
    }
    
    // URLs
    let urls = extract_urls(&xml_content);
    if !urls.is_empty() {
        print_section_header("URLs");
        for (url_type, url) in &urls {
            println!("  • {}: {}", url_type.bright_magenta(), url.bright_white());
        }
    }
    
    // Dependencies
    print_dependencies(&package.build_deps, "Build Dependencies");
    print_dependencies(&package.buildtool_deps, "Build Tool Dependencies");
    print_dependencies(&package.exec_deps, "Execution Dependencies");
    print_dependencies(&package.test_deps, "Test Dependencies");
    
    // Build status
    print_section_header("Build Status");
    let package_build_dir = build_base.join(&package.name);
    let package_install_dir = install_base.join(&package.name);
    
    let build_status = if package_install_dir.exists() {
        "✓ Built and installed".green()
    } else if package_build_dir.exists() {
        "⚠ Partially built (not installed)".yellow()
    } else {
        "✗ Not built".red()
    };
    
    println!("  {}", build_status);
    
    if package_build_dir.exists() {
        println!("  Build directory: {}", package_build_dir.display().to_string().bright_black());
    }
    
    if package_install_dir.exists() {
        println!("  Install directory: {}", package_install_dir.display().to_string().bright_black());
    }
    
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(run_command(matches)) {
        eprintln!("{}: {}", "Error".red().bold(), e);
        std::process::exit(1);
    }
}
