use crate::commands::cli::{required_string, run_async_command};
use crate::shared::package_discovery::{discover_packages, BuildType, DiscoveryConfig};
use crate::ui::{blocks, table};
use anyhow::Result;
use clap::ArgMatches;
use colored::*;
use roxmltree::Document;
use std::fs;

fn has_isolated_install(package_name: &str, install_base: &std::path::Path) -> bool {
    install_base.join(package_name).exists()
}

fn has_merged_install(package_name: &str, install_base: &std::path::Path) -> bool {
    install_base.join("share").join(package_name).exists()
        || install_base.join("lib").join(package_name).exists()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallLayout {
    None,
    Isolated,
    Merged,
}

fn detect_install_layout(package_name: &str, install_base: &std::path::Path) -> InstallLayout {
    if has_isolated_install(package_name, install_base) {
        InstallLayout::Isolated
    } else if has_merged_install(package_name, install_base) {
        InstallLayout::Merged
    } else {
        InstallLayout::None
    }
}

#[cfg(test)]
pub(crate) fn detect_install_layout_for_tests(
    package_name: &str,
    install_base: &std::path::Path,
) -> String {
    match detect_install_layout(package_name, install_base) {
        InstallLayout::None => "none".to_string(),
        InstallLayout::Isolated => "isolated".to_string(),
        InstallLayout::Merged => "merged".to_string(),
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

fn print_dependencies(deps: &[String], title: &str) {
    if !deps.is_empty() {
        println!();
        blocks::print_section(title);
        let rows = deps
            .iter()
            .map(|dep| vec![dep.bright_white().to_string()])
            .collect();
        table::print_table(&["Dependency"], rows);
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
                n.text()
                    .map(|text| (url_type.to_string(), text.to_string()))
            })
            .collect()
    } else {
        Vec::new()
    }
}

async fn run_command_in_workspace(
    matches: ArgMatches,
    workspace_root: std::path::PathBuf,
) -> Result<()> {
    let package_name = required_string(&matches, "PACKAGE_NAME")
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let show_xml = matches.get_flag("xml");

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
    let package = packages
        .iter()
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

    blocks::print_section("Package");
    blocks::print_field("Name", package.name.bright_white().bold());
    blocks::print_field("Version", package.version.bright_white());
    blocks::print_field("Build Type", format_build_type(&package.build_type));
    blocks::print_field("Path", package.path.display().to_string().bright_black());

    // Description
    if !package.description.is_empty() {
        blocks::print_field("Description", package.description.bright_white());
    }

    // Maintainers
    if !package.maintainers.is_empty() {
        println!();
        blocks::print_section("Maintainers");
        let rows = package
            .maintainers
            .iter()
            .map(|maintainer| vec![maintainer.bright_white().to_string()])
            .collect();
        table::print_table(&["Maintainer"], rows);
    }

    // Authors
    let authors = extract_authors(&xml_content);
    if !authors.is_empty() {
        println!();
        blocks::print_section("Authors");
        let rows = authors
            .iter()
            .map(|author| vec![author.bright_white().to_string()])
            .collect();
        table::print_table(&["Author"], rows);
    }

    // Licenses
    let licenses = extract_licenses(&xml_content);
    if !licenses.is_empty() {
        println!();
        blocks::print_section("Licenses");
        let rows = licenses
            .iter()
            .map(|license| vec![license.bright_white().to_string()])
            .collect();
        table::print_table(&["License"], rows);
    }

    // URLs
    let urls = extract_urls(&xml_content);
    if !urls.is_empty() {
        println!();
        blocks::print_section("URLs");
        let rows = urls
            .iter()
            .map(|(url_type, url)| {
                vec![
                    url_type.bright_magenta().to_string(),
                    url.bright_white().to_string(),
                ]
            })
            .collect();
        table::print_table(&["Type", "URL"], rows);
    }

    // Dependencies
    print_dependencies(&package.depend_deps, "Generic Dependencies");
    print_dependencies(&package.build_deps, "Build Dependencies");
    print_dependencies(&package.buildtool_deps, "Build Tool Dependencies");
    print_dependencies(&package.build_export_deps, "Build Export Dependencies");
    print_dependencies(&package.exec_deps, "Execution Dependencies");
    print_dependencies(
        &package.runtime_deps(),
        "Derived Runtime/Setup Dependencies",
    );
    print_dependencies(&package.test_deps, "Test Dependencies");

    // Build status
    println!();
    blocks::print_section("Build Status");
    let package_build_dir = build_base.join(&package.name);
    let package_install_dir = install_base.join(&package.name);
    let install_layout = detect_install_layout(&package.name, &install_base);

    let build_status = match install_layout {
        InstallLayout::Isolated => "✓ Built and installed (isolated)".green(),
        InstallLayout::Merged => "✓ Built and installed (merged)".green(),
        InstallLayout::None => {
            if package_build_dir.exists() {
                "⚠ Partially built (not installed)".yellow()
            } else {
                "✗ Not built".red()
            }
        }
    };

    blocks::print_field("Status", build_status);

    if package_build_dir.exists() {
        blocks::print_field(
            "Build directory",
            package_build_dir.display().to_string().bright_black(),
        );
    }

    match install_layout {
        InstallLayout::Isolated => {
            blocks::print_field(
                "Install directory",
                package_install_dir.display().to_string().bright_black(),
            );
        }
        InstallLayout::Merged => {
            blocks::print_field(
                "Install directory",
                install_base.display().to_string().bright_black(),
            );
        }
        InstallLayout::None => {}
    }

    Ok(())
}

async fn run_command(matches: ArgMatches) -> Result<()> {
    let workspace_root = std::env::current_dir()?;
    run_command_in_workspace(matches, workspace_root).await
}

#[cfg(test)]
pub(crate) async fn run_command_for_tests(
    matches: ArgMatches,
    workspace_root: std::path::PathBuf,
) -> Result<()> {
    run_command_in_workspace(matches, workspace_root).await
}

pub fn handle(matches: ArgMatches) {
    run_async_command(run_command(matches));
}

#[cfg(test)]
mod tests {
    use super::{detect_install_layout, has_isolated_install, has_merged_install, InstallLayout};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_isolated_install_layout() {
        let temp = tempdir().unwrap();
        let install_base = temp.path().join("install");
        let package_name = "demo_pkg";

        fs::create_dir_all(install_base.join(package_name)).unwrap();

        assert!(has_isolated_install(package_name, &install_base));
        assert!(!has_merged_install(package_name, &install_base));
    }

    #[test]
    fn detects_merged_install_layout() {
        let temp = tempdir().unwrap();
        let install_base = temp.path().join("install");
        let package_name = "demo_pkg";

        fs::create_dir_all(install_base.join("share").join(package_name)).unwrap();

        assert!(!has_isolated_install(package_name, &install_base));
        assert!(has_merged_install(package_name, &install_base));
    }

    #[test]
    fn detect_install_layout_returns_merged() {
        let temp = tempdir().unwrap();
        let install_base = temp.path().join("install");
        let package_name = "demo_pkg";

        fs::create_dir_all(install_base.join("share").join(package_name)).unwrap();

        assert_eq!(
            detect_install_layout(package_name, &install_base),
            InstallLayout::Merged
        );
    }
}
