use crate::arguments::topic::CommonTopicArgs;
use crate::commands::cli::run_async_command;
use crate::graph::RclGraphContext;
use crate::ui::blocks;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use rclrs::{Context, CreateBasicExecutor, DynamicMessage, MessageTypeName};
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

struct TopicDelayInterceptor {
    input_topic: String,
    output_topic: String,
    delay_duration: Duration,
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
        let context =
            RclGraphContext::new().map_err(|e| anyhow!("Failed to create RCL context: {}", e))?;

        let output_topic = output_topic.unwrap_or_else(|| {
            // Create namespaced delayed topic: /chatter -> /chatter/delayed
            format!("{}/delayed", input_topic)
        });

        Ok(Self {
            input_topic,
            output_topic,
            delay_duration,
            context,
            verbose,
            stats: Arc::new(Mutex::new(DelayStats::default())),
        })
    }

    fn snapshot_stats(&self) -> Result<DelayStats> {
        self.stats
            .lock()
            .map(|stats| DelayStats {
                messages_received: stats.messages_received,
                messages_published: stats.messages_published,
                buffer_size: stats.buffer_size,
            })
            .map_err(|_| anyhow!("Topic delay stats state poisoned"))
    }

    async fn start_intercepting(&mut self, running: Arc<AtomicBool>) -> Result<()> {
        // Verify input topic exists
        let topics = self
            .context
            .get_topic_names()
            .map_err(|e| anyhow!("Failed to get topic names: {}", e))?;

        if !topics.contains(&self.input_topic) {
            return Err(anyhow!("Input topic '{}' not found", self.input_topic));
        }

        // Get topic type
        let topic_type = {
            let topics_and_types = self
                .context
                .get_topic_names_and_types()
                .map_err(|e| anyhow!("Failed to get topic types: {}", e))?;

            topics_and_types
                .iter()
                .find(|(name, _)| name == &self.input_topic)
                .and_then(|(_, topic_type)| Some(topic_type.clone()))
                .ok_or_else(|| {
                    anyhow!("Could not determine topic type for '{}'", self.input_topic)
                })?
        };

        blocks::print_section("Topic Delay");
        blocks::print_field("Input", &self.input_topic);
        blocks::print_field("Output", &self.output_topic);
        blocks::print_field("Delay", format_duration_label(self.delay_duration));
        blocks::print_field("Type", &topic_type);
        blocks::print_field("Mode", if self.verbose { "verbose" } else { "standard" });
        println!();
        blocks::print_note("Press Ctrl+C to stop");

        self.start_message_processing(&topic_type, running).await
    }

    async fn start_message_processing(
        &mut self,
        topic_type: &str,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        // Create a dynamic subscription (native).
        // Note: this spins its own executor thread internally.
        let subscription = self
            .context
            .create_subscription(&self.input_topic, topic_type)?;

        // Create a dynamic publisher for the output topic.
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node = executor.create_node("roc_topic_delay")?;
        let msg_type: MessageTypeName = topic_type
            .try_into()
            .map_err(|e| anyhow!("Invalid message type '{}': {}", topic_type, e))?;
        let publisher = node.create_dynamic_publisher(msg_type, self.output_topic.as_str())?;

        // Keep only the latest message; on each tick publish whatever the latest is.
        // This matches a "sample and hold" delay/filter, not a true FIFO delay queue.
        let latest_msg: Arc<Mutex<Option<(DynamicMessage, Instant)>>> = Arc::new(Mutex::new(None));

        let latest_clone = latest_msg.clone();
        let stats_clone = self.stats.clone();
        let running_clone = running.clone();
        let verbose = self.verbose;
        let input_topic = self.input_topic.clone();

        tokio::spawn(async move {
            let poll_interval = Duration::from_millis(5);
            while running_clone.load(Ordering::Relaxed) {
                tokio::time::sleep(poll_interval).await;

                match subscription.take_message() {
                    Ok(Some(received)) => {
                        let now = Instant::now();
                        let Ok(mut latest) = latest_clone.lock() else {
                            blocks::eprint_warning("Topic delay latest-message state poisoned");
                            break;
                        };
                        *latest = Some((received.message, now));

                        let Ok(mut stats) = stats_clone.lock() else {
                            blocks::eprint_warning("Topic delay stats state poisoned");
                            break;
                        };
                        stats.messages_received += 1;
                        stats.buffer_size = 1;

                        if verbose {
                            blocks::print_status("Received", &[("topic", input_topic.clone())]);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        blocks::eprint_warning(&format!("Error receiving message: {e}"));
                        break;
                    }
                }
            }
        });

        // Publisher loop: every delay_duration, publish the latest received message (if any).
        let output_topic = self.output_topic.clone();
        let stats_clone_publisher = self.stats.clone();
        while running.load(Ordering::Relaxed) {
            sleep(self.delay_duration).await;

            let latest = latest_msg
                .lock()
                .map_err(|_| anyhow!("Topic delay latest-message state poisoned"))?
                .take();
            let Some((msg, received_at)) = latest else {
                continue;
            };

            publisher.publish(msg)?;

            if self.verbose {
                let held_for = Instant::now().duration_since(received_at);
                blocks::print_status(
                    "Published",
                    &[
                        ("topic", output_topic.clone()),
                        ("held", format!("{:.3} s", held_for.as_secs_f64())),
                    ],
                );
            }

            let mut stats = stats_clone_publisher
                .lock()
                .map_err(|_| anyhow!("Topic delay stats state poisoned"))?;
            stats.messages_published += 1;
            stats.buffer_size = 0;
        }

        Ok(())
    }
}

