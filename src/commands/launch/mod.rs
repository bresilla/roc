use crate::commands::cli::{print_error_and_exit, required_string};
use crate::ui::{
    blocks,
    output::{self, OutputMode},
};
use crate::utils::get_ros_workspace_paths;
use clap::ArgMatches;
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use walkdir::WalkDir;

#[derive(Clone)]
struct LaunchConfig {
    package_name: String,
    launch_file_name: String,
    launch_arguments: String,
    noninteractive: bool,
    debug: bool,
    print_only: bool,
    show_args: bool,
    show_all: bool,
    launch_prefix: Option<String>,
    launch_prefix_filter: Option<String>,
    output_mode: OutputMode,
}

struct LaunchOutcome {
    resolved_path: PathBuf,
    command_preview: String,
    success: bool,
    exit_code: Option<i32>,
    elapsed_secs: f64,
    stdout: String,
    stderr: String,
}

async fn find_launch_file(
    package_name: &str,
    launch_file_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_paths = get_ros_workspace_paths();

    for workspace_path in workspace_paths {
        if workspace_path.exists() {
            let search_paths = vec![
                workspace_path.join("src"),
                workspace_path.join("install"),
                workspace_path.join("share"),
            ];

            for search_path in search_paths {
                if search_path.exists() {
                    for entry in WalkDir::new(&search_path)
                        .follow_links(true)
                        .max_depth(6)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            let path = entry.path();

                            if let Some(file_name) = path.file_name() {
                                let file_name_str = file_name.to_string_lossy();
                                let matches_name = file_name_str == launch_file_name
                                    || path.file_stem().map(|s| s.to_string_lossy())
                                        == Some(launch_file_name.into());

                                if matches_name {
                                    let path_str = path.to_string_lossy();
                                    if path_str.contains(&format!("/{}/", package_name))
                                        || path_str.contains(&format!("/{}/launch/", package_name))
                                        || path_str.contains(&format!(
                                            "/{}/share/{}/",
                                            package_name, package_name
                                        ))
                                    {
                                        if path_str.contains("/launch/") {
                                            return Ok(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err(format!(
        "Launch file '{}' not found in package '{}'",
        launch_file_name, package_name
    )
    .into())
}

fn config_from_matches(matches: &ArgMatches) -> Result<LaunchConfig, Box<dyn std::error::Error>> {
    Ok(LaunchConfig {
        package_name: required_string(matches, "package_name")?.to_string(),
        launch_file_name: required_string(matches, "launch_file_name")?.to_string(),
        launch_arguments: matches
            .get_one::<String>("launch_arguments")
            .map(|value| value.to_string())
            .unwrap_or_default(),
        noninteractive: matches.get_flag("noninteractive"),
        debug: matches.get_flag("debug"),
        print_only: matches.get_flag("print"),
        show_args: matches.get_flag("show_args"),
        show_all: matches.get_flag("show_all"),
        launch_prefix: matches.get_one::<String>("launch_prefix").cloned(),
        launch_prefix_filter: matches.get_one::<String>("launch_prefix_filter").cloned(),
        output_mode: OutputMode::from_matches(matches),
    })
}

fn print_multiline_output(label: &str, content: &str) {
    if !content.trim().is_empty() {
        output::print_plain_multiline_field(label, content);
    }
}

fn print_launch_error(config: Option<&LaunchConfig>, error: &str, exit_code: Option<i32>) -> ! {
    let output_mode = config
        .map(|config| config.output_mode)
        .unwrap_or(OutputMode::Human);

    match output_mode {
        OutputMode::Human => print_error_and_exit(error),
        OutputMode::Plain => {
            output::print_plain_section("launch");
            if let Some(config) = config {
                output::print_plain_field("package", &config.package_name);
                output::print_plain_field("launch_file", &config.launch_file_name);
                if !config.launch_arguments.is_empty() {
                    output::print_plain_field("arguments", &config.launch_arguments);
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
                    "command": "launch",
                    "package": config.package_name,
                    "launch_file": config.launch_file_name,
                    "arguments": config.launch_arguments,
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            });
            let _ = output::print_json(&payload.unwrap_or_else(|| {
                json!({
                    "command": "launch",
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            }));
            std::process::exit(1);
        }
    }
}

async fn execute_launch(config: &LaunchConfig) -> Result<LaunchOutcome, Box<dyn std::error::Error>> {
    if config.output_mode == OutputMode::Human {
        blocks::print_section("Launch");
        blocks::print_field("Package", &config.package_name);
        blocks::print_field("Launch File", &config.launch_file_name);
        if !config.launch_arguments.is_empty() {
            blocks::print_field("Arguments", &config.launch_arguments);
        }
        if config.noninteractive {
            blocks::print_field("Noninteractive", "yes");
        }
        if config.debug {
            blocks::print_field("Debug", "enabled");
        }
        if config.print_only {
            blocks::print_field("Print Only", "yes");
        }
        if config.show_args {
            blocks::print_field("Show Args", "yes");
        }
        if config.show_all {
            blocks::print_field("Show All Output", "yes");
        }
        if let Some(prefix) = &config.launch_prefix {
            blocks::print_field("Launch Prefix", prefix);
        }
        if let Some(filter) = &config.launch_prefix_filter {
            blocks::print_field("Prefix Filter", filter);
        }
    }

    let launch_file_path = find_launch_file(&config.package_name, &config.launch_file_name).await?;
    let mut cmd = Command::new("ros2");
    cmd.args(["launch", &config.package_name, &config.launch_file_name]);

    for arg in config.launch_arguments.split_whitespace() {
        cmd.arg(arg);
    }

    if config.noninteractive {
        cmd.arg("--noninteractive");
    }
    if config.debug {
        cmd.arg("--debug");
    }
    if config.print_only {
        cmd.arg("--print");
    }
    if config.show_args {
        cmd.arg("--show-args");
    }
    if config.show_all {
        cmd.arg("--show-all-subprocesses-output");
    }
    if let Some(launch_prefix) = &config.launch_prefix {
        cmd.args(["--launch-prefix", launch_prefix]);
    }
    if let Some(launch_prefix_filter) = &config.launch_prefix_filter {
        cmd.args(["--launch-prefix-filter", launch_prefix_filter]);
    }

    let command_preview = format!("{cmd:?}");

    if config.output_mode == OutputMode::Human {
        blocks::print_field("Resolved Path", launch_file_path.display());
        println!();
        blocks::print_field("Command", &command_preview);
        blocks::print_note("Child process stdio is attached to the terminal");

        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());

        let started_at = Instant::now();
        let status = cmd.status().await?;
        return Ok(LaunchOutcome {
            resolved_path: launch_file_path,
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
    Ok(LaunchOutcome {
        resolved_path: launch_file_path,
        command_preview,
        success: output.status.success(),
        exit_code: output.status.code(),
        elapsed_secs: started_at.elapsed().as_secs_f64(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn render_launch_result(
    config: &LaunchConfig,
    outcome: &LaunchOutcome,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.output_mode {
        OutputMode::Human => {
            if !outcome.success {
                return Err(format!("Launch failed with exit code: {:?}", outcome.exit_code).into());
            }

            println!();
            blocks::print_section("Launch Summary");
            blocks::print_field("Package", &config.package_name);
            blocks::print_field("Launch File", &config.launch_file_name);
            blocks::print_field("Elapsed", format!("{:.2}s", outcome.elapsed_secs));
            blocks::print_success("Launch command exited successfully");
        }
        OutputMode::Plain => {
            output::print_plain_section("launch");
            output::print_plain_field("package", &config.package_name);
            output::print_plain_field("launch_file", &config.launch_file_name);
            if !config.launch_arguments.is_empty() {
                output::print_plain_field("arguments", &config.launch_arguments);
            }
            output::print_plain_field("noninteractive", config.noninteractive);
            output::print_plain_field("debug", config.debug);
            output::print_plain_field("print_only", config.print_only);
            output::print_plain_field("show_args", config.show_args);
            output::print_plain_field("show_all", config.show_all);
            if let Some(prefix) = &config.launch_prefix {
                output::print_plain_field("launch_prefix", prefix);
            }
            if let Some(filter) = &config.launch_prefix_filter {
                output::print_plain_field("launch_prefix_filter", filter);
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
                "command": "launch",
                "package": config.package_name,
                "launch_file": config.launch_file_name,
                "arguments": config.launch_arguments,
                "noninteractive": config.noninteractive,
                "debug": config.debug,
                "print_only": config.print_only,
                "show_args": config.show_args,
                "show_all": config.show_all,
                "launch_prefix": config.launch_prefix,
                "launch_prefix_filter": config.launch_prefix_filter,
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
        Err(error) => print_launch_error(None, &error.to_string(), None),
    };

    let runtime = tokio::runtime::Runtime::new()
        .unwrap_or_else(|error| print_error_and_exit(format!("Failed to create async runtime: {error}")));

    let outcome = match runtime.block_on(execute_launch(&config)) {
        Ok(outcome) => outcome,
        Err(error) => print_launch_error(Some(&config), &error.to_string(), None),
    };

    if let Err(error) = render_launch_result(&config, &outcome) {
        print_launch_error(Some(&config), &error.to_string(), outcome.exit_code);
    }

    if !outcome.success {
        std::process::exit(1);
    }
}
