use crate::arguments::topic::CommonTopicArgs;
use crate::commands::cli::handle_anyhow_result;
use crate::graph::RclGraphContext;
use crate::ui::{blocks, output};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::json;
use std::time::Duration;

// Topic Type (Kind) Implementation
//
// This implementation shows the message type for a topic using:
// 1. Direct RCL API calls to get topic types
// 2. Simple topic name to type lookup
// 3. Clean output matching ros2 topic type behavior

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches_with_compat(&matches, common_args.ros_style);
    let topic_name = matches
        .get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;

    // Create RCL context for direct API access
    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Wait for topic to appear (especially useful for /chatter)
    if !context.wait_for_topic(topic_name, Duration::from_secs(3))? {
        let daemon_status = RclGraphContext::get_daemon_status();
        return Err(anyhow!(
            "Topic '{}' not found. [{}]",
            topic_name,
            daemon_status
        ));
    }

    // Get topic type
    let topic_type = {
        let topics_and_types = context
            .get_topic_names_and_types()
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

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Topic");
            blocks::print_field("Name", topic_name);
            blocks::print_field("Type", &topic_type);
        }
        output::OutputMode::Plain => println!("{topic_type}"),
        output::OutputMode::Json => {
            output::print_json(&json!({ "name": topic_name, "type": topic_type }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
