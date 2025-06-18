use clap::ArgMatches;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use crate::arguments::topic::CommonTopicArgs;

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = "ros2 topic pub".to_owned();

    let topic_name = matches.get_one::<String>("topic_name").unwrap();
    command.push_str(" ");
    command.push_str(&topic_name.to_string());
    let message_type = matches.get_one::<String>("message_type").unwrap();
    command.push_str(" ");
    command.push_str(&message_type.to_string());
    let values = matches.get_many::<String>("values").unwrap();
    let mut full_value = String::new();
    command.push_str(" \"");
    for value in values {
        full_value.push_str(&value.to_string());
        full_value.push_str(" ");
    }
    command.push_str(&full_value.to_string());
    command.push_str("\" ");

    // println!("{}", full_value);

    if matches.get_one::<String>("rate") != None {
        let rate_value = matches.get_one::<String>("rate").unwrap();
        command.push_str(" --rate ");
        command.push_str(&rate_value.to_string());
    }
    if matches.get_one::<String>("print") != None {
        let print_value = matches.get_one::<String>("print").unwrap();
        command.push_str(" --print ");
        command.push_str(&print_value.to_string());
    }
    if matches.get_flag("once") {
        command.push_str(" --once");
    }
    if matches.get_one::<String>("times") != None {
        let times_value = matches.get_one::<String>("times").unwrap();
        command.push_str(" --times ");
        command.push_str(&times_value.to_string());
    }
    if matches.get_one::<String>("wait_matching_subscriptions") != None {
        let wait_matching_subscriptions_value = matches.get_one::<String>("wait_matching_subscriptions").unwrap();
        command.push_str(" --wait-matching-subscriptions ");
        command.push_str(&wait_matching_subscriptions_value.to_string());
    }
    if matches.get_one::<String>("keep_alive") != None {
        let keep_alive_value = matches.get_one::<String>("keep_alive").unwrap();
        command.push_str(" --keep-alive ");
        command.push_str(&keep_alive_value.to_string());
    }
    if matches.get_one::<String>("node_name") != None {
        let node_name_value = matches.get_one::<String>("node_name").unwrap();
        command.push_str(" --node-name ");
        command.push_str(&node_name_value.to_string());
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
    if let Some(spin_time_value) = &common_args.spin_time {
        command.push_str(" --spin-time ");
        command.push_str(spin_time_value);
    }
    if common_args.use_sim_time {
        command.push_str(" --use-sim-time");
    }

    println!("{}", command);

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