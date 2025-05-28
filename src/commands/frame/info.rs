use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 run tf2_ros buffer_client".to_owned();

    let frame_name = matches.get_one::<String>("frame_name").unwrap();
    command.push_str(" ");
    command.push_str(&frame_name.to_string());

    if matches.get_flag("include_hidden_services") {
        command.push_str(" --include-hidden-services");
    }

    if let Some(export_dot) = matches.get_one::<String>("export_dot") {
        command.push_str(" --export-dot ");
        command.push_str(&export_dot.to_string());
    }
    
    if let Some(export_json) = matches.get_one::<String>("export_json") {
        command.push_str(" --export-json ");
        command.push_str(&export_json.to_string());
    }
    
    if let Some(export_yaml) = matches.get_one::<String>("export_yaml") {
        command.push_str(" --export-yaml ");
        command.push_str(&export_yaml.to_string());
    }

    if let Some(export_image) = matches.get_one::<String>("export_image") {
        command.push_str(" --export-image ");
        command.push_str(&export_image.to_string());
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
