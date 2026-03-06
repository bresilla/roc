use crate::arguments::action::CommonActionArgs;
use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;

async fn run_command(
    matches: ArgMatches,
    _common_args: CommonActionArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let action_name = matches.get_one::<String>("action_name").unwrap();
    let action_type = matches.get_one::<String>("action_type").unwrap();

    let mut cmd = Command::new("ros2");
    cmd.arg("action")
        .arg("send_goal")
        .arg(action_name)
        .arg(action_type);

    let values = matches.get_many::<String>("goal").unwrap();
    let payload = values
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if !payload.is_empty() {
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
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches, common_args));
}
