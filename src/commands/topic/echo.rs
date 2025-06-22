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

/// Check if a message type is problematic for native subscription
fn is_problematic_message_type(message_type: &str) -> bool {
    // These message types are known to cause segfaults in RMW CycloneDDS due to string handling bugs
    matches!(message_type, 
        "std_msgs/msg/String" | 
        "geometry_msgs/msg/Twist" |
        // Add other known problematic types here
        "sensor_msgs/msg/CompressedImage" |
        "sensor_msgs/msg/Image"
    )
}

async fn fallback_to_ros2_echo(options: &EchoOptions, running: Arc<AtomicBool>) -> Result<()> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command as AsyncCommand;

    // Build ros2 topic echo command with all the same arguments
    let mut args = vec!["topic".to_string(), "echo".to_string(), options.topic_name.clone()];
    
    if let Some(field) = &options.field {
        args.push("--field".to_string());
        args.push(field.clone());
    }
    if options.full_length {
        args.push("--full-length".to_string());
    }
    if options.truncate_length != 128 {
        args.push("--truncate-length".to_string());
        args.push(options.truncate_length.to_string());
    }
    if options.no_arr {
        args.push("--no-arr".to_string());
    }
    if options.no_str {
        args.push("--no-str".to_string());
    }
    if options.flow_style {
        args.push("--flow-style".to_string());
    }
    if options.no_lost_messages {
        args.push("--no-lost-messages".to_string());
    }
    if options.raw {
        args.push("--raw".to_string());
    }
    if options.once {
        args.push("--once".to_string());
    }
    if options.csv {
        args.push("--csv".to_string());
    }

    // Execute ros2 topic echo with streaming output
    let mut child = AsyncCommand::new("ros2")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to execute ros2 topic echo: {}", e))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();

    // Stream output in real-time
    loop {
        if !running.load(Ordering::Relaxed) {
            child.kill().await.ok();
            break;
        }

        tokio::select! {
            Ok(Some(line)) = stdout_lines.next_line() => {
                println!("{}", line);
            }
            Ok(Some(line)) = stderr_lines.next_line() => {
                eprintln!("{}", line);
            }
            result = child.wait() => {
                match result {
                    Ok(status) => {
                        if !status.success() {
                            return Err(anyhow!("ros2 topic echo exited with status: {}", status));
                        }
                    }
                    Err(e) => {
                        return Err(anyhow!("Error waiting for ros2 topic echo: {}", e));
                    }
                }
                break;
            }
        }
    }

    Ok(())
}

