use crate::arguments::service::CommonServiceArgs;
use crate::commands::cli::{joined_values, print_error_and_exit, required_string};
use crate::ui::{
    blocks,
    output::{self, OutputMode},
};
use clap::ArgMatches;
use serde_json::json;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

#[derive(Clone)]
struct ServiceCallConfig {
    service_name: String,
    service_type: String,
    payload: Option<String>,
    rate_value: Option<String>,
    output_mode: OutputMode,
}

struct CommandOutcome {
    command_preview: String,
    success: bool,
    exit_code: Option<i32>,
    elapsed_secs: f64,
    stdout: String,
    stderr: String,
}

fn config_from_matches(matches: &ArgMatches) -> Result<ServiceCallConfig, Box<dyn std::error::Error>> {
    Ok(ServiceCallConfig {
        service_name: required_string(matches, "service_name")?.to_string(),
        service_type: required_string(matches, "service_type")?.to_string(),
        payload: joined_values(matches, "values"),
        rate_value: matches.get_one::<String>("rate").cloned(),
        output_mode: OutputMode::from_matches(matches),
    })
}

fn print_multiline_output(label: &str, content: &str) {
    if !content.trim().is_empty() {
        output::print_plain_multiline_field(label, content);
    }
}

fn print_service_call_error(
    config: Option<&ServiceCallConfig>,
    error: &str,
    exit_code: Option<i32>,
) -> ! {
    let output_mode = config
        .map(|config| config.output_mode)
        .unwrap_or(OutputMode::Human);

    match output_mode {
        OutputMode::Human => print_error_and_exit(error),
        OutputMode::Plain => {
            output::print_plain_section("service-call");
            if let Some(config) = config {
                output::print_plain_field("service", &config.service_name);
                output::print_plain_field("type", &config.service_type);
                if let Some(payload) = &config.payload {
                    output::print_plain_field("request", payload);
                }
                if let Some(rate_value) = &config.rate_value {
                    output::print_plain_field("rate_hz", rate_value);
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
                    "command": "service call",
                    "service": config.service_name,
                    "type": config.service_type,
                    "request": config.payload,
                    "rate_hz": config.rate_value,
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            });
            let _ = output::print_json(&payload.unwrap_or_else(|| {
                json!({
                    "command": "service call",
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            }));
            std::process::exit(1);
        }
    }
}

async fn execute_service_call(config: &ServiceCallConfig) -> Result<CommandOutcome, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("ros2");
    cmd.arg("service")
        .arg("call")
        .arg(&config.service_name)
        .arg(&config.service_type);

    if let Some(payload) = &config.payload {
        cmd.arg(payload);
    }

    if let Some(rate_value) = &config.rate_value {
        cmd.arg("--rate").arg(rate_value);
    }

    let command_preview = format!("{cmd:?}");

    if config.output_mode == OutputMode::Human {
        blocks::print_section("Service Call");
        blocks::print_field("Service", &config.service_name);
        blocks::print_field("Type", &config.service_type);
        if let Some(payload) = &config.payload {
            blocks::print_field("Request", payload);
        }
        if let Some(rate) = &config.rate_value {
            blocks::print_field("Rate", format!("{rate} Hz"));
        } else {
            blocks::print_field("Mode", "single call");
        }

        println!();
        blocks::print_field("Command", &command_preview);
        blocks::print_note("Child process stdio is attached to the terminal");

        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());

        let started_at = Instant::now();
        let status = cmd.status().await?;
        return Ok(CommandOutcome {
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
    Ok(CommandOutcome {
        command_preview,
        success: output.status.success(),
        exit_code: output.status.code(),
        elapsed_secs: started_at.elapsed().as_secs_f64(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn render_service_call_result(
    config: &ServiceCallConfig,
    outcome: &CommandOutcome,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.output_mode {
        OutputMode::Human => {
            if !outcome.success {
                return Err(format!(
                    "ros2 service call failed with exit code: {:?}",
                    outcome.exit_code
                )
                .into());
            }

            println!();
            blocks::print_section("Call Summary");
            blocks::print_field("Service", &config.service_name);
            blocks::print_field("Elapsed", format!("{:.2}s", outcome.elapsed_secs));
            blocks::print_success("Service call exited successfully");
        }
        OutputMode::Plain => {
            output::print_plain_section("service-call");
            output::print_plain_field("service", &config.service_name);
            output::print_plain_field("type", &config.service_type);
            if let Some(payload) = &config.payload {
                output::print_plain_field("request", payload);
            }
            if let Some(rate_value) = &config.rate_value {
                output::print_plain_field("rate_hz", rate_value);
            } else {
                output::print_plain_field("mode", "single");
            }
            output::print_plain_field("command", &outcome.command_preview);
            output::print_plain_field("exit_code", outcome.exit_code.unwrap_or_default());
            output::print_plain_field("elapsed_secs", format!("{:.3}", outcome.elapsed_secs));
            output::print_plain_field("status", if outcome.success { "ok" } else { "error" });
            print_multiline_output("stdout", &outcome.stdout);
            print_multiline_output("stderr", &outcome.stderr);
        }
        OutputMode::Json => {
            output::print_json(&json!({
                "command": "service call",
                "service": config.service_name,
                "type": config.service_type,
                "request": config.payload,
                "rate_hz": config.rate_value,
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

pub fn handle(matches: ArgMatches, _common_args: CommonServiceArgs) {
    let config = match config_from_matches(&matches) {
        Ok(config) => config,
        Err(error) => print_service_call_error(None, &error.to_string(), None),
    };

    let runtime = tokio::runtime::Runtime::new()
        .unwrap_or_else(|error| print_error_and_exit(format!("Failed to create async runtime: {error}")));

    let outcome = match runtime.block_on(execute_service_call(&config)) {
        Ok(outcome) => outcome,
        Err(error) => print_service_call_error(Some(&config), &error.to_string(), None),
    };

    if let Err(error) = render_service_call_result(&config, &outcome) {
        print_service_call_error(Some(&config), &error.to_string(), outcome.exit_code);
    }

    if !outcome.success {
        std::process::exit(1);
    }
}
