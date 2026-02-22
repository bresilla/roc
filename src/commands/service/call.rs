use crate::arguments::service::CommonServiceArgs;
use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;

async fn run_command(
    matches: ArgMatches,
    _common_args: CommonServiceArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = matches.get_one::<String>("service_name").unwrap();
    let service_type = matches.get_one::<String>("service_type").unwrap();

    let mut cmd = Command::new("ros2");
    cmd.arg("service")
        .arg("call")
        .arg(service_name)
        .arg(service_type);

    if let Some(values) = matches.get_many::<String>("values") {
        let payload = values
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
        if !payload.is_empty() {
            cmd.arg(payload);
        }
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
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches, common_args));
}
