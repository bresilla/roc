use crate::arguments::action::CommonActionArgs;
use crate::commands::cli::{joined_values, required_string, run_async_command};
use crate::ui::blocks;
use clap::ArgMatches;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

async fn run_command(
    matches: ArgMatches,
    _common_args: CommonActionArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let action_name = required_string(&matches, "action_name")?;
    let action_type = required_string(&matches, "action_type")?;
    let goal_payload = joined_values(&matches, "goal");
    let feedback = matches.get_flag("feedback");

    blocks::print_section("Action Goal");
    blocks::print_field("Action", action_name);
    blocks::print_field("Type", action_type);
    if let Some(goal_payload) = &goal_payload {
        blocks::print_field("Goal", goal_payload);
    }
    blocks::print_field("Feedback", if feedback { "enabled" } else { "disabled" });

    let mut cmd = Command::new("ros2");
    cmd.arg("action")
        .arg("send_goal")
        .arg(action_name)
        .arg(action_type);

    if let Some(payload) = &goal_payload {
        cmd.arg(payload);
    }

    if feedback {
        cmd.arg("--feedback");
    }

    println!();
    blocks::print_field("Command", format!("{cmd:?}"));
    blocks::print_note("Child process stdio is attached to the terminal");

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let started_at = Instant::now();
    let status = cmd.status().await?;
    if !status.success() {
        return Err(format!(
            "ros2 action send_goal failed with exit code: {:?}",
            status.code()
        )
        .into());
    }

    println!();
    blocks::print_section("Goal Summary");
    blocks::print_field("Action", action_name);
    blocks::print_field(
        "Elapsed",
        format!("{:.2}s", started_at.elapsed().as_secs_f64()),
    );
    blocks::print_success("Goal command exited successfully");

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonActionArgs) {
    run_async_command(run_command(matches, common_args));
}
