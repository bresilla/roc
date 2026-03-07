use crate::arguments::service::CommonServiceArgs;
use crate::commands::cli::{joined_values, required_string, run_async_command};
use crate::ui::blocks;
use clap::ArgMatches;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

async fn run_command(
    matches: ArgMatches,
    _common_args: CommonServiceArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = required_string(&matches, "service_name")?;
    let service_type = required_string(&matches, "service_type")?;
    let payload = joined_values(&matches, "values");
    let rate_value = matches.get_one::<String>("rate").cloned();

    blocks::print_section("Service Call");
    blocks::print_field("Service", service_name);
    blocks::print_field("Type", service_type);
    if let Some(payload) = &payload {
        blocks::print_field("Request", payload);
    }
    if let Some(rate) = &rate_value {
        blocks::print_field("Rate", format!("{rate} Hz"));
    } else {
        blocks::print_field("Mode", "single call");
    }

    let mut cmd = Command::new("ros2");
    cmd.arg("service")
        .arg("call")
        .arg(service_name)
        .arg(service_type);

    if let Some(payload) = &payload {
        cmd.arg(payload);
    }

    if let Some(rate_value) = &rate_value {
        cmd.arg("--rate").arg(rate_value);
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
            "ros2 service call failed with exit code: {:?}",
            status.code()
        )
        .into());
    }

    println!();
    blocks::print_section("Call Summary");
    blocks::print_field("Service", service_name);
    blocks::print_field(
        "Elapsed",
        format!("{:.2}s", started_at.elapsed().as_secs_f64()),
    );
    blocks::print_success("Service call exited successfully");

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    run_async_command(run_command(matches, common_args));
}
