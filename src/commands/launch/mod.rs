use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 launch".to_owned();

    let package_name = matches.get_one::<String>("package_name").unwrap();
    let launch_file_name = matches.get_one::<String>("launch_file_name").unwrap();
    
    command.push_str(" ");
    command.push_str(&package_name.to_string());
    command.push_str(" ");
    command.push_str(&launch_file_name.to_string());

    if let Some(launch_arguments) = matches.get_one::<String>("launch_arguments") {
        command.push_str(" ");
        command.push_str(&launch_arguments.to_string());
    }

    if matches.get_flag("noninteractive") {
        command.push_str(" --noninteractive");
    }
    
    if matches.get_flag("debug") {
        command.push_str(" --debug");
    }
    
    if matches.get_flag("print") {
        command.push_str(" --print");
    }
    
    if matches.get_flag("show_args") {
        command.push_str(" --show-args");
    }
    
    if matches.get_flag("show_all") {
        command.push_str(" --show-all-subprocesses-output");
    }

    if let Some(launch_prefix) = matches.get_one::<String>("launch_prefix") {
        command.push_str(" --launch-prefix '");
        command.push_str(&launch_prefix.to_string());
        command.push_str("'");
    }

    if let Some(launch_prefix_filter) = matches.get_one::<String>("launch_prefix_filter") {
        command.push_str(" --launch-prefix-filter '");
        command.push_str(&launch_prefix_filter.to_string());
        command.push_str("'");
    }

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