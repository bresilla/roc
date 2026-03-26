use crate::commands::cli::run_async_command;
use crate::shared::package_discovery::{discover_packages, BuildType, DiscoveryConfig};
use anyhow::Result;
use clap::ArgMatches;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn has_isolated_install(package_name: &str, install_base: &PathBuf) -> bool {
    install_base.join(package_name).exists()
}

fn has_merged_install(package_name: &str, install_base: &PathBuf) -> bool {
    install_base.join("share").join(package_name).exists()
        || install_base.join("lib").join(package_name).exists()
}

fn load_workspace_build_state(
    workspace_root: &PathBuf,
    log_base: &PathBuf,
) -> Option<HashMap<String, String>> {
    let state_path = log_base.join("latest").join("workspace_state.txt");
    let contents = fs::read_to_string(state_path).ok()?;
    let mut lines = contents.lines();
    let recorded_root = lines.next()?.strip_prefix("workspace_root=")?;
    if recorded_root != workspace_root.to_string_lossy() {
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

fn load_package_status_file(log_base: &PathBuf, package_name: &str) -> Option<String> {
    let state_path = log_base
        .join("latest")
        .join(package_name)
        .join("status.txt");
    let contents = fs::read_to_string(state_path).ok()?;
    contents.lines().find_map(|line| {
        line.strip_prefix("status=")
            .map(std::string::ToString::to_string)
    })
}

fn format_build_status(
    package_path: &PathBuf,
    build_base: &PathBuf,
    install_base: &PathBuf,
    log_base: &PathBuf,
    workspace_state: Option<&HashMap<String, String>>,
) -> String {
    let package_name = package_path.file_name().unwrap().to_string_lossy();
    let explicit_status = workspace_state
        .and_then(|state| state.get(package_name.as_ref()).cloned())
        .or_else(|| load_package_status_file(log_base, package_name.as_ref()));
    if let Some(status) = explicit_status {
        match status.as_str() {
            "completed" => return "✓ Built".green().to_string(),
            "failed" => return "✗ Failed".red().to_string(),
            "blocked" => return "⚠ Blocked".yellow().to_string(),
            "building" => return "… Building".cyan().to_string(),
            "pending" => return "… Pending".bright_black().to_string(),
            _ => {}
        }
    }

    let build_dir = build_base.join(&*package_name);
    let has_isolated = has_isolated_install(package_name.as_ref(), install_base);
    let has_merged = has_merged_install(package_name.as_ref(), install_base);

    if has_isolated {
        "✓ Built".green().to_string()
    } else if has_merged {
        "✓ Built (merged)".green().to_string()
    } else if build_dir.exists() {
        "⚠ Partial".yellow().to_string()
    } else {
        "✗ Not built".red().to_string()
    }
}

#[cfg(test)]
pub(crate) fn format_build_status_for_tests(
    package_path: &PathBuf,
    build_base: &PathBuf,
    install_base: &PathBuf,
    log_base: &PathBuf,
    workspace_state: Option<&HashMap<String, String>>,
) -> String {
    format_build_status(
        package_path,
        build_base,
        install_base,
        log_base,
        workspace_state,
    )
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

async fn run_command_in_workspace(matches: ArgMatches, workspace_root: PathBuf) -> Result<()> {
    let build_base = workspace_root.join("build");
    let install_base = workspace_root.join("install");
    let log_base = workspace_root.join("log");
    let workspace_state = load_workspace_build_state(&workspace_root, &log_base);

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
        let status = format_build_status(
            &package.path,
            &build_base,
            &install_base,
            &log_base,
            workspace_state.as_ref(),
        );
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

async fn run_command(matches: ArgMatches) -> Result<()> {
    let workspace_root = std::env::current_dir()?;
    run_command_in_workspace(matches, workspace_root).await
}

#[cfg(test)]
pub(crate) async fn run_command_for_tests(
    matches: ArgMatches,
    workspace_root: PathBuf,
) -> Result<()> {
    run_command_in_workspace(matches, workspace_root).await
}

pub fn handle(matches: ArgMatches) {
    run_async_command(run_command(matches));
}

#[cfg(test)]
mod tests {
    use super::{format_build_status, load_workspace_build_state};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn package_path(workspace: &PathBuf, package_name: &str) -> PathBuf {
        workspace.join("src").join(package_name)
    }

    #[test]
    fn build_status_reports_isolated_install() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().to_path_buf();
        let build_base = workspace.join("build");
        let install_base = workspace.join("install");
        let log_base = workspace.join("log");
        let package_name = "demo_pkg";
        let pkg_path = package_path(&workspace, package_name);

        fs::create_dir_all(&pkg_path).unwrap();
        fs::create_dir_all(install_base.join(package_name)).unwrap();

        let status = format_build_status(&pkg_path, &build_base, &install_base, &log_base, None);
        assert!(status.contains("Built"));
        assert!(!status.contains("merged"));
    }

    #[test]
    fn build_status_reports_merged_install() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().to_path_buf();
        let build_base = workspace.join("build");
        let install_base = workspace.join("install");
        let log_base = workspace.join("log");
        let package_name = "demo_pkg";
        let pkg_path = package_path(&workspace, package_name);

        fs::create_dir_all(&pkg_path).unwrap();
        fs::create_dir_all(install_base.join("share").join(package_name)).unwrap();

        let status = format_build_status(&pkg_path, &build_base, &install_base, &log_base, None);
        assert!(status.contains("Built"));
        assert!(status.contains("merged"));
    }

    #[test]
    fn build_status_reports_partial_when_only_build_exists() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().to_path_buf();
        let build_base = workspace.join("build");
        let install_base = workspace.join("install");
        let log_base = workspace.join("log");
        let package_name = "demo_pkg";
        let pkg_path = package_path(&workspace, package_name);

        fs::create_dir_all(&pkg_path).unwrap();
        fs::create_dir_all(build_base.join(package_name)).unwrap();

        let status = format_build_status(&pkg_path, &build_base, &install_base, &log_base, None);
        assert!(status.contains("Partial"));
    }

    #[test]
    fn build_status_prefers_workspace_state_over_install_shape() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().to_path_buf();
        let build_base = workspace.join("build");
        let install_base = workspace.join("install");
        let log_base = workspace.join("log");
        let package_name = "demo_pkg";
        let pkg_path = package_path(&workspace, package_name);

        fs::create_dir_all(&pkg_path).unwrap();
        fs::create_dir_all(install_base.join(package_name)).unwrap();
        fs::create_dir_all(log_base.join("latest")).unwrap();
        fs::write(
            log_base.join("latest/workspace_state.txt"),
            format!(
                "workspace_root={}\npackage=demo_pkg\tstatus=failed\n",
                workspace.display()
            ),
        )
        .unwrap();

        let state = load_workspace_build_state(&workspace, &log_base);
        let status = format_build_status(
            &pkg_path,
            &build_base,
            &install_base,
            &log_base,
            state.as_ref(),
        );
        assert!(status.contains("Failed"));
    }

    #[test]
    fn build_status_falls_back_to_package_status_file_when_workspace_state_missing() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().to_path_buf();
        let build_base = workspace.join("build");
        let install_base = workspace.join("install");
        let log_base = workspace.join("log");
        let package_name = "demo_pkg";
        let pkg_path = package_path(&workspace, package_name);

        fs::create_dir_all(&pkg_path).unwrap();
        fs::create_dir_all(log_base.join("latest").join(package_name)).unwrap();
        fs::write(
            log_base
                .join("latest")
                .join(package_name)
                .join("status.txt"),
            "status=blocked\n",
        )
        .unwrap();

        let status = format_build_status(&pkg_path, &build_base, &install_base, &log_base, None);
        assert!(status.contains("Blocked"));
    }

    #[test]
    fn workspace_build_state_ignores_stale_workspace_root() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().to_path_buf();
        let log_base = workspace.join("log");
        fs::create_dir_all(log_base.join("latest")).unwrap();
        fs::write(
            log_base.join("latest/workspace_state.txt"),
            "workspace_root=/tmp/other\npackage=demo_pkg\tstatus=completed\n",
        )
        .unwrap();

        let state = load_workspace_build_state(&workspace, &log_base);

        assert_eq!(state, Some(HashMap::new()));
    }
}
