use crate::graph::{RclGraphContext, action_operations, interface_operations};
use crate::utils::{get_ros_workspace_paths, is_executable};
use clap::ArgMatches;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{env, fs};
use walkdir::WalkDir;

const TOPIC_SUBCOMMANDS: &[&str] = &[
    "echo", "hz", "info", "list", "kind", "pub", "bw", "find", "delay",
];
const SERVICE_SUBCOMMANDS: &[&str] = &["call", "find", "list", "kind"];
const PARAM_SUBCOMMANDS: &[&str] = &[
    "get", "set", "list", "describe", "remove", "export", "import",
];
const NODE_SUBCOMMANDS: &[&str] = &["list", "info"];
const ACTION_SUBCOMMANDS: &[&str] = &["list", "info", "goal"];
const INTERFACE_SUBCOMMANDS: &[&str] = &["list", "package", "all", "show", "model"];
const BAG_SUBCOMMANDS: &[&str] = &["record", "play", "info", "list"];
const WORK_SUBCOMMANDS: &[&str] = &["build", "create", "info", "list"];
const FRAME_SUBCOMMANDS: &[&str] = &["list", "echo", "info", "pub"];
const DAEMON_SUBCOMMANDS: &[&str] = &["start", "stop", "status"];
const MIDDLEWARE_SUBCOMMANDS: &[&str] = &["get", "set", "list"];

/// Handle internal dynamic completion (_complete)
pub fn handle(matches: ArgMatches) {
    let command = matches
        .get_one::<String>("command")
        .map(|s| s.as_str())
        .unwrap_or_default();
    let sub = matches.get_one::<String>("subcommand").map(|s| s.as_str());
    let subsub = matches
        .get_one::<String>("subsubcommand")
        .map(|s| s.as_str());
    let pos = matches
        .get_one::<String>("position")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let current_args: Vec<String> = matches
        .get_many::<String>("current_args")
        .map(|values| values.cloned().collect())
        .unwrap_or_default();

    for item in complete(command, sub, subsub, pos, &current_args) {
        println!("{}", item);
    }
}

