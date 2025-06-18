use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use crate::arguments::topic::CommonTopicArgs;

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 topic echo".to_owned();
    
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
    if matches.get_one::<String>("qos_profile") != None {
        let qos_profile_value = matches.get_one::<String>("qos_profile").unwrap();
        command.push_str(" --qos-profile ");
        command.push_str(&qos_profile_value.to_string());
    }
    if matches.get_one::<String>("qos_depth") != None {
        let qos_depth_value = matches.get_one::<String>("qos_depth").unwrap();
        command.push_str(" --qos-depth ");
        command.push_str(&qos_depth_value.to_string());
    }
    if matches.get_one::<String>("qos_history") != None {
        let qos_history_value = matches.get_one::<String>("qos_history").unwrap();
        command.push_str(" --qos-history ");
        command.push_str(&qos_history_value.to_string());
    }
    if matches.get_one::<String>("qos_reliability") != None {
        let qos_reliability_value = matches.get_one::<String>("qos_reliability").unwrap();
        command.push_str(" --qos-reliability ");
        command.push_str(&qos_reliability_value.to_string());
    }
    if matches.get_one::<String>("qos_durability") != None {
        let qos_durability_value = matches.get_one::<String>("qos_durability").unwrap();
        command.push_str(" --qos-durability ");
        command.push_str(&qos_durability_value.to_string());
    }
    if matches.get_flag("csv") {
        command.push_str(" --csv");
    }
    if matches.get_one::<String>("field") != None {
        let field_value = matches.get_one::<String>("field").unwrap();
        command.push_str(" --field ");
        command.push_str(&field_value.to_string());
    }
    if matches.get_flag("full_length") {
        command.push_str(" --full-length");
    }
    if matches.get_one::<String>("truncate_length") != None {
        let truncate_length_value = matches.get_one::<String>("truncate_length").unwrap();
        command.push_str(" --truncate-length ");
        command.push_str(&truncate_length_value.to_string());
    }
    if matches.get_flag("no_arr") {
        command.push_str(" --no-arr");
    }
    if matches.get_flag("no_str") {
        command.push_str(" --no-str");
    }
    if matches.get_flag("flow_style") {
        command.push_str(" --flow-style");
    }
    if matches.get_flag("no_lost_messages") {
        command.push_str(" --no-lost-messages");
    }
    if matches.get_flag("raw") {
        command.push_str(" --raw");
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

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs){
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(run_command(matches, common_args));
}