use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 bag record".to_owned();

    if let Some(topics) = matches.get_many::<String>("topics") {
        for topic in topics {
            command.push_str(" ");
            command.push_str(&topic.to_string());
        }
    }

    if let Some(output) = matches.get_one::<String>("output") {
        command.push_str(" -o ");
        command.push_str(&output.to_string());
    }

    if matches.get_flag("all") {
        command.push_str(" --all");
    }

    println!("Recording bag: {}", command);

    let mut cmd = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