fn complete(
    command: &str,
    subcommand: Option<&str>,
    subsubcommand: Option<&str>,
    position: usize,
    current_args: &[String],
) -> Vec<String> {
    let sub = subcommand.filter(|s| !s.is_empty());
    let subsub = subsubcommand.filter(|s| !s.is_empty());

    match (command, sub, subsub, position) {
        ("launch", None, None, 1) => find_packages_with_launch_files(),
        ("launch", None, None, 2) => {
            let package_filter = current_args.first().map(|s| s.as_str());
            find_launch_files_for_package(package_filter)
                .into_iter()
                .filter_map(|item| item.split_once(':').map(|(_, file)| file.to_string()))
                .collect()
        }
        ("run", None, None, 1) => find_packages(),
        ("run", None, None, 2) => {
            let package_filter = current_args.first().map(|s| s.as_str());
            find_executables_for_package(package_filter)
                .into_iter()
                .filter_map(|item| item.split_once(':').map(|(_, exec)| exec.to_string()))
                .collect()
        }
        ("topic", None, None, 1) => TOPIC_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("topic", Some("echo" | "hz" | "info" | "kind" | "bw" | "delay"), None, 1) => find_topics(),
        ("topic", Some("pub"), None, 1) => find_topics(),
        ("topic", Some("pub"), None, 2) => find_message_types(),
        ("topic", Some("find"), None, 1) => find_message_types(),

        ("service", None, None, 1) => SERVICE_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("service", Some("call"), None, 1) => find_services(),
        ("service", Some("call"), None, 2) => {
            let service_name = current_args.first().map(|s| s.as_str());
            find_service_types_for_name(service_name)
        }
        ("service", Some("find"), None, 1) => find_service_types(),
        ("service", Some("kind"), None, 1) => find_services(),

        ("param", None, None, 1) => PARAM_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("param", Some("get" | "set" | "list" | "describe" | "remove" | "export"), None, 1) => {
            find_nodes()
        }
        ("param", Some("get" | "set" | "describe" | "remove"), None, 2) => find_parameters(),

        ("node", None, None, 1) => NODE_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("node", Some("info"), None, 1) => find_nodes(),

        ("action", None, None, 1) => ACTION_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("action", Some("info" | "goal"), None, 1) => find_actions(),
        ("action", Some("goal"), None, 2) => {
            let action_name = current_args.first().map(|s| s.as_str());
            find_action_types_for_name(action_name)
        }

        ("interface", None, None, 1) => INTERFACE_SUBCOMMANDS
            .iter()
            .map(|s| s.to_string())
            .collect(),
        ("interface", Some("show" | "model"), None, 1) => find_interfaces(),
        ("interface", Some("package"), None, 1) => find_packages(),

        ("bag", None, None, 1) => BAG_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("bag", Some("record"), None, 1) => find_topics(),
        ("bag", Some("play" | "info"), None, 1) => find_bag_files(),

        ("work", None, None, 1) => WORK_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("work", Some("build" | "info"), None, 1) => find_packages(),

        ("frame", None, None, 1) => FRAME_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("frame", Some("echo"), None, 1 | 2) => find_frames(),
        ("frame", Some("info"), None, 1) => find_frames(),

        ("daemon", None, None, 1) => DAEMON_SUBCOMMANDS.iter().map(|s| s.to_string()).collect(),
        ("middleware", None, None, 1) => MIDDLEWARE_SUBCOMMANDS
            .iter()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Scan ROS workspaces for launch files
fn find_launch_files() -> Vec<String> {
    let mut launch_files = HashSet::new();

    if let Ok(distro) = env::var("ROS_DISTRO") {
        let ros_path = PathBuf::from(format!("/opt/ros/{}/share", distro));
        if ros_path.exists() {
            for entry in WalkDir::new(&ros_path)
                .follow_links(true)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.to_string_lossy().contains("/launch/") {
                        if let Some(share_idx) = path.to_string_lossy().find("/share/") {
                            let after_share = &path.to_string_lossy()[share_idx + 7..];
                            if let Some(next_slash) = after_share.find('/') {
                                let package_name = &after_share[..next_slash];
                                if let Some(file_name) = path.file_name() {
                                    launch_files.insert(format!(
                                        "{}:{}",
                                        package_name,
                                        file_name.to_string_lossy()
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for workspace_path in get_ros_workspace_paths() {
        if workspace_path.exists() && !workspace_path.to_string_lossy().contains("/opt/ros/") {
            for entry in WalkDir::new(&workspace_path)
                .follow_links(true)
                .max_depth(6)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.to_string_lossy().contains("/launch/") {
                        let mut current_path = path.parent();
                        while let Some(dir) = current_path {
                            if let Some(pkg_name) = find_package_name(&dir.to_path_buf()) {
                                if let Some(file_name) = path.file_name() {
                                    launch_files.insert(format!(
                                        "{}:{}",
                                        pkg_name,
                                        file_name.to_string_lossy()
                                    ));
                                }
                                break;
                            }
                            current_path = dir.parent();
                        }
                    }
                }
            }
        }
    }

    sorted(launch_files)
}

fn find_packages_with_launch_files() -> Vec<String> {
    let mut packages = HashSet::new();
    for launch_file in find_launch_files() {
        if let Some((package, _)) = launch_file.split_once(':') {
            packages.insert(package.to_string());
        }
    }
    sorted(packages)
}

fn find_launch_files_for_package(package_filter: Option<&str>) -> Vec<String> {
    let all_launch_files = find_launch_files();
    if let Some(package) = package_filter {
        all_launch_files
            .into_iter()
            .filter(|launch_file| {
                launch_file
                    .split_once(':')
                    .map(|(pkg, _)| pkg == package)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        all_launch_files
    }
}

fn find_executables_for_package(package_filter: Option<&str>) -> Vec<String> {
    let all_executables = find_executables();
    if let Some(package) = package_filter {
        all_executables
            .into_iter()
            .filter(|executable| {
                executable
                    .split_once(':')
                    .map(|(pkg, _)| pkg == package)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        all_executables
    }
}

fn find_executables() -> Vec<String> {
    let mut executables = HashSet::new();

    if let Ok(distro) = env::var("ROS_DISTRO") {
        let ros_lib_path = PathBuf::from(format!("/opt/ros/{}/lib", distro));
        let ros_bin_path = PathBuf::from(format!("/opt/ros/{}/bin", distro));

        if ros_lib_path.exists() {
            for entry in WalkDir::new(&ros_lib_path)
                .follow_links(true)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if is_executable(path) {
                        if let Some(lib_idx) = path.to_string_lossy().find("/lib/") {
                            let after_lib = &path.to_string_lossy()[lib_idx + 5..];
                            if let Some(next_slash) = after_lib.find('/') {
                                let package_name = &after_lib[..next_slash];
                                if let Some(exec_name) = path.file_name() {
                                    executables.insert(format!(
                                        "{}:{}",
                                        package_name,
                                        exec_name.to_string_lossy()
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        if ros_bin_path.exists() {
            for entry in WalkDir::new(&ros_bin_path)
                .follow_links(true)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if is_executable(path) {
                        if let Some(exec_name) = path.file_name() {
                            executables.insert(format!("system:{}", exec_name.to_string_lossy()));
                        }
                    }
                }
            }
        }
    }

    for workspace_path in get_ros_workspace_paths() {
        if workspace_path.exists() && !workspace_path.to_string_lossy().contains("/opt/ros/") {
            let install_path = workspace_path.join("install");
            if install_path.exists() {
                for entry in WalkDir::new(&install_path)
                    .follow_links(true)
                    .max_depth(4)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if path.to_string_lossy().contains("/lib/") {
                            if let Some(parent) = path.parent() {
                                if let Some(pkg) = parent.file_name() {
                                    if is_executable(path) {
                                        executables.insert(format!(
                                            "{}:{}",
                                            pkg.to_string_lossy(),
                                            path.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or_default()
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    sorted(executables)
}

fn find_packages() -> Vec<String> {
    let mut packages = HashSet::new();

    if let Ok(distro) = env::var("ROS_DISTRO") {
        let ros_share_path = PathBuf::from(format!("/opt/ros/{}/share", distro));
        if ros_share_path.exists() {
            for entry in WalkDir::new(&ros_share_path)
                .follow_links(true)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() && entry.file_name() == "package.xml" {
                    if let Some(parent) = entry.path().parent() {
                        if let Some(package_name) = parent.file_name().and_then(|n| n.to_str()) {
                            packages.insert(package_name.to_string());
                        }
                    }
                }
            }
        }
    }

    for workspace_path in get_ros_workspace_paths() {
        if workspace_path.exists() && !workspace_path.to_string_lossy().contains("/opt/ros/") {
            for entry in WalkDir::new(&workspace_path)
                .follow_links(true)
                .max_depth(6)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() && entry.file_name() == "package.xml" {
                    if let Some(parent) = entry.path().parent() {
                        if let Some(name) = find_package_name(&parent.to_path_buf()) {
                            packages.insert(name);
                        }
                    }
                }
            }
        }
    }

    sorted(packages)
}

fn find_package_name(dir: &PathBuf) -> Option<String> {
    let mut current = dir.clone();
    for _ in 0..10 {
        let xml = current.join("package.xml");
        if xml.exists() {
            if let Ok(content) = fs::read_to_string(&xml) {
                if let Some(start) = content.find("<name>") {
                    if let Some(end) = content[start + 6..].find("</name>") {
                        return Some(content[start + 6..start + 6 + end].trim().to_string());
                    }
                }
            }
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    None
}

fn find_topics() -> Vec<String> {
    with_graph_context(|context| context.get_topic_names().unwrap_or_default())
}

fn find_message_types() -> Vec<String> {
    let interfaces = interface_operations::list_interfaces(true, false, false).unwrap_or_default();
    sorted(interfaces.into_iter().collect())
}

fn find_services() -> Vec<String> {
    with_graph_context(|context| context.get_service_names().unwrap_or_default())
}

fn find_service_types() -> Vec<String> {
    let interfaces = interface_operations::list_interfaces(false, true, false).unwrap_or_default();
    sorted(interfaces.into_iter().collect())
}

fn find_service_types_for_name(service_name: Option<&str>) -> Vec<String> {
    with_graph_context(|context| {
        let pairs = context.get_service_names_and_types().unwrap_or_default();
        let items: HashSet<String> = match service_name {
            Some(name) => pairs
                .into_iter()
                .filter(|(service, _)| service == name)
                .map(|(_, ty)| ty)
                .collect(),
            None => pairs.into_iter().map(|(_, ty)| ty).collect(),
        };
        sorted(items)
    })
}

fn find_parameters() -> Vec<String> {
    Vec::new()
}

fn find_nodes() -> Vec<String> {
    with_graph_context(|context| context.get_node_names().unwrap_or_default())
}

fn find_actions() -> Vec<String> {
    with_graph_context(|context| action_operations::get_action_names(context).unwrap_or_default())
}

fn find_action_types_for_name(action_name: Option<&str>) -> Vec<String> {
    with_graph_context(|context| {
        let items: HashSet<String> = match action_name {
            Some(name) => action_operations::get_action_type(context, name)
                .ok()
                .flatten()
                .into_iter()
                .collect(),
            None => action_operations::get_action_names(context)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|name| {
                    action_operations::get_action_type(context, &name)
                        .ok()
                        .flatten()
                })
                .collect(),
        };
        sorted(items)
    })
}

fn find_interfaces() -> Vec<String> {
    interface_operations::list_interfaces(false, false, false).unwrap_or_default()
}

fn find_bag_files() -> Vec<String> {
    let mut bags = HashSet::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("metadata.yaml").is_file() {
                bags.insert(path.display().to_string());
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "mcap" | "db3") {
                        bags.insert(path.display().to_string());
                    }
                }
            }
        }
    }
    sorted(bags)
}

fn find_frames() -> Vec<String> {
    Vec::new()
}

fn with_graph_context<F>(resolver: F) -> Vec<String>
where
    F: FnOnce(&RclGraphContext) -> Vec<String>,
{
    RclGraphContext::new()
        .map(|context| resolver(&context))
        .unwrap_or_default()
}

fn sorted(items: HashSet<String>) -> Vec<String> {
    let mut items: Vec<String> = items.into_iter().collect();
    items.sort();
    items
}

#[cfg(test)]
mod tests {
    use super::complete;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn topic_completions_use_kind_subcommand() {
        assert_eq!(
            complete("topic", None, None, 1, &[]),
            strings(&[
                "echo", "hz", "info", "list", "kind", "pub", "bw", "find", "delay"
            ])
        );
    }

    #[test]
    fn service_completions_use_kind_subcommand() {
        assert_eq!(
            complete("service", None, None, 1, &[]),
            strings(&["call", "find", "list", "kind"])
        );
    }

    #[test]
    fn interface_completions_include_all_subcommand() {
        assert_eq!(
            complete("interface", None, None, 1, &[]),
            strings(&["list", "package", "all", "show", "model"])
        );
    }

    #[test]
    fn unknown_position_returns_no_completions() {
        assert!(complete("topic", Some("list"), None, 4, &[]).is_empty());
    }
}
