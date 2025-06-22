use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::time::Duration;
use colored::*;

// Topic Type (Kind) Implementation
// 
// This implementation shows the message type for a topic using:
// 1. Direct RCL API calls to get topic types
// 2. Simple topic name to type lookup
// 3. Clean output matching ros2 topic type behavior

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let topic_name = matches
        .get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;

    // Create RCL context for direct API access
    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Wait for topic to appear (especially useful for /chatter)
    if !context.wait_for_topic(topic_name, Duration::from_secs(3))? {
        let daemon_status = RclGraphContext::get_daemon_status();
        return Err(anyhow!("Topic '{}' not found. [{}]", topic_name, daemon_status));
    }

    // Get topic type
    let topic_type = {
        let topics_and_types = context.get_topic_names_and_types()
            .map_err(|e| anyhow!("Failed to get topic types: {}", e))?;

        topics_and_types
            .iter()
            .find(|(name, _)| name == topic_name)
            .map(|(_, type_name)| type_name.clone())
            .ok_or_else(|| {
                let daemon_status = RclGraphContext::get_daemon_status();
                anyhow!("Topic '{}' not found. [{}]", topic_name, daemon_status)
            })?
    };

    // Simple output - just the type name (like ros2 topic type)
    if common_args.ros_style {
        // Original ROS2 CLI style
        println!("{}", topic_type);
    } else {
        // Enhanced colored output
        println!("{}", topic_type.bright_cyan());
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    match run_command(matches, common_args) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}