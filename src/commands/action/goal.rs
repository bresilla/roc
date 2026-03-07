use crate::arguments::action::CommonActionArgs;
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
struct ActionGoalConfig {
    action_name: String,
    action_type: String,
    goal_payload: Option<String>,
    feedback: bool,
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

fn config_from_matches(matches: &ArgMatches) -> Result<ActionGoalConfig, Box<dyn std::error::Error>> {
    Ok(ActionGoalConfig {
        action_name: required_string(matches, "action_name")?.to_string(),
        action_type: required_string(matches, "action_type")?.to_string(),
        goal_payload: joined_values(matches, "goal"),
        feedback: matches.get_flag("feedback"),
        output_mode: OutputMode::from_matches(matches),
    })
}

fn print_multiline_output(label: &str, content: &str) {
    if !content.trim().is_empty() {
        output::print_plain_multiline_field(label, content);
    }
}

fn print_action_goal_error(
    config: Option<&ActionGoalConfig>,
    error: &str,
    exit_code: Option<i32>,
) -> ! {
    let output_mode = config
        .map(|config| config.output_mode)
        .unwrap_or(OutputMode::Human);

    match output_mode {
        OutputMode::Human => print_error_and_exit(error),
        OutputMode::Plain => {
            output::print_plain_section("action-goal");
            if let Some(config) = config {
                output::print_plain_field("action", &config.action_name);
                output::print_plain_field("type", &config.action_type);
                if let Some(goal_payload) = &config.goal_payload {
                    output::print_plain_field("goal", goal_payload);
                }
                output::print_plain_field("feedback", config.feedback);
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
                    "command": "action goal",
                    "action": config.action_name,
                    "type": config.action_type,
                    "goal": config.goal_payload,
                    "feedback": config.feedback,
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            });
            let _ = output::print_json(&payload.unwrap_or_else(|| {
                json!({
                    "command": "action goal",
                    "exit_code": exit_code,
                    "status": "error",
                    "error": error
                })
            }));
            std::process::exit(1);
        }
    }
}

async fn execute_action_goal(config: &ActionGoalConfig) -> Result<CommandOutcome, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("ros2");
    cmd.arg("action")
        .arg("send_goal")
        .arg(&config.action_name)
        .arg(&config.action_type);

    if let Some(payload) = &config.goal_payload {
        cmd.arg(payload);
    }

    if config.feedback {
        cmd.arg("--feedback");
    }

    let command_preview = format!("{cmd:?}");

    if config.output_mode == OutputMode::Human {
        blocks::print_section("Action Goal");
        blocks::print_field("Action", &config.action_name);
        blocks::print_field("Type", &config.action_type);
        if let Some(goal_payload) = &config.goal_payload {
            blocks::print_field("Goal", goal_payload);
        }
        blocks::print_field(
            "Feedback",
            if config.feedback { "enabled" } else { "disabled" },
        );

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

fn render_action_goal_result(
    config: &ActionGoalConfig,
    outcome: &CommandOutcome,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.output_mode {
        OutputMode::Human => {
            if !outcome.success {
                return Err(format!(
                    "ros2 action send_goal failed with exit code: {:?}",
                    outcome.exit_code
                )
                .into());
            }

            println!();
            blocks::print_section("Goal Summary");
            blocks::print_field("Action", &config.action_name);
            blocks::print_field("Elapsed", format!("{:.2}s", outcome.elapsed_secs));
            blocks::print_success("Goal command exited successfully");
        }
        OutputMode::Plain => {
            output::print_plain_section("action-goal");
            output::print_plain_field("action", &config.action_name);
            output::print_plain_field("type", &config.action_type);
            if let Some(goal_payload) = &config.goal_payload {
                output::print_plain_field("goal", goal_payload);
            }
            output::print_plain_field("feedback", config.feedback);
            output::print_plain_field("command", &outcome.command_preview);
            output::print_plain_field("exit_code", outcome.exit_code.unwrap_or_default());
            output::print_plain_field("elapsed_secs", format!("{:.3}", outcome.elapsed_secs));
            output::print_plain_field("status", if outcome.success { "ok" } else { "error" });
            print_multiline_output("stdout", &outcome.stdout);
            print_multiline_output("stderr", &outcome.stderr);
        }
        OutputMode::Json => {
            output::print_json(&json!({
                "command": "action goal",
                "action": config.action_name,
                "type": config.action_type,
                "goal": config.goal_payload,
                "feedback": config.feedback,
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

pub fn handle(matches: ArgMatches, _common_args: CommonActionArgs) {
    let config = match config_from_matches(&matches) {
        Ok(config) => config,
        Err(error) => print_action_goal_error(None, &error.to_string(), None),
    };

    let runtime = tokio::runtime::Runtime::new()
        .unwrap_or_else(|error| print_error_and_exit(format!("Failed to create async runtime: {error}")));

    let outcome = match runtime.block_on(execute_action_goal(&config)) {
        Ok(outcome) => outcome,
        Err(error) => print_action_goal_error(Some(&config), &error.to_string(), None),
    };

    if let Err(error) = render_action_goal_result(&config, &outcome) {
        print_action_goal_error(Some(&config), &error.to_string(), outcome.exit_code);
    }

    if !outcome.success {
        std::process::exit(1);
    }
}
