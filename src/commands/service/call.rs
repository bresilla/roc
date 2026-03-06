use crate::arguments::service::CommonServiceArgs;
use crate::commands::cli::{joined_values, required_string, run_async_command};
use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;

async fn run_command(
    matches: ArgMatches,
    _common_args: CommonServiceArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = required_string(&matches, "service_name")?;
    let service_type = required_string(&matches, "service_type")?;

    let mut cmd = Command::new("ros2");
    cmd.arg("service")
        .arg("call")
        .arg(service_name)
        .arg(service_type);

    if let Some(payload) = joined_values(&matches, "values") {
        cmd.arg(payload);
    }

    if let Some(rate_value) = matches.get_one::<String>("rate") {
        cmd.arg("--rate").arg(rate_value);
    }

    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let status = cmd.status().await?;
    if !status.success() {
        return Err(format!(
            "ros2 service call failed with exit code: {:?}",
            status.code()
        )
        .into());
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    run_async_command(run_command(matches, common_args));
}
