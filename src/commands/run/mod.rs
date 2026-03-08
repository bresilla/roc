use crate::commands::cli::{print_error_and_exit, required_string};
use crate::ui::{
    blocks,
    output::{self, OutputMode},
};
use crate::utils::{get_ros_workspace_paths, is_executable};
use clap::ArgMatches;
use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use walkdir::WalkDir;

#[derive(Clone)]
struct RunConfig {
    package_name: String,
    executable_name: String,
    argv: String,
    prefix: Option<String>,
    output_mode: OutputMode,
}

struct RunOutcome {
    resolved_path: PathBuf,
    command_preview: String,
    success: bool,
    exit_code: Option<i32>,
    elapsed_secs: f64,
    stdout: String,
    stderr: String,
}

async fn find_executable(
    package_name: &str,
    executable_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_paths = get_ros_workspace_paths();

    for workspace_path in workspace_paths {
        if workspace_path.exists() {
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
                            if let Some(file_name) = path.file_name() {
                                if file_name == executable_name {
                                    let path_str = path.to_string_lossy();
                                    if path_str.contains(&format!("/{}/", package_name))
                                        || path_str.contains(&format!("/install/{}/", package_name))
                                    {
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

fn config_from_matches(matches: &ArgMatches) -> Result<RunConfig, Box<dyn std::error::Error>> {
    Ok(RunConfig {
        package_name: required_string(matches, "package_name")?.to_string(),
        executable_name: required_string(matches, "executable_name")?.to_string(),
        argv: matches
            .get_one::<String>("argv")
            .map(|value| value.to_string())
            .unwrap_or_default(),
        prefix: matches.get_one::<String>("prefix").cloned(),
        output_mode: OutputMode::from_matches(matches),
    })
}

fn print_multiline_output(label: &str, content: &str) {
    if !content.trim().is_empty() {
        output::print_plain_multiline_field(label, content);
    }
}

fn print_run_error(config: Option<&RunConfig>, error: &str, exit_code: Option<i32>) -> ! {
    let output_mode = config
        .map(|config| config.output_mode)
        .unwrap_or(OutputMode::Human);

    match output_mode {
        OutputMode::Human => print_error_and_exit(error),
        OutputMode::Plain => {
            output::print_plain_section("run");
            if let Some(config) = config {
                output::print_plain_field("package", &config.package_name);
                output::print_plain_field("executable", &config.executable_name);
                if !config.argv.is_empty() {
                    output::print_plain_field("arguments", &config.argv);
                }
                if let Some(prefix) = &config.prefix {
                    output::print_plain_field("prefix", prefix);
                }
            }
            if let Some(exit_code) = exit_code {
                output::print_plain_field("exit_code", exit_code);
            }
            output::print_plain_field("status", "error");
            output::print_plain_field("error", error);
            std::process::exit(1);
        }
        OutputMode::Json => {
            let payload = config.map(|config| {
                json!({
                    "command": "run",
                    "package": config.package_name,
                    "executable": config.executable_name,
                    "arguments": config.argv,
                    "prefix": config.prefix,
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            });
            let _ = output::print_json(&payload.unwrap_or_else(|| {
                json!({
                    "command": "run",
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            }));
            std::process::exit(1);
        }
    }
}

async fn execute_run(config: &RunConfig) -> Result<RunOutcome, Box<dyn std::error::Error>> {
    if config.output_mode == OutputMode::Human {
        blocks::print_section("Run");
        blocks::print_field("Package", &config.package_name);
        blocks::print_field("Executable", &config.executable_name);
        if !config.argv.is_empty() {
            blocks::print_field("Arguments", &config.argv);
        }
        if let Some(prefix) = &config.prefix {
            blocks::print_field("Prefix", prefix);
        }
    }

    let executable_path = find_executable(&config.package_name, &config.executable_name).await?;
    let argv_parts = config
        .argv
        .split_whitespace()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

    let mut cmd = if let Some(prefix) = &config.prefix {
        let prefix_parts: Vec<&str> = prefix.split_whitespace().collect();
        if prefix_parts.is_empty() {
            Command::new(&executable_path)
        } else {
            let mut prefixed_cmd = Command::new(prefix_parts[0]);
            for part in &prefix_parts[1..] {
                prefixed_cmd.arg(part);
            }
            prefixed_cmd.arg(&executable_path);
            prefixed_cmd
        }
    } else {
        Command::new(&executable_path)
    };

    for (key, value) in env::vars() {
        cmd.env(key, value);
    }
    for arg in &argv_parts {
        cmd.arg(arg);
    }

    let command_preview = format!("{cmd:?}");

    if config.output_mode == OutputMode::Human {
        blocks::print_field("Resolved Path", executable_path.display());
        println!();
        blocks::print_field("Command", &command_preview);
        blocks::print_note("Child process stdio is attached to the terminal");

        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());

        let started_at = Instant::now();
        let status = cmd.status().await?;
        return Ok(RunOutcome {
            resolved_path: executable_path,
            command_preview,
            success: status.success(),
            exit_code: status.code(),
            elapsed_secs: started_at.elapsed().as_secs_f64(),
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let started_at = Instant::now();
    let output = cmd.output().await?;
    Ok(RunOutcome {
        resolved_path: executable_path,
        command_preview,
        success: output.status.success(),
        exit_code: output.status.code(),
        elapsed_secs: started_at.elapsed().as_secs_f64(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn render_run_result(
    config: &RunConfig,
    outcome: &RunOutcome,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.output_mode {
        OutputMode::Human => {
            if !outcome.success {
                return Err(format!(
                    "Executable failed with exit code: {:?}",
                    outcome.exit_code
                )
                .into());
            }

            println!();
            blocks::print_section("Run Summary");
            blocks::print_field("Package", &config.package_name);
            blocks::print_field("Executable", &config.executable_name);
            blocks::print_field("Elapsed", format!("{:.2}s", outcome.elapsed_secs));
            blocks::print_success("Process exited successfully");
        }
        OutputMode::Plain => {
            output::print_plain_section("run");
            output::print_plain_field("package", &config.package_name);
            output::print_plain_field("executable", &config.executable_name);
            if !config.argv.is_empty() {
                output::print_plain_field("arguments", &config.argv);
            }
            if let Some(prefix) = &config.prefix {
                output::print_plain_field("prefix", prefix);
            }
            output::print_plain_field("resolved_path", outcome.resolved_path.display());
            output::print_plain_field("command", &outcome.command_preview);
            output::print_plain_field("exit_code", outcome.exit_code.unwrap_or_default());
            output::print_plain_field("elapsed_secs", format!("{:.3}", outcome.elapsed_secs));
            output::print_plain_field("status", if outcome.success { "ok" } else { "error" });
            print_multiline_output("stdout", &outcome.stdout);
            print_multiline_output("stderr", &outcome.stderr);
        }
        OutputMode::Json => {
            output::print_json(&json!({
                "command": "run",
                "package": config.package_name,
                "executable": config.executable_name,
                "arguments": config.argv,
                "prefix": config.prefix,
                "resolved_path": outcome.resolved_path,
                "spawned_command": outcome.command_preview,
                "exit_code": outcome.exit_code,
                "elapsed_secs": outcome.elapsed_secs,
                "status": if outcome.success { "ok" } else { "error" },
                "stdout": outcome.stdout,
                "stderr": outcome.stderr
            }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let config = match config_from_matches(&matches) {
        Ok(config) => config,
        Err(error) => print_run_error(None, &error.to_string(), None),
    };

    let runtime = tokio::runtime::Runtime::new()
        .unwrap_or_else(|error| print_error_and_exit(format!("Failed to create async runtime: {error}")));

    let outcome = match runtime.block_on(execute_run(&config)) {
        Ok(outcome) => outcome,
        Err(error) => print_run_error(Some(&config), &error.to_string(), None),
    };

    if let Err(error) = render_run_result(&config, &outcome) {
        print_run_error(Some(&config), &error.to_string(), outcome.exit_code);
    }

    if !outcome.success {
        std::process::exit(1);
    }
}
