use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use crate::ui::{blocks, output, table};
use anyhow::Result;
use clap::ArgMatches;
use colored::*;
use serde_json::json;

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches_with_compat(&matches, common_args.ros_style);
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

    // Handle --count-topics flag
    if matches.get_flag("count_topics") {
        match output_mode {
            output::OutputMode::Human => {
                println!(
                    "{} {}",
                    "Total:".bright_green(),
                    filtered_topics.len().to_string().bright_white().bold()
                );
            }
            output::OutputMode::Plain => println!("{}", filtered_topics.len()),
            output::OutputMode::Json => {
                output::print_json(&json!({ "count": filtered_topics.len() }))?
            }
        }
        return Ok(());
    }

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
        match output_mode {
            output::OutputMode::Plain => {
                for topic in &filtered_topics {
                    if topic.types.is_empty() {
                        println!("{}\tunknown", topic.name);
                    } else if topic.types.len() == 1 {
                        println!("{}\t{}", topic.name, topic.types[0]);
                    } else {
                        println!("{}\t{}", topic.name, topic.types.join(", "));
                    }
                }
            }
            output::OutputMode::Human => {
                blocks::print_section("Topics");
                let rows = filtered_topics
                    .iter()
                    .map(|topic| {
                        let type_label = if topic.types.is_empty() {
                            "unknown".red().to_string()
                        } else if topic.types.len() == 1 {
                            topic.types[0].green().to_string()
                        } else {
                            topic.types.join(", ").green().to_string()
                        };
                        vec![topic.name.bright_cyan().to_string(), type_label]
                    })
                    .collect();
                table::print_table(&["Topic", "Type"], rows);

                if filtered_topics.is_empty() {
                    eprintln!(
                        "{} {}",
                        "No topics found.".yellow(),
                        format!("[{}]", RclGraphContext::get_daemon_status()).bright_black()
                    );
                } else {
                    blocks::print_total(filtered_topics.len(), "topic", "topics");
                }
            }
            output::OutputMode::Json => {
                let topics = filtered_topics
                    .iter()
                    .map(|topic| json!({ "name": topic.name, "types": topic.types }))
                    .collect::<Vec<_>>();
                output::print_json(&json!({ "topics": topics, "count": filtered_topics.len() }))?;
            }
        }

        return Ok(());
    }

    // Simple topic list (default behavior)
    match output_mode {
        output::OutputMode::Plain => {
            for topic in &filtered_topics {
                println!("{topic}");
            }
        }
        output::OutputMode::Human => {
            if !filtered_topics.is_empty() {
                blocks::print_section("Topics");
                let rows = filtered_topics
                    .iter()
                    .map(|topic| vec![topic.bright_cyan().to_string()])
                    .collect();
                table::print_table(&["Topic"], rows);
                blocks::print_total(filtered_topics.len(), "topic", "topics");
            }
        }
        output::OutputMode::Json => {
            let count = filtered_topics.len();
            output::print_json(&json!({ "topics": filtered_topics, "count": count }))?;
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
        match output_mode {
            output::OutputMode::Human => {
                eprintln!(
                    "{} {}",
                    "No topics found.".yellow(),
                    format!("[{}]", daemon_status).bright_black()
                );
            }
            output::OutputMode::Plain => eprintln!("No topics found. [{}]", daemon_status),
            output::OutputMode::Json => {}
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
