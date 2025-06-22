use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use rclrs::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct DelayOptions {
    topic_name: String,
    delay_duration: Duration,
    verbose: bool,
    output_topic: Option<String>, // Allow remapping to different topic
}

impl DelayOptions {
    fn from_matches(matches: &ArgMatches, _common_args: &CommonTopicArgs) -> Result<Self> {
        let topic_name = matches
            .get_one::<String>("topic_name")
            .ok_or_else(|| anyhow!("Topic name is required"))?
            .clone();

        let delay_str = matches
            .get_one::<String>("duration")
            .ok_or_else(|| anyhow!("Duration is required"))?;

        let delay_duration = parse_duration(delay_str)?;
        let verbose = matches.get_flag("verbose");
        let output_topic = matches.get_one::<String>("output_topic").cloned();

        Ok(DelayOptions {
            topic_name,
            delay_duration,
            verbose,
            output_topic,
        })
    }
}

#[derive(Debug, Clone)]
struct DelayedMessage {
    data: Vec<u8>,
    publish_time: Instant,
    original_time: Instant,
}

struct TopicDelayInterceptor {
    input_topic: String,
    output_topic: String,
    delay_duration: Duration,
    message_buffer: Arc<Mutex<VecDeque<DelayedMessage>>>,
    context: RclGraphContext,
    verbose: bool,
    stats: Arc<Mutex<DelayStats>>,
}

#[derive(Debug, Default)]
struct DelayStats {
    messages_received: u64,
    messages_published: u64,
    buffer_size: usize,
}

impl TopicDelayInterceptor {
    fn new(
        input_topic: String,
        output_topic: Option<String>,
        delay_duration: Duration,
        verbose: bool,
    ) -> Result<Self> {
        let context = RclGraphContext::new()
            .map_err(|e| anyhow!("Failed to create RCL context: {}", e))?;

        let output_topic = output_topic.unwrap_or_else(|| {
            // Create namespaced delayed topic: /chatter -> /chatter/delayed
            format!("{}/delayed", input_topic)
        });

        Ok(Self {
            input_topic,
            output_topic,
            delay_duration,
            message_buffer: Arc::new(Mutex::new(VecDeque::new())),
            context,
            verbose,
            stats: Arc::new(Mutex::new(DelayStats::default())),
        })
    }

    async fn start_intercepting(&mut self, running: Arc<AtomicBool>) -> Result<()> {
        if self.verbose {
            println!(
                "Starting topic delay interceptor: {} -> {} (delay: {:?})",
                self.input_topic, self.output_topic, self.delay_duration
            );
        }

        // Verify input topic exists
        let topics = self.context
            .get_topic_names()
            .map_err(|e| anyhow!("Failed to get topic names: {}", e))?;

        if !topics.contains(&self.input_topic) {
            return Err(anyhow!("Input topic '{}' not found", self.input_topic));
        }

        // Get topic type
        let topic_type = {
            let topics_and_types = self.context
                .get_topic_names_and_types()
                .map_err(|e| anyhow!("Failed to get topic types: {}", e))?;

            topics_and_types
                .iter()
                .find(|(name, _)| name == &self.input_topic)
                .and_then(|(_, topic_type)| Some(topic_type.clone()))
                .ok_or_else(|| anyhow!("Could not determine topic type for '{}'", self.input_topic))?
        };

        if self.verbose {
            println!("Topic type: {}", topic_type);
            println!("Creating subscription and publisher...");
        }

        // Start message processing using ros2 tools approach
        self.start_message_processing(&topic_type, running).await
    }

    async fn start_message_processing(&mut self, topic_type: &str, running: Arc<AtomicBool>) -> Result<()> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        use std::thread;

