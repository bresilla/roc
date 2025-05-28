use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 run tf2_ros static_transform_publisher".to_owned();

    let frame_id = matches.get_one::<String>("frame_id").unwrap();
    let child_frame_id = matches.get_one::<String>("child_frame_id").unwrap();
    
    // Add translation parameters with proper string handling
    let default_zero = "0".to_string();
    let default_one = "1".to_string();
    
    let x = matches.get_one::<String>("x").unwrap_or(&default_zero);
    let y = matches.get_one::<String>("y").unwrap_or(&default_zero);
    let z = matches.get_one::<String>("z").unwrap_or(&default_zero);
    
    command.push_str(" ");
    command.push_str(&x.to_string());
    command.push_str(" ");
    command.push_str(&y.to_string());
    command.push_str(" ");
    command.push_str(&z.to_string());

    // Handle rotation (quaternion vs euler)
    if let Some(qx) = matches.get_one::<String>("qx") {
        let qy = matches.get_one::<String>("qy").unwrap_or(&default_zero);
        let qz = matches.get_one::<String>("qz").unwrap_or(&default_zero);
        let qw = matches.get_one::<String>("qw").unwrap_or(&default_one);
        
        command.push_str(" ");
        command.push_str(&qx.to_string());
        command.push_str(" ");
        command.push_str(&qy.to_string());
        command.push_str(" ");
        command.push_str(&qz.to_string());
        command.push_str(" ");
        command.push_str(&qw.to_string());
    } else {
        // Use euler angles (convert to quaternion or use default)
        let roll = matches.get_one::<String>("roll").unwrap_or(&default_zero);
        let pitch = matches.get_one::<String>("pitch").unwrap_or(&default_zero);
        let yaw = matches.get_one::<String>("yaw").unwrap_or(&default_zero);
        
        command.push_str(" ");
        command.push_str(&roll.to_string());
        command.push_str(" ");
        command.push_str(&pitch.to_string());
        command.push_str(" ");
        command.push_str(&yaw.to_string());
    }

    command.push_str(" ");
    command.push_str(&frame_id.to_string());
    command.push_str(" ");
    command.push_str(&child_frame_id.to_string());

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
