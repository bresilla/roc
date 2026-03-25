use crate::commands::cli::{required_string, run_async_command};
use crate::utils::{get_ros_workspace_paths, is_executable};
use clap::ArgMatches;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use walkdir::WalkDir;

fn parse_shell_args(
    value: &str,
    field_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    shlex::split(value)
        .ok_or_else(|| format!("Failed to parse {field_name}: unmatched quotes or escapes").into())
}

fn resolve_command_line(
    executable_path: &std::path::Path,
    argv: Option<&str>,
    prefix: Option<&str>,
) -> Result<(OsString, Vec<OsString>), Box<dyn std::error::Error>> {
    let executable_args = match argv {
        Some(argv) => parse_shell_args(argv, "argv")?
            .into_iter()
            .map(OsString::from)
            .collect(),
        None => Vec::new(),
    };

    if let Some(prefix) = prefix {
        let mut prefix_parts = parse_shell_args(prefix, "prefix")?;
        if prefix_parts.is_empty() {
            return Err("Prefix command cannot be empty".into());
        }

        let program = OsString::from(prefix_parts.remove(0));
        let mut args: Vec<OsString> = prefix_parts.into_iter().map(OsString::from).collect();
        args.push(executable_path.as_os_str().to_os_string());
        args.extend(executable_args);
        return Ok((program, args));
    }

    Ok((executable_path.as_os_str().to_os_string(), executable_args))
}

async fn find_executable(
    package_name: &str,
    executable_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_paths = get_ros_workspace_paths();

    for workspace_path in workspace_paths {
        if workspace_path.exists() {
            // Look for executables in install spaces
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

                        // Check if it's in a lib directory (where executables are typically stored)
                        if path.to_string_lossy().contains("/lib/") {
                            if let Some(file_name) = path.file_name() {
                                if file_name == executable_name {
                                    // Check if it's in the right package directory
                                    let path_str = path.to_string_lossy();
                                    if path_str.contains(&format!("/{}/", package_name))
                                        || path_str.contains(&format!("/install/{}/", package_name))
                                    {
                                        // Verify it's executable
                                        if is_executable(path) {
                                            return Ok(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Also look in devel spaces (for catkin workspaces)
            let devel_path = workspace_path.join("devel/lib");
            if devel_path.exists() {
                for entry in WalkDir::new(&devel_path)
                    .follow_links(true)
                    .max_depth(3)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if let Some(parent) = path.parent() {
                            if let Some(package_dir) = parent.file_name() {
                                if package_dir == package_name {
                                    if let Some(file_name) = path.file_name() {
                                        if file_name == executable_name && is_executable(path) {
                                            return Ok(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Look in system installation paths
            let system_lib_path = workspace_path.join("lib").join(package_name);
            if system_lib_path.exists() {
                let executable_path = system_lib_path.join(executable_name);
                if executable_path.exists() && is_executable(&executable_path) {
                    return Ok(executable_path);
                }
            }
        }
    }

    Err(format!(
        "Executable '{}' not found in package '{}'",
        executable_name, package_name
    )
    .into())
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let package_name = required_string(&matches, "package_name")?;
    let executable_name = required_string(&matches, "executable_name")?;

    // Find the actual executable file
    let executable_path = find_executable(package_name, executable_name).await?;

    println!("Running: {}", executable_path.display());

    let (program, args) = resolve_command_line(
        &executable_path,
        matches.get_one::<String>("argv").map(String::as_str),
        matches.get_one::<String>("prefix").map(String::as_str),
    )?;

    // Set up environment for ROS2
    let mut cmd = Command::new(program);

    // Add current environment, making sure ROS environment is preserved
    for (key, value) in env::vars() {
        cmd.env(key, value);
    }
    cmd.args(args);

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let status = cmd.status().await?;

    if !status.success() {
        return Err(format!("Executable failed with exit code: {:?}", status.code()).into());
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    run_async_command(run_command(matches));
}

#[cfg(test)]
mod tests {
    use super::{parse_shell_args, resolve_command_line};
    use std::ffi::OsString;
    use std::path::Path;

    #[test]
    fn parse_shell_args_preserves_quoted_arguments() {
        let args = parse_shell_args("--name 'demo node' \"__ns:=/qa team\"", "argv").unwrap();

        assert_eq!(args, vec!["--name", "demo node", "__ns:=/qa team"]);
    }

    #[test]
    fn parse_shell_args_rejects_unmatched_quotes() {
        let err = parse_shell_args("--name 'broken", "argv").unwrap_err();

        assert!(err.to_string().contains("Failed to parse argv"));
    }

    #[test]
    fn resolve_command_line_applies_prefix_and_preserves_nested_quotes() {
        let executable = Path::new("/tmp/demo executable");
        let (program, args) = resolve_command_line(
            executable,
            Some("--ros-args -r '__ns:=/demo node'"),
            Some("gdb -ex 'run --verbose' --args"),
        )
        .unwrap();

        assert_eq!(program, OsString::from("gdb"));
        assert_eq!(
            args,
            vec![
                OsString::from("-ex"),
                OsString::from("run --verbose"),
                OsString::from("--args"),
                OsString::from("/tmp/demo executable"),
                OsString::from("--ros-args"),
                OsString::from("-r"),
                OsString::from("__ns:=/demo node"),
            ]
        );
    }

    #[test]
    fn resolve_command_line_without_prefix_preserves_argument_spacing() {
        let executable = Path::new("/tmp/talker");
        let (program, args) =
            resolve_command_line(executable, Some("--label 'hello world'"), None).unwrap();

        assert_eq!(program, OsString::from("/tmp/talker"));
        assert_eq!(
            args,
            vec![OsString::from("--label"), OsString::from("hello world")]
        );
    }
}
