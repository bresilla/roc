use crate::arguments::action::CommonActionArgs;
use crate::commands::cli::{joined_values, required_string, run_async_command};
use crate::shared::preflight::{ensure_command_available, ensure_ros_environment};
use clap::ArgMatches;
use colored::Colorize;
use std::process::Stdio;
use tokio::process::Command;

async fn run_command(
    matches: ArgMatches,
    _common_args: CommonActionArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_command_available("ros2", "roc action goal")?;
    ensure_ros_environment("roc action goal")?;
    println!("{}", "Delegating to ros2 action send_goal".bright_black());
    let action_name = required_string(&matches, "action_name")?;
    let action_type = required_string(&matches, "action_type")?;

    let mut cmd = Command::new("ros2");
    cmd.arg("action")
        .arg("send_goal")
        .arg(action_name)
        .arg(action_type);

    if let Some(payload) = joined_values(&matches, "goal") {
        cmd.arg(payload);
    }

    if matches.get_flag("feedback") {
        cmd.arg("--feedback");
    }

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let status = cmd.status().await?;
    if !status.success() {
        return Err(format!(
            "ros2 action send_goal failed with exit code: {:?}",
            status.code()
        )
        .into());
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonActionArgs) {
    run_async_command(run_command(matches, common_args));
}
