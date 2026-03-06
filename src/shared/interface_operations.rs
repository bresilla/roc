use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::shared::ros_interface_parser;

fn ros_share_dirs() -> Vec<PathBuf> {
    // Prefer AMENT_PREFIX_PATH like the rest of ROS tooling.
    let mut dirs = Vec::new();
    if let Ok(prefixes) = std::env::var("AMENT_PREFIX_PATH") {
        for prefix in prefixes.split(':') {
            if prefix.is_empty() {
                continue;
            }
            dirs.push(PathBuf::from(prefix).join("share"));
        }
    }
    dirs
}

fn list_interface_dirs(pkg_share: &Path) -> Vec<(&'static str, PathBuf)> {
    vec![
        ("msg", pkg_share.join("msg")),
        ("srv", pkg_share.join("srv")),
        ("action", pkg_share.join("action")),
    ]
}

fn has_ros_package_manifest(pkg_share: &Path) -> bool {
    // In installed ROS packages, `share/<pkg>/package.xml` exists.
    pkg_share.join("package.xml").is_file()
}

fn interface_type_from_file(pkg: &str, kind: &str, path: &Path) -> Option<String> {
    let name = path.file_stem()?.to_str()?;
    Some(format!("{}/{}/{}", pkg, kind, name))
}

fn collect_installed_packages() -> BTreeMap<String, PathBuf> {
    let mut pkgs = BTreeMap::new();
    for share_dir in ros_share_dirs() {
        if !share_dir.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&share_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if has_ros_package_manifest(&path) {
                    // First match wins; AMENT_PREFIX_PATH order matters.
                    pkgs.entry(name.to_string()).or_insert(path);
                }
            }
        }
    }
    pkgs
}

pub fn list_interfaces(
    only_msgs: bool,
    only_srvs: bool,
    only_actions: bool,
) -> Result<Vec<String>> {
    let kinds: Vec<(&str, &str)> = match (only_msgs, only_srvs, only_actions) {
        (true, false, false) => vec![("msg", "msg")],
        (false, true, false) => vec![("srv", "srv")],
        (false, false, true) => vec![("action", "action")],
        (false, false, false) => vec![("msg", "msg"), ("srv", "srv"), ("action", "action")],
        _ => {
            return Err(anyhow!(
                "Only one of --messages/--services/--actions can be used"
            ));
        }
    };

    let pkgs = collect_installed_packages();
    let mut out = BTreeSet::new();

    for (pkg, pkg_share) in pkgs {
        for (dir_kind, kind_name) in &kinds {
            let dir = pkg_share.join(dir_kind);
            if !dir.is_dir() {
                continue;
            }
            let ext = *dir_kind;
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    if path.extension().and_then(|e| e.to_str()) != Some(ext) {
                        continue;
                    }
                    if let Some(t) = interface_type_from_file(&pkg, kind_name, &path) {
                        out.insert(t);
                    }
                }
            }
        }
    }

    Ok(out.into_iter().collect())
}

pub fn list_packages_with_interfaces(
    only_msgs: bool,
    only_srvs: bool,
    only_actions: bool,
) -> Result<Vec<String>> {
    let pkgs = collect_installed_packages();
    let mut out = BTreeSet::new();

    for (pkg, pkg_share) in pkgs {
        let dirs = list_interface_dirs(&pkg_share);

        let mut has_any = false;
        for (kind, dir) in dirs {
            if !dir.is_dir() {
                continue;
            }
            let enabled = match kind {
                "msg" => !only_srvs && !only_actions,
                "srv" => !only_msgs && !only_actions,
                "action" => !only_msgs && !only_srvs,
                _ => false,
            };
            if !enabled {
                continue;
            }

            if let Ok(mut it) = fs::read_dir(&dir) {
                if it.any(|e| e.ok().is_some()) {
                    has_any = true;
                    break;
                }
            }
        }

        if has_any {
            out.insert(pkg);
        }
    }

    Ok(out.into_iter().collect())
}

pub fn list_interfaces_in_package(package: &str) -> Result<Vec<String>> {
    let pkgs = collect_installed_packages();
    let Some(pkg_share) = pkgs.get(package) else {
        return Err(anyhow!(
            "Package '{}' not found in AMENT_PREFIX_PATH",
            package
        ));
    };

    let mut out = BTreeSet::new();
    for (kind, dir) in list_interface_dirs(pkg_share) {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some(kind) {
                    continue;
                }
                if let Some(t) = interface_type_from_file(package, kind, &path) {
                    out.insert(t);
                }
            }
        }
    }
    Ok(out.into_iter().collect())
}

