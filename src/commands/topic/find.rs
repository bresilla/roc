use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::json;
use std::time::Duration;

// Topic Find Implementation
//
// This implementation finds topics by message type using:
// 1. Direct RCL API calls to get topic names and types
// 2. Filtering topics by the specified message type
// 3. Supporting count and hidden topic options
// 4. Matching ros2 topic find behavior exactly

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches_with_compat(&matches, common_args.ros_style);
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
        match output_mode {
            output::OutputMode::Human => {
                blocks::print_total(matching_topics.len(), "topic", "topics");
            }
            output::OutputMode::Plain => println!("{}", matching_topics.len()),
            output::OutputMode::Json => {
                let count = matching_topics.len();
                output::print_json(
                    &json!({ "topic_type": message_type, "topics": matching_topics, "count": count }),
                )?;
            }
        }
        return Ok(());
    }

    // Sort topics for consistent output
    matching_topics.sort();

    match output_mode {
        output::OutputMode::Human => {
            if matching_topics.is_empty() {
                blocks::eprint_warning(&format!("No topics found for type {message_type}"));
                return Ok(());
            }

            blocks::print_section("Topics");
            blocks::print_field("Requested Type", message_type);
            println!();
            let rows = matching_topics
                .iter()
                .map(|topic| vec![topic.clone()])
                .collect();
            table::print_table(&["Topic"], rows);
            blocks::print_total(matching_topics.len(), "topic", "topics");
        }
        output::OutputMode::Plain => {
            for topic in &matching_topics {
                println!("{topic}");
            }
        }
        output::OutputMode::Json => {
            let count = matching_topics.len();
            output::print_json(
                &json!({ "topic_type": message_type, "topics": matching_topics, "count": count }),
            )?;
        }
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