fn format_duration_label(duration: Duration) -> String {
    if duration.as_secs() > 0 && duration.subsec_nanos() == 0 {
        format!("{} s", duration.as_secs())
    } else if duration.as_millis() > 0 {
        format!("{:.3} s", duration.as_secs_f64())
    } else {
        format!("{} ns", duration.as_nanos())
    }
}

fn parse_duration(duration_str: &str) -> Result<Duration> {
    let duration_str = duration_str.trim();

    if duration_str.is_empty() {
        return Err(anyhow!("Empty duration string"));
    }

    // Handle different formats: "5s", "1.5m", "300ms", "2h", or just "5" (assume seconds)
    let Some(last_char) = duration_str.chars().next_back() else {
        return Err(anyhow!("Empty duration string"));
    };

    let (number_part, unit_part) = if last_char.is_ascii_digit() {
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

    let number: f64 = number_part
        .parse()
        .map_err(|_| anyhow!("Invalid number in duration: {}", number_part))?;

    if number < 0.0 {
        return Err(anyhow!("Duration cannot be negative"));
    }

    let duration = match unit_part.to_lowercase().as_str() {
        "ms" | "milliseconds" => Duration::from_millis((number) as u64),
        "s" | "sec" | "seconds" => Duration::from_secs_f64(number),
        "m" | "min" | "minutes" => Duration::from_secs_f64(number * 60.0),
        "h" | "hr" | "hours" => Duration::from_secs_f64(number * 3600.0),
        _ => {
            return Err(anyhow!(
                "Unknown time unit: {}. Use ms, s, m, or h",
                unit_part
            ));
        }
    };

    Ok(duration)
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    run_async_command(run_command(matches, common_args));
}

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let options = DelayOptions::from_matches(&matches, &common_args)?;

    // Setup signal handling for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            blocks::eprint_warning(&format!("Failed to listen for ctrl+c: {error}"));
            return;
        }
        running_clone.store(false, Ordering::Relaxed);
    });

    let mut interceptor = TopicDelayInterceptor::new(
        options.topic_name,
        options.output_topic,
        options.delay_duration,
        options.verbose,
    )?;

    interceptor.start_intercepting(running).await?;

    let stats = interceptor.snapshot_stats()?;
    println!();
    blocks::print_section("Delay Summary");
    blocks::print_field("Received", stats.messages_received);
    blocks::print_field("Published", stats.messages_published);
    blocks::print_field("Buffered", stats.buffer_size);

    Ok(())
}
