use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use crate::arguments::service::CommonServiceArgs;


async fn run_command(matches: ArgMatches, common_args: CommonServiceArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 service call".to_owned();

    let action_name = matches.get_one::<String>("service_name").unwrap();
    command.push_str(" ");
    command.push_str(&action_name.to_string());
    let action_type = matches.get_one::<String>("service_type").unwrap();
    command.push_str(" ");
    command.push_str(&action_type.to_string());

    if matches.get_one::<String>("values") != None {
        let values = matches.get_many::<String>("values").unwrap();
        let mut full_value = String::new();
        command.push_str(" \"");
        for value in values {
            full_value.push_str(&value.to_string());
            full_value.push_str(" ");
        }
        command.push_str(&full_value.to_string());
        command.push_str("\" ");
    }

    if matches.get_one::<String>("rate") != None {
        let rate_value = matches.get_one::<String>("rate").unwrap();
        command.push_str(" --rate ");
        command.push_str(&rate_value.to_string());
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

        let output = String::from_utf8_lossy(&buffer[0..n]);
        print!("{}", output);
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs){
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches, common_args));
}