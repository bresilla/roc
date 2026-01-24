use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use crate::arguments::action::CommonActionArgs;


async fn run_command(matches: ArgMatches, common_args: CommonActionArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 action info".to_owned();

    let action_name = matches.get_one::<String>("action_name").unwrap();
    command.push_str(" ");
    command.push_str(&action_name.to_string());

    if matches.get_flag("show_types") {
        command.push_str(" --show-types");
    }
    if matches.get_flag("count_actions") {
        command.push_str(" --count");
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

pub fn handle(matches: ArgMatches, common_args: CommonActionArgs){
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches, common_args));
}