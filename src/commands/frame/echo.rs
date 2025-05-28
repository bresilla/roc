use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 run tf2_ros tf2_echo".to_owned();

    let frame_id = matches.get_one::<String>("frame_id").unwrap();
    let child_frame_id = matches.get_one::<String>("child_frame_id").unwrap();
    command.push_str(" ");
    command.push_str(&frame_id.to_string());
    command.push_str(" ");
    command.push_str(&child_frame_id.to_string());

    if let Some(rate) = matches.get_one::<String>("rate") {
        command.push_str(" --rate ");
        command.push_str(&rate.to_string());
    }
    
    if matches.get_flag("once") {
        command.push_str(" --once");
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

pub fn handle(matches: ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches));
}
