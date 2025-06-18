use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use crate::arguments::topic::CommonTopicArgs;

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 topic type".to_owned();

    let topic_name = matches.get_one::<String>("topic_name").unwrap();
    command.push_str(" ");
    command.push_str(&topic_name.to_string());

    if let Some(spin_time_value) = &common_args.spin_time {
        command.push_str(" --spin-time ");
        command.push_str(spin_time_value);
    }
    if common_args.use_sim_time {
        command.push_str(" --use-sim-time");
    }
    if common_args.no_daemon {
        command.push_str(" --no-daemon");
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

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs){
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches, common_args));
}