use crate::arguments::topic::CommonTopicArgs;
use crate::commands::cli::handle_anyhow_result;
use crate::graph::RclGraphContext;
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use serde_json::json;

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches_with_compat(&matches, common_args.ros_style);
    let topic_name = matches
        .get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;

    let verbose = matches.get_flag("verbose");

    // Create a single RCL context for all operations
    // Note: Our implementation always does direct DDS discovery (daemon-free by design)
    // so --no-daemon doesn't change our behavior, but we acknowledge it for compatibility
    let context =
        RclGraphContext::new().map_err(|e| anyhow!("Failed to initialize RCL context: {}", e))?;

    // Log a note about daemon usage if the flag is explicitly set
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    // Get topic type
    let topic_type = {
        let topics_and_types = context
            .get_topic_names_and_types()
            .map_err(|e| anyhow!("Failed to get topic names and types: {}", e))?;

        topics_and_types
            .iter()
            .find(|(name, _)| name == topic_name)
            .map(|(_, type_name)| type_name.clone())
            .ok_or_else(|| {
                let daemon_status = RclGraphContext::get_daemon_status();
                anyhow!("Topic '{}' not found. [{}]", topic_name, daemon_status)
            })?
    };

    // Get publisher count
    let publisher_count = context
        .count_publishers(topic_name)
        .map_err(|e| anyhow!("Failed to count publishers: {}", e))?;

    // Get subscriber count
    let subscriber_count = context
        .count_subscribers(topic_name)
        .map_err(|e| anyhow!("Failed to count subscribers: {}", e))?;

    match output_mode {
        output::OutputMode::Plain => {
            output::print_plain_section("Topic");
            output::print_plain_field("Name", topic_name);
            output::print_plain_field("Type", &topic_type);
            output::print_plain_field("Publisher count", publisher_count);
            output::print_plain_field("Subscription count", subscriber_count);
        }
        output::OutputMode::Human => {
            blocks::print_section("Topic");
            blocks::print_field("Name", topic_name.bright_cyan());
            blocks::print_field("Type", topic_type.bright_green());
            blocks::print_field(
                "Publishers",
                if publisher_count > 0 {
                    publisher_count.to_string().bright_green().to_string()
                } else {
                    publisher_count.to_string().red().to_string()
                },
            );
            blocks::print_field(
                "Subscribers",
                if subscriber_count > 0 {
                    subscriber_count.to_string().bright_green().to_string()
                } else {
                    subscriber_count.to_string().red().to_string()
                },
            );
        }
        output::OutputMode::Json => {}
    }

    let mut publishers_json = Vec::new();
    let mut subscribers_json = Vec::new();

    if verbose {
        // Get detailed publisher info
        let publishers_info = context
            .get_publishers_info(topic_name)
            .map_err(|e| anyhow!("Failed to get publishers info: {}", e))?;

        // Allow some time for any internal RCL state to settle after the first call
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Get detailed subscriber info
        let subscribers_info = context
            .get_subscribers_info(topic_name)
            .map_err(|e| anyhow!("Failed to get subscribers info: {}", e))?;

        publishers_json = publishers_info
            .iter()
            .map(|pub_info| {
                json!({
                    "node_name": pub_info.node_name,
                    "node_namespace": pub_info.node_namespace,
                    "topic_type": pub_info.topic_type,
                })
            })
            .collect();
        subscribers_json = subscribers_info
            .iter()
            .map(|sub_info| {
                json!({
                    "node_name": sub_info.node_name,
                    "node_namespace": sub_info.node_namespace,
                    "topic_type": sub_info.topic_type,
                })
            })
            .collect();

        match output_mode {
            output::OutputMode::Plain => {
                output::print_plain_section("Publishers");
                if publishers_info.is_empty() {
                    println!("<none>");
                } else {
                    for pub_info in &publishers_info {
                        println!(
                            "{}\t{}\t{}",
                            pub_info.node_name, pub_info.node_namespace, pub_info.topic_type
                        );
                    }
                }
                println!();
                output::print_plain_section("Subscribers");
                if subscribers_info.is_empty() {
                    println!("<none>");
                } else {
                    for sub_info in &subscribers_info {
                        println!(
                            "{}\t{}\t{}",
                            sub_info.node_name, sub_info.node_namespace, sub_info.topic_type
                        );
                    }
                }
            }
            output::OutputMode::Human => {
                println!();
                blocks::print_section("Publishers");
                if publishers_info.is_empty() {
                    println!("{}", "<none>".bright_black());
                } else {
                    let rows = publishers_info
                        .iter()
                        .map(|pub_info| {
                            vec![
                                pub_info.node_name.bright_white().to_string(),
                                pub_info.node_namespace.bright_black().to_string(),
                                pub_info.topic_type.bright_green().to_string(),
                            ]
                        })
                        .collect();
                    table::print_table(&["Node", "Namespace", "Type"], rows);
                }

                println!();
                blocks::print_section("Subscribers");
                if subscribers_info.is_empty() {
                    println!("{}", "<none>".bright_black());
                } else {
                    let rows = subscribers_info
                        .iter()
                        .map(|sub_info| {
                            vec![
                                sub_info.node_name.bright_white().to_string(),
                                sub_info.node_namespace.bright_black().to_string(),
                                sub_info.topic_type.bright_green().to_string(),
                            ]
                        })
                        .collect();
                    table::print_table(&["Node", "Namespace", "Type"], rows);
                }
            }
            output::OutputMode::Json => {}
        }
    }

    if output_mode == output::OutputMode::Json {
        output::print_json(&json!({
            "name": topic_name,
            "type": topic_type,
            "publisher_count": publisher_count,
            "subscriber_count": subscriber_count,
            "publishers": publishers_json,
            "subscribers": subscribers_json,
        }))?;
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