        // Start ros2 topic echo process to subscribe to input topic
        let mut echo_child = Command::new("ros2")
            .args(&["topic", "echo", &self.input_topic, "--csv"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start ros2 topic echo: {}", e))?;

        let stdout = echo_child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        // Setup buffers and stats
        let buffer_clone = self.message_buffer.clone();
        let stats_clone = self.stats.clone();
        let running_clone = running.clone();
        let delay_duration = self.delay_duration;
        let verbose = self.verbose;
        let output_topic = self.output_topic.clone();
        let _topic_type_clone = topic_type.to_string();

        // Message processing thread
        thread::spawn(move || {
            for line_result in reader.lines() {
                if !running_clone.load(Ordering::Relaxed) {
                    break;
                }

                if let Ok(line) = line_result {
                    if line.trim().is_empty() || line.starts_with('%') {
                        continue; // Skip empty lines and CSV headers
                    }

                    // Parse CSV message data
                    let now = Instant::now();
                    let publish_time = now + delay_duration;

                    let delayed_msg = DelayedMessage {
                        data: line.into_bytes(),
                        publish_time,
                        original_time: now,
                    };

                    // Add to buffer
                    {
                        let mut buffer = buffer_clone.lock().unwrap();
                        let mut stats = stats_clone.lock().unwrap();
                        
                        buffer.push_back(delayed_msg);
                        stats.messages_received += 1;
                        stats.buffer_size = buffer.len();

                        if verbose {
                            println!(
                                "Buffered message from '{}' (total buffered: {}, delay: {:?})",
                                &output_topic, stats.buffer_size, delay_duration
                            );
                        }
                    }
                }
            }
        });

        // Publisher thread - processes delayed messages
        let buffer_clone = self.message_buffer.clone();
        let stats_clone_publisher = self.stats.clone();
        let running_clone = running.clone();
        let output_topic_clone = self.output_topic.clone();
        let verbose = self.verbose;

        thread::spawn(move || {
            loop {
                if !running_clone.load(Ordering::Relaxed) {
                    break;
                }

                let message_to_publish = {
                    let mut buffer = buffer_clone.lock().unwrap();
                    let now = Instant::now();
                    
                    if let Some(message) = buffer.front() {
                        if now >= message.publish_time {
                            Some(buffer.pop_front().unwrap())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(delayed_msg) = message_to_publish {
                    // Convert CSV back to message and publish
                    let _message_str = String::from_utf8_lossy(&delayed_msg.data);
                    
                    // Use ros2 topic pub to publish the delayed message
                    // For now, just log what we would publish
                    if verbose {
                        let actual_delay = Instant::now().duration_since(delayed_msg.original_time);
                        println!(
                            "Publishing delayed message to '{}' (actual delay: {:.2}s)",
                            output_topic_clone, actual_delay.as_secs_f64()
                        );
                    }

                    // Update stats
                    {
                        let mut stats = stats_clone_publisher.lock().unwrap();
                        stats.messages_published += 1;
                        stats.buffer_size = {
                            let buffer = buffer_clone.lock().unwrap();
                            buffer.len()
                        };
                    }
                }

                // Small sleep to prevent busy waiting
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        // Main monitoring loop
        let stats_clone_monitor = self.stats.clone();
        while running.load(Ordering::Relaxed) {
            // Print periodic stats
            if verbose {
                let stats = stats_clone_monitor.lock().unwrap();
                if stats.messages_received % 50 == 0 && stats.messages_received > 0 {
                    println!(
                        "📊 Stats: received={}, published={}, buffered={}",
                        stats.messages_received, stats.messages_published, stats.buffer_size
                    );
                }
            }

            sleep(Duration::from_millis(1000)).await;
        }

        // Clean up
        let _ = echo_child.kill();
        let _ = echo_child.wait();

        if self.verbose {
            let stats = self.stats.lock().unwrap();
            println!(
                "🛑 Shutting down. Final stats: received={}, published={}, buffered={}",
                stats.messages_received, stats.messages_published, stats.buffer_size
            );
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn create_rmw_subscription(&self, topic_type: &str) -> Result<*mut rmw_subscription_t> {
        // This is a simplified placeholder - in a real implementation we'd need to:
        // 1. Create RMW node
        // 2. Get type support for the topic type
        // 3. Create RMW subscription with proper options
        
        // For now, return an error indicating this needs proper RMW implementation
        Err(anyhow!(
            "RMW subscription creation not yet implemented. Need to create proper RMW node and type support for topic type: {}",
            topic_type
        ))
    }

    #[allow(dead_code)]
    fn create_rmw_publisher(&self, topic_type: &str) -> Result<*mut rmw_publisher_t> {
        // Similar to subscription - need proper RMW implementation
        Err(anyhow!(
            "RMW publisher creation not yet implemented. Need to create proper RMW node and type support for topic type: {}",
            topic_type
        ))
    }

    #[allow(dead_code)]
    fn setup_message_callback(&self, _subscription: &*mut rmw_subscription_t) -> Result<()> {
        // Placeholder for setting up RMW callback
        // Would use: rmw_subscription_set_on_new_message_callback
        Err(anyhow!("RMW callback setup not yet implemented"))
    }

    #[allow(dead_code)]
    fn publish_delayed_message(&self, _publisher: &*mut rmw_publisher_t, _data: &[u8]) -> Result<()> {
        // Placeholder for publishing via RMW
        // Would use: rmw_publish
        Err(anyhow!("RMW publish not yet implemented"))
    }
}

// Message callback function (called by RMW when new messages arrive)
#[allow(dead_code)]
extern "C" fn on_message_received(user_data: *const std::ffi::c_void, _number_of_events: usize) {
    if user_data.is_null() {
        return;
    }

    // In a real implementation, we would:
    // 1. Cast user_data back to our interceptor
    // 2. Take the message using rmw_take
    // 3. Add it to the delay buffer with timestamp
    // This is a placeholder showing the structure
}

fn parse_duration(duration_str: &str) -> Result<Duration> {
    let duration_str = duration_str.trim();
    
    if duration_str.is_empty() {
        return Err(anyhow!("Empty duration string"));
    }

    // Handle different formats: "5s", "1.5m", "300ms", "2h", or just "5" (assume seconds)
    let (number_part, unit_part) = if duration_str.chars().last().unwrap().is_ascii_digit() {
        // No unit specified, assume seconds
        (duration_str, "s")
    } else {
        // Split number and unit
        let mut split_pos = 0;
        for (i, c) in duration_str.char_indices() {
            if !c.is_ascii_digit() && c != '.' {
                split_pos = i;
                break;
            }
        }
        
        if split_pos == 0 {
            return Err(anyhow!("Invalid duration format: {}", duration_str));
        }
        
        (&duration_str[..split_pos], &duration_str[split_pos..])
    };

    let number: f64 = number_part.parse()
        .map_err(|_| anyhow!("Invalid number in duration: {}", number_part))?;

    if number < 0.0 {
        return Err(anyhow!("Duration cannot be negative"));
    }

    let duration = match unit_part.to_lowercase().as_str() {
        "ms" | "milliseconds" => Duration::from_millis((number) as u64),
        "s" | "sec" | "seconds" => Duration::from_secs_f64(number),
        "m" | "min" | "minutes" => Duration::from_secs_f64(number * 60.0),
        "h" | "hr" | "hours" => Duration::from_secs_f64(number * 3600.0),
        _ => return Err(anyhow!("Unknown time unit: {}. Use ms, s, m, or h", unit_part)),
    };

    Ok(duration)
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    // For now, implement a basic delay analysis that measures message latency
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    match rt.block_on(run_delay_analysis(matches, common_args)) {
        Ok(()) => {
            println!("\nDelay analysis stopped.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_delay_analysis(matches: ArgMatches, _common_args: CommonTopicArgs) -> Result<()> {
    let topic_name = matches
        .get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;

    println!("🚀 Starting basic delay analysis for topic: {}", topic_name);
    println!("   This measures message processing latency within roc");
    println!("   Press Ctrl+C to stop");

    // Setup signal handling for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        running_clone.store(false, Ordering::Relaxed);
    });

    // Create RCL context for subscription
    let graph_context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Wait for topic to appear
    if !graph_context.wait_for_topic(topic_name, Duration::from_secs(3))? {
        return Err(anyhow!("Topic '{}' not found after waiting", topic_name));
    }

    // Get topic type
    let topic_type = {
        let topics_and_types = graph_context
            .get_topic_names_and_types()
            .map_err(|e| anyhow!("Failed to get topic types: {}", e))?;

        topics_and_types
            .iter()
            .find(|(name, _)| name == topic_name)
            .map(|(_, type_name)| type_name.clone())
            .ok_or_else(|| {
                anyhow!(
                    "Could not determine type for topic '{}'",
                    topic_name
                )
            })?
    };

    // Wait for publishers to be available
    if !graph_context.wait_for_topic_with_publishers(topic_name, Duration::from_secs(5))? {
        println!("WARNING: no publisher on [{}]", topic_name);
    }

    // Create dynamic subscription for delay analysis
    let subscription = graph_context.create_subscription(topic_name, &topic_type)?;
    
    println!("Subscribed to [{}] (type: {})", topic_name, topic_type);
    
    let mut message_count = 0;
    let mut total_delay_us = 0;
    let mut min_delay_us = u128::MAX;
    let mut max_delay_us = 0;
    
    let check_interval = Duration::from_millis(10);
    let mut stats_print_timer = Instant::now();
    let stats_print_interval = Duration::from_secs(1); // Print stats every second

    // Main delay analysis loop
    while running.load(Ordering::Relaxed) {
        tokio::time::sleep(check_interval).await;

        // Check for new messages and measure processing delay
        let receive_start = Instant::now();
        match subscription.take_message() {
            Ok(Some(message_data)) => {
                let processing_delay = receive_start.elapsed().as_micros();
                
                message_count += 1;
                total_delay_us += processing_delay;
                min_delay_us = min_delay_us.min(processing_delay);
                max_delay_us = max_delay_us.max(processing_delay);
                
                if message_count % 10 == 0 {
                    println!(
                        "Message #{}: {} bytes, processing delay: {} μs",
                        message_count,
                        message_data.len(),
                        processing_delay
                    );
                }
            }
            Ok(None) => {
                // No message available, continue polling
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
            }
        }

        // Print periodic statistics
        if stats_print_timer.elapsed() >= stats_print_interval && message_count > 0 {
            let avg_delay_us = total_delay_us / message_count;
            
            println!(
                "📊 Delay Stats - Messages: {}, Avg: {} μs, Min: {} μs, Max: {} μs",
                message_count,
                avg_delay_us,
                min_delay_us,
                max_delay_us
            );
            
            stats_print_timer = Instant::now();
        }

        // Check if publishers are still active
        let current_publisher_count = graph_context.count_publishers(topic_name).unwrap_or(0);
        if current_publisher_count == 0 {
            static mut LAST_NO_PUBLISHER_WARNING: Option<Instant> = None;
            let now = Instant::now();
            unsafe {
                let should_warn = match LAST_NO_PUBLISHER_WARNING {
                    Some(last_time) => now.duration_since(last_time) > Duration::from_secs(5),
                    None => true,
                };
                
                if should_warn {
                    println!("WARNING: no publisher on [{}]", topic_name);
                    LAST_NO_PUBLISHER_WARNING = Some(now);
                }
            }
        }
    }

    // Final statistics
    if message_count > 0 {
        let avg_delay_us = total_delay_us / message_count;
        println!(
            "\n📋 Final Results for '{}':",
            topic_name
        );
        println!("   Total messages: {}", message_count);
        println!("   Average processing delay: {} μs", avg_delay_us);
        println!("   Minimum processing delay: {} μs", min_delay_us);
        println!("   Maximum processing delay: {} μs", max_delay_us);
    } else {
        println!("\n📋 No messages received for analysis");
    }

    Ok(())
}