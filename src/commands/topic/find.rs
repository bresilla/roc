use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::{Result, anyhow};
use clap::ArgMatches;
use colored::*;
use std::time::Duration;

// Topic Find Implementation
//
// This implementation finds topics by message type using:
// 1. Direct RCL API calls to get topic names and types
// 2. Filtering topics by the specified message type
// 3. Supporting count and hidden topic options
// 4. Matching ros2 topic find behavior exactly

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let message_type = matches
        .get_one::<String>("topic_type")
        .ok_or_else(|| anyhow!("Message type is required"))?;

    // Create RCL context for direct API access
    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Allow some time for topic discovery (helpful for /chatter)
    std::thread::sleep(Duration::from_millis(500));

    // Get topics with their types
    let topics_with_types = {
        context
            .get_topics_with_types()
            .map_err(|e| anyhow!("Failed to get topic types: {}", e))?
    };

    // Filter topics by the specified message type
    let mut matching_topics: Vec<String> = topics_with_types
        .iter()
        .filter(|topic| topic.types.contains(message_type))
        .map(|topic| topic.name.clone())
        .collect();

    // Filter hidden topics if needed
    if !matches.get_flag("include_hidden_topics") {
        matching_topics = matching_topics
            .into_iter()
            .filter(|topic| !topic.starts_with("/_"))
            .collect();
    }

    // Handle --count-topics flag
    if matches.get_flag("count_topics") {
        if common_args.ros_style {
            println!("{}", matching_topics.len());
        } else {
            println!(
                "{} {}",
                "Total:".bright_green(),
                matching_topics.len().to_string().bright_white().bold()
            );
        }
        return Ok(());
    }

    // Sort topics for consistent output
    matching_topics.sort();

    // Print matching topics (one per line, like ros2 topic find)
    if common_args.ros_style {
        // Original ROS2 CLI style
        for topic in matching_topics {
            println!("{}", topic);
        }
    } else {
        if matching_topics.is_empty() {
            eprintln!(
                "{} {}",
                "No topics found with type".yellow(),
                format!("[{}]", message_type).bright_cyan()
            );
            return Ok(());
        }

        println!(
            "{} {}",
            "Topics with type".bright_yellow().bold(),
            format!("[{}]", message_type).bright_cyan()
        );
        for topic in matching_topics.iter() {
            println!("  {}", topic.bright_cyan());
        }
        println!();
        println!(
            "{} {} topics found",
            "Total:".bright_green(),
            matching_topics.len().to_string().bright_white().bold()
        );
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
