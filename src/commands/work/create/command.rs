use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 pkg create".to_owned();

    let package_name = matches.get_one::<String>("package_name").unwrap();
    command.push_str(" ");
    command.push_str(&package_name.to_string());

    if let Some(package_format) = matches.get_one::<String>("package_format") {
        command.push_str(" --package-format ");
        command.push_str(&package_format.to_string());
    }

    if let Some(description) = matches.get_one::<String>("description") {
        command.push_str(" --description '");
        command.push_str(&description.to_string());
        command.push_str("'");
    }

    if let Some(license) = matches.get_one::<String>("license") {
        command.push_str(" --license ");
        command.push_str(&license.to_string());
    }

    if let Some(destination_directory) = matches.get_one::<String>("destination_directory") {
        command.push_str(" --destination-directory ");
        command.push_str(&destination_directory.to_string());
    }

    if let Some(build_type) = matches.get_one::<String>("build_type") {
        command.push_str(" --build-type ");
        command.push_str(&build_type.to_string());
    }

    if let Some(dependencies) = matches.get_many::<String>("dependencies") {
        for dep in dependencies {
            command.push_str(" --dependencies ");
            command.push_str(&dep.to_string());
        }
    }

    if let Some(maintainer_email) = matches.get_one::<String>("maintainer_email") {
        command.push_str(" --maintainer-email ");
        command.push_str(&maintainer_email.to_string());
    }

    if let Some(maintainer_name) = matches.get_one::<String>("maintainer_name") {
        command.push_str(" --maintainer-name '");
        command.push_str(&maintainer_name.to_string());
        command.push_str("'");
    }

    if let Some(node_name) = matches.get_one::<String>("node_name") {
        command.push_str(" --node-name ");
        command.push_str(&node_name.to_string());
    }

    if let Some(library_name) = matches.get_one::<String>("library_name") {
        command.push_str(" --library-name ");
        command.push_str(&library_name.to_string());
    }

    println!("Creating package: {}", command);

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
