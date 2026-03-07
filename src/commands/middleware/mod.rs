use clap::ArgMatches;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::cli::print_error_and_exit;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("list", args)) => {
            list::handle(args.clone());
        }
        Some(("set", args)) => {
            set::handle(args.clone());
        }
        Some(("get", args)) => {
            get::handle(args.clone());
        }
        _ => print_error_and_exit("No middleware subcommand selected"),
    }
}

pub mod get;
pub mod list;
pub mod set;

pub(crate) fn current_implementation() -> Option<String> {
    env::var("RMW_IMPLEMENTATION")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn discover_implementations() -> Vec<String> {
    let mut implementations = BTreeSet::new();

    for lib_dir in candidate_library_dirs() {
        let entries = match fs::read_dir(&lib_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };

            if let Some(implementation) = parse_rmw_library_name(file_name) {
                implementations.insert(implementation);
            }
        }
    }

    implementations.into_iter().collect()
}

pub(crate) fn export_command(implementation: &str) -> String {
    format!("export RMW_IMPLEMENTATION={implementation}")
}

fn candidate_library_dirs() -> Vec<PathBuf> {
    let mut dirs = BTreeSet::new();

    for key in ["AMENT_PREFIX_PATH", "COLCON_PREFIX_PATH"] {
        if let Ok(prefixes) = env::var(key) {
            for prefix in env::split_paths(&prefixes) {
                dirs.insert(prefix.join("lib"));
            }
        }
    }

    if let Ok(distro) = env::var("ROS_DISTRO") {
        dirs.insert(Path::new("/opt/ros").join(distro).join("lib"));
    }

    dirs.into_iter().collect()
}

fn parse_rmw_library_name(file_name: &str) -> Option<String> {
    let stem = file_name
        .split(".so")
        .next()
        .or_else(|| file_name.split(".dylib").next())
        .or_else(|| file_name.split(".dll").next())
        .unwrap_or(file_name);

    let stem = stem.strip_prefix("lib").unwrap_or(stem);
    if !stem.starts_with("rmw_") {
        return None;
    }

    if stem == "rmw_implementation"
        || stem.contains("_shared_")
        || stem.contains("_common")
        || stem.contains("serialization")
        || stem.contains("test")
    {
        return None;
    }

    Some(stem.to_string())
}

#[cfg(test)]
mod tests {
    use super::{export_command, parse_rmw_library_name};

    #[test]
    fn parses_rmw_implementation_library_names() {
        assert_eq!(
            parse_rmw_library_name("librmw_fastrtps_cpp.so"),
            Some("rmw_fastrtps_cpp".to_string())
        );
        assert_eq!(
            parse_rmw_library_name("librmw_cyclonedds_cpp.so.1"),
            Some("rmw_cyclonedds_cpp".to_string())
        );
    }

    #[test]
    fn ignores_non_implementation_rmw_libraries() {
        assert_eq!(parse_rmw_library_name("librmw_dds_common.so"), None);
        assert_eq!(
            parse_rmw_library_name("librmw_fastrtps_shared_cpp.so"),
            None
        );
        assert_eq!(parse_rmw_library_name("libnot_rmw.so"), None);
    }

    #[test]
    fn renders_shell_export_command() {
        assert_eq!(
            export_command("rmw_cyclonedds_cpp"),
            "export RMW_IMPLEMENTATION=rmw_cyclonedds_cpp"
        );
    }
}
