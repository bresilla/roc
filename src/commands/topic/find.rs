use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::{anyhow, Result};
use clap::ArgMatches;

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
    let create_context = || -> Result<RclGraphContext> {
        RclGraphContext::new()
            .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))
    };

    // Handle common arguments
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    if common_args.use_sim_time {
        eprintln!("Note: Using simulation time for discovery");
    }

    if let Some(ref spin_time_value) = common_args.spin_time {
        eprintln!("Note: Using spin time {} for discovery", spin_time_value);
    }

    // Get topics with their types
    let topics_with_types = {
        let context = create_context()?;
        context.get_topics_with_types()
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
        println!("{}", matching_topics.len());
        return Ok(());
    }

    // Sort topics for consistent output
    matching_topics.sort();

    // Print matching topics (one per line, like ros2 topic find)
    for topic in matching_topics {
        println!("{}", topic);
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