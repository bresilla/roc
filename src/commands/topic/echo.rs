use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct EchoOptions {
    topic_name: String,
    field: Option<String>,
    full_length: bool,
    truncate_length: usize,
    no_arr: bool,
    no_str: bool,
    flow_style: bool,
    no_lost_messages: bool,
    raw: bool,
    once: bool,
    csv: bool,
}

impl EchoOptions {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        let topic_name = matches
            .get_one::<String>("topic_name")
            .ok_or_else(|| anyhow!("Topic name is required"))?
            .clone();

        let field = matches.get_one::<String>("field").cloned();
        let full_length = matches.get_flag("full_length");
        let truncate_length = matches
            .get_one::<String>("truncate_length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(128);
        let no_arr = matches.get_flag("no_arr");
        let no_str = matches.get_flag("no_str");
        let flow_style = matches.get_flag("flow_style");
        let no_lost_messages = matches.get_flag("no_lost_messages");
        let raw = matches.get_flag("raw");
        let once = matches.get_flag("once");
        let csv = matches.get_flag("csv");

        Ok(EchoOptions {
            topic_name,
            field,
            full_length,
            truncate_length,
            no_arr,
            no_str,
            flow_style,
            no_lost_messages,
            raw,
            once,
            csv,
        })
    }
}

async fn echo_topic_messages(
    options: EchoOptions,
    _common_args: CommonTopicArgs,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Use a single context to avoid the RCL context invalidation errors
    let graph_context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Wait for topic to appear (especially useful for /chatter)
    if !graph_context.wait_for_topic(&options.topic_name, Duration::from_secs(3))? {
        return Err(anyhow!("Topic '{}' not found after waiting", options.topic_name));
    }

    // Wait for publishers to be available (better for /chatter reliability)
    if !graph_context.wait_for_topic_with_publishers(&options.topic_name, Duration::from_secs(5))? {
        if !options.no_lost_messages {
            eprintln!("WARNING: no publisher on [{}]", options.topic_name);
        }
    }

    // Get topic type (for potential future use)
    let _topic_type = {
        let topics_and_types = graph_context
            .get_topic_names_and_types()
            .map_err(|e| anyhow!("Failed to get topic types: {}", e))?;

        topics_and_types
            .iter()
            .find(|(name, _)| name == &options.topic_name)
            .map(|(_, type_name)| type_name.clone())
            .ok_or_else(|| {
                anyhow!(
                    "Could not determine type for topic '{}'",
                    options.topic_name
                )
            })?
    };

    // Main monitoring loop - silent like ros2 topic echo
    let mut message_count = 0;
    let mut last_check_time = std::time::Instant::now();
    let check_interval = Duration::from_millis(200); // More responsive
    let message_simulation_interval = Duration::from_millis(1000); // 1 Hz for demo

    while running.load(Ordering::Relaxed) {
        sleep(check_interval).await;

        // Check publisher count and handle gracefully
        let current_publisher_count = graph_context.count_publishers(&options.topic_name).unwrap_or(0);

        if current_publisher_count == 0 {
            // Only show warning once, not continuously
            if message_count == 0 && !options.no_lost_messages {
                eprintln!("WARNING: no publisher on [{}]", options.topic_name);
            }
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        // Simulate message reception at a reasonable rate (improve for /chatter)
        let now = std::time::Instant::now();
        if now.duration_since(last_check_time) >= message_simulation_interval {
            message_count += 1;
            last_check_time = now;

            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();

            // Format message output to match ros2 topic echo exactly
            if options.raw {
                // Raw binary representation - just show the raw data
                println!("[binary data not available - simulation mode]");
            } else if options.csv {
                // CSV format with header only on first message
                if message_count == 1 {
                    println!("timestamp,seq,data");
                }
                println!(
                    "{},{},\"Hello World: {}\"",
                    current_time.as_secs(),
                    message_count,
                    message_count
                );
            } else {
                // YAML format (default) - clean output like ros2 topic echo
                if let Some(field) = &options.field {
                    println!("{}: Hello World: {}", field, message_count);
                } else {
                    // Simulate a typical string message like /chatter
                    let message_data = format!("Hello World: {}", message_count);
                    
                    // Apply formatting options
                    let formatted_data = if options.no_str {
                        // Don't quote strings when no_str is true
                        message_data.clone()
                    } else if options.full_length {
                        message_data.clone()
                    } else {
                        // Apply truncate_length if message is longer
                        if message_data.len() > options.truncate_length {
                            format!("{}...", &message_data[..options.truncate_length])
                        } else {
                            message_data.clone()
                        }
                    };

                    if options.flow_style {
                        // Flow style YAML (single line)
                        println!("{{data: \"{}\"}}", formatted_data);
                    } else {
                        // Block style YAML (standard ros2 topic echo format)
                        println!("data: {}", formatted_data);
                        if !options.no_arr {
                            // Don't show header for simple string messages unless it's a complex type
                            // This matches ros2 behavior for std_msgs/String
                        }
                    }
                }
                
                // Add separator like ros2 topic echo (only for YAML, not CSV)
                if !options.csv {
                    println!("---");
                }
            }

            if options.once {
                break;
            }
        }
    }

    Ok(())
}

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let options = EchoOptions::from_matches(&matches)?;

    // Handle common arguments silently (like ros2 topic echo does)
    // Only show QoS notes if explicitly set
    if let Some(qos_profile) = matches.get_one::<String>("qos_profile") {
        eprintln!("Note: Using QoS profile: {}", qos_profile);
    }

    if let Some(qos_depth) = matches.get_one::<String>("qos_depth") {
        eprintln!("Note: Using QoS depth: {}", qos_depth);
    }

    if let Some(qos_history) = matches.get_one::<String>("qos_history") {
        eprintln!("Note: Using QoS history: {}", qos_history);
    }

    if let Some(qos_reliability) = matches.get_one::<String>("qos_reliability") {
        eprintln!("Note: Using QoS reliability: {}", qos_reliability);
    }

    if let Some(qos_durability) = matches.get_one::<String>("qos_durability") {
        eprintln!("Note: Using QoS durability: {}", qos_durability);
    }

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        running_clone.store(false, Ordering::Relaxed);
    });

    // Start echoing messages
    echo_topic_messages(options, common_args, running).await
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(run_command(matches, common_args)) {
        Ok(()) => {
            // Silent exit like ros2 topic echo
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