fn find_interface_file(type_name: &str) -> Result<PathBuf> {
    if type_name.trim() == "-" {
        return Err(anyhow!("Reading type from stdin is not supported yet"));
    }

    // Accept both `pkg/msg/Type` and `pkg/msg/Type.msg` (same for srv/action).
    let re = Regex::new(
        r"^(?P<pkg>[^/]+)/(?P<kind>msg|srv|action)/(?P<name>[^/]+?)(?:\.(?P<ext>msg|srv|action))?$",
    )
    .map_err(|e| anyhow!("Failed to compile interface type matcher: {}", e))?;
    let caps = re.captures(type_name).ok_or_else(|| {
        anyhow!(
            "Invalid interface type '{}'. Expected pkg/msg/Type",
            type_name
        )
    })?;

    let pkg = caps
        .name("pkg")
        .map(|value| value.as_str())
        .ok_or_else(|| anyhow!("Invalid interface type '{}': missing package", type_name))?;
    let kind = caps
        .name("kind")
        .map(|value| value.as_str())
        .ok_or_else(|| anyhow!("Invalid interface type '{}': missing kind", type_name))?;
    let name = caps
        .name("name")
        .map(|value| value.as_str())
        .ok_or_else(|| anyhow!("Invalid interface type '{}': missing name", type_name))?;

    let pkgs = collect_installed_packages();
    let Some(pkg_share) = pkgs.get(pkg) else {
        return Err(anyhow!("Package '{}' not found in AMENT_PREFIX_PATH", pkg));
    };

    let path = pkg_share.join(kind).join(format!("{}.{}", name, kind));
    if !path.is_file() {
        return Err(anyhow!(
            "Interface definition not found: {}",
            path.display()
        ));
    }
    Ok(path)
}

pub fn show_interface(type_name: &str, no_comments: bool, all_comments: bool) -> Result<String> {
    if no_comments && all_comments {
        return Err(anyhow!("--no-comments conflicts with --all-comments"));
    }

    let path = find_interface_file(type_name)?;
    let text = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read {}: {}", path.display(), e))?;

    if all_comments {
        // Full recursive show isn't implemented yet; mimic ROS2 CLI by just showing this file.
        return Ok(text);
    }
    if !no_comments {
        return Ok(text);
    }

    // Strip comments + blank lines (approx to ros2 interface show --no-comments).
    let mut out = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    Ok(out)
}

pub fn model_interface(_type_name: &str, no_quotes: bool) -> Result<String> {
    let type_name = _type_name;
    let path = find_interface_file(type_name)?;

    let text = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read {}: {}", path.display(), e))?;

    let parts: Vec<&str> = type_name.split('/').collect();
    if parts.len() != 3 {
        return Err(anyhow!(
            "Invalid interface type '{}'. Expected pkg/msg/Type",
            type_name
        ));
    }
    let pkg = parts[0];

    let resolver = |t: &str| -> Result<ros_interface_parser::InterfaceSpec> {
        let p = find_interface_file(t)?;
        let content =
            fs::read_to_string(&p).map_err(|e| anyhow!("Failed to read {}: {}", p.display(), e))?;
        ros_interface_parser::parse_interface(t, pkg, &content)
    };

    let spec = ros_interface_parser::parse_interface(type_name, pkg, &text)?;
    let yaml_val = match spec {
        ros_interface_parser::InterfaceSpec::Msg(m) => {
            ros_interface_parser::default_yaml_for_message(pkg, &m, &resolver)?
        }
        ros_interface_parser::InterfaceSpec::Srv(s) => {
            ros_interface_parser::default_yaml_for_message(pkg, &s.request, &resolver)?
        }
        ros_interface_parser::InterfaceSpec::Action(a) => {
            ros_interface_parser::default_yaml_for_message(pkg, &a.goal, &resolver)?
        }
    };

    let yaml = serde_yaml::to_string(&yaml_val)?;
    let yaml = yaml.trim_end_matches('\n');

    if no_quotes {
        Ok(yaml.to_string())
    } else {
        Ok(format!("\"{}\"", yaml.replace('"', "\\\"")))
    }
}
