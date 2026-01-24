use crate::arguments::service::CommonServiceArgs;
use clap::ArgMatches;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

async fn run_command(
    matches: ArgMatches,
    _common_args: CommonServiceArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = String::from("ros2 service call");

    let service_name = matches.get_one::<String>("service_name").unwrap();
    command.push(' ');
    command.push_str(service_name);

    let service_type = matches.get_one::<String>("service_type").unwrap();
    command.push(' ');
    command.push_str(service_type);

    if let Some(values) = matches.get_many::<String>("values") {
        let mut full_value = String::new();
        for value in values {
            full_value.push_str(value);
            full_value.push(' ');
        }
        command.push_str(" \"");
        command.push_str(full_value.trim_end());
        command.push_str("\"");
    }

    if let Some(rate_value) = matches.get_one::<String>("rate") {
        command.push_str(" --rate ");
        command.push_str(rate_value);
    }

    let mut cmd = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().unwrap();
    let mut reader = tokio::io::BufReader::new(stdout);

    let mut buffer = [0u8; 1024];
    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        let output = String::from_utf8_lossy(&buffer[..n]);
        print!("{}", output);
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches, common_args));
}