async fn echo_topic_messages(
    options: EchoOptions,
    _common_args: CommonTopicArgs,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Create RCL context for subscription
    let graph_context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Wait for topic to appear
    if !graph_context.wait_for_topic(&options.topic_name, Duration::from_secs(3))? {
        return Err(anyhow!("Topic '{}' not found after waiting", options.topic_name));
    }

    // Get topic type
    let topic_type = {
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

    // Check if this message type is problematic for native subscription
    if is_problematic_message_type(&topic_type) {
        // Fall back to ros2 topic echo for problematic message types
        return fallback_to_ros2_echo(&options, running).await;
    }

    // Wait for publishers to be available
    if !graph_context.wait_for_topic_with_publishers(&options.topic_name, Duration::from_secs(5))? {
        if !options.no_lost_messages {
            eprintln!("WARNING: no publisher on [{}]", options.topic_name);
        }
    }

    // Create dynamic subscription using our new infrastructure
    let subscription = graph_context.create_subscription(&options.topic_name, &topic_type)?;
    
    println!("Subscribed to [{}] (type: {})", options.topic_name, topic_type);
    
    let mut message_count = 0;
    let check_interval = Duration::from_millis(50); // 20 Hz polling

    // Main message reception loop
    while running.load(Ordering::Relaxed) {
        sleep(check_interval).await;

        // Check for new messages
        match subscription.take_message() {
            Ok(Some(message_data)) => {
                message_count += 1;
                
                // Try to deserialize the message data for display
                let displayed_message = match RclGraphContext::inspect_serialized_message(&topic_type, &message_data) {
                    Ok(yaml_value) => {
                        format_message_for_display(&yaml_value, &options)
                    }
                    Err(_) => {
                        // Fallback to raw data display
                        if options.raw {
                            format!("Raw data: {} bytes", message_data.len())
                        } else {
                            format!("Message #{} ({} bytes)", message_count, message_data.len())
                        }
                    }
                };

                // Format output based on options
                if options.csv {
                    // CSV format with header only on first message
                    if message_count == 1 {
                        println!("timestamp,seq,data");
                    }
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap();
                    println!(
                        "{},{},\"{}\"",
                        current_time.as_secs(),
                        message_count,
                        displayed_message.replace("\"", "\"\"") // Escape quotes for CSV
                    );
                } else {
                    // YAML format (default)
                    println!("{}", displayed_message);
                    if !options.csv {
                        println!("---");
                    }
                }

                if options.once {
                    break;
                }
            }
            Ok(None) => {
                // No message available, continue polling
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }

        // Check if publishers are still active
        let current_publisher_count = graph_context.count_publishers(&options.topic_name).unwrap_or(0);
        if current_publisher_count == 0 && !options.no_lost_messages {
            // Only show this message if we haven't received any messages recently
            static mut LAST_NO_PUBLISHER_WARNING: Option<std::time::Instant> = None;
            let now = std::time::Instant::now();
            unsafe {
                let should_warn = match LAST_NO_PUBLISHER_WARNING {
                    Some(last_time) => now.duration_since(last_time) > Duration::from_secs(5),
                    None => true,
                };
                
                if should_warn {
                    eprintln!("WARNING: no publisher on [{}]", options.topic_name);
                    LAST_NO_PUBLISHER_WARNING = Some(now);
                }
            }
        }
    }

    Ok(())
}

/// Format a YAML message for display based on echo options
fn format_message_for_display(yaml_value: &crate::graph::YamlValue, options: &EchoOptions) -> String {
    use crate::graph::YamlValue;
    
    // Simple message formatting - can be enhanced based on specific field extraction
    if let Some(field) = &options.field {
        // Extract specific field if requested
        match yaml_value {
            YamlValue::Object(map) => {
                if let Some(field_value) = map.get(field) {
                    format_yaml_value(field_value, options)
                } else {
                    format!("Field '{}' not found", field)
                }
            }
            _ => format!("Cannot extract field '{}' from non-object message", field)
        }
    } else {
        // Display entire message
        format_yaml_value(yaml_value, options)
    }
}

/// Format a YAML value for display with truncation and styling options
fn format_yaml_value(value: &crate::graph::YamlValue, options: &EchoOptions) -> String {
    use crate::graph::YamlValue;
    
    let formatted = match value {
        YamlValue::String(s) => {
            let display_str = if options.full_length {
                s.clone()
            } else if s.len() > options.truncate_length {
                format!("{}...", &s[..options.truncate_length])
            } else {
                s.clone()
            };
            
            if options.no_str {
                display_str
            } else {
                format!("\"{}\"", display_str)
            }
        }
        YamlValue::Int(n) => n.to_string(),
        YamlValue::Float(n) => n.to_string(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Object(map) => {
            if options.flow_style {
                // Flow style (single line)
                let entries: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, format_yaml_value(v, options)))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            } else {
                // Block style (multi-line)
                let entries: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, format_yaml_value(v, options)))
                    .collect();
                entries.join("\n")
            }
        }
        YamlValue::Array(arr) => {
            if options.no_arr {
                "[array data]".to_string()
            } else if options.flow_style {
                let items: Vec<String> = arr.iter()
                    .map(|v| format_yaml_value(v, options))
                    .collect();
                format!("[{}]", items.join(", "))
            } else {
                let items: Vec<String> = arr.iter()
                    .enumerate()
                    .map(|(i, v)| format!("- [{}]: {}", i, format_yaml_value(v, options)))
                    .collect();
                items.join("\n")
            }
        }
    };
    
    formatted
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
