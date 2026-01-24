use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::Result;
use clap::ArgMatches;
use colored::*;

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    // Create RCL graph context for direct API access
    // Note: Our implementation always does direct DDS discovery (daemon-free by design)
    let graph_context = RclGraphContext::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Log a note about daemon usage if the flag is explicitly set
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    // Get topic names using direct RCL API calls
    let topics = graph_context
        .get_topic_names()
        .map_err(|e| anyhow::anyhow!("Failed to get topic names: {}", e))?;

    // Handle --count-topics flag
    if matches.get_flag("count_topics") {
        if common_args.ros_style {
            println!("{}", topics.len());
        } else {
            println!(
                "{} {}",
                "Total:".bright_green(),
                topics.len().to_string().bright_white().bold()
            );
        }
        return Ok(());
    }

    // Handle --include-hidden-topics flag
    let filtered_topics: Vec<String> = if matches.get_flag("include_hidden_topics") {
        topics
    } else {
        // Filter out hidden topics (those starting with underscore)
        topics
            .into_iter()
            .filter(|topic| !topic.starts_with("/_"))
            .collect()
    };

    // Handle --show-types flag
    if matches.get_flag("show_types") {
        // Get topics with their type information
        let topics_with_types = graph_context
            .get_topics_with_types()
            .map_err(|e| anyhow::anyhow!("Failed to get topic types: {}", e))?;

        // Filter hidden topics if needed
        let filtered_topics: Vec<_> = if matches.get_flag("include_hidden_topics") {
            topics_with_types
        } else {
            topics_with_types
                .into_iter()
                .filter(|topic| !topic.name.starts_with("/_"))
                .collect()
        };

        // Display topics with types
        for topic in &filtered_topics {
            if common_args.ros_style {
                // Original ROS2 CLI style
                if topic.types.is_empty() {
                    println!("{} [unknown type]", topic.name);
                } else if topic.types.len() == 1 {
                    println!("{} [{}]", topic.name, topic.types[0]);
                } else {
                    // Multiple types (rare but possible)
                    println!("{} [{}]", topic.name, topic.types.join(", "));
                }
            } else {
                if topic.types.is_empty() {
                    println!("{} {}", topic.name.bright_cyan(), "[unknown type]".red());
                } else if topic.types.len() == 1 {
                    println!(
                        "{} {}",
                        topic.name.bright_cyan(),
                        format!("[{}]", topic.types[0]).green()
                    );
                } else {
                    // Multiple types (rare but possible)
                    println!(
                        "{} {}",
                        topic.name.bright_cyan(),
                        format!("[{}]", topic.types.join(", ")).green()
                    );
                }
            }
        }

        if !common_args.ros_style {
            if filtered_topics.is_empty() {
                eprintln!(
                    "{} {}",
                    "No topics found.".yellow(),
                    format!("[{}]", RclGraphContext::get_daemon_status()).bright_black()
                );
            } else {
                println!();
                println!(
                    "{} {} topics found",
                    "Total:".bright_green(),
                    filtered_topics.len().to_string().bright_white().bold()
                );
            }
        }

        return Ok(());
    }

    // Simple topic list (default behavior)
    if common_args.ros_style {
        // Original ROS2 CLI style
        for topic in &filtered_topics {
            println!("{}", topic);
        }
    } else {
        // Enhanced colored output with count header
        if !filtered_topics.is_empty() {
            println!("{}", "Available Topics:".bright_yellow().bold());
            for topic in &filtered_topics {
                println!("  {}", topic.bright_cyan());
            }
            println!();
            println!(
                "{} {} topics found",
                "Total:".bright_green(),
                filtered_topics.len().to_string().bright_white().bold()
            );
        }
    }

    // Handle other flags (for future implementation)
    if common_args.use_sim_time {
        // TODO: Implement simulation time handling when needed
        eprintln!("Warning: --use-sim-time flag not yet implemented in direct RCL mode");
    }

    if common_args.no_daemon {
        // TODO: Our implementation already bypasses daemon, so this is effectively handled
        // We could add logic here to ensure no daemon interaction if needed
    }

    if let Some(spin_time_value) = common_args.spin_time {
        // TODO: Implement spin time logic when needed for live topic discovery
        eprintln!(
            "Warning: --spin-time {} flag not yet implemented in direct RCL mode",
            spin_time_value
        );
    }

    // Show helpful message if no topics found
    if filtered_topics.is_empty() {
        let daemon_status = RclGraphContext::get_daemon_status();
        if common_args.ros_style {
            eprintln!("No topics found. [{}]", daemon_status);
        } else {
            eprintln!(
                "{} {}",
                "No topics found.".yellow(),
                format!("[{}]", daemon_status).bright_black()
            );
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
