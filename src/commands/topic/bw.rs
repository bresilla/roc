use crate::arguments::topic::CommonTopicArgs;
use crate::commands::cli::run_async_command;
use crate::graph::RclGraphContext;
use crate::ui::blocks;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Topic Bandwidth Implementation
//
// This implementation monitors topic bandwidth by:
// 1. Tracking message rate and size estimates
// 2. Calculating rolling averages over a time window
// 3. Displaying bandwidth statistics in real-time
// 4. Supporting the same options as ros2 topic bw

const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

struct BandwidthCalculator {
    message_sizes: VecDeque<(i64, usize)>,
    window_duration_ns: i64,
    total_messages: usize,
    total_bytes: usize,
    start_time_ns: Option<i64>,
}

impl BandwidthCalculator {
    fn new(window_duration: Duration) -> Self {
        Self {
            message_sizes: VecDeque::new(),
            window_duration_ns: window_duration.as_nanos().min(i64::MAX as u128) as i64,
            total_messages: 0,
            total_bytes: 0,
            start_time_ns: None,
        }
    }

    fn add_message(&mut self, timestamp_ns: i64, message_size: usize) -> bool {
        let reset = matches!(self.message_sizes.back(), Some((last, _)) if timestamp_ns < *last);

        if reset {
            self.message_sizes.clear();
            self.total_messages = 0;
            self.total_bytes = 0;
            self.start_time_ns = Some(timestamp_ns);
        } else if self.start_time_ns.is_none() {
            self.start_time_ns = Some(timestamp_ns);
        }

        self.message_sizes.push_back((timestamp_ns, message_size));
        self.total_messages += 1;
        self.total_bytes += message_size;

        // Remove old entries outside the window
        let cutoff_time = timestamp_ns.saturating_sub(self.window_duration_ns);
        while let Some((time, _)) = self.message_sizes.front() {
            if *time < cutoff_time {
                self.message_sizes.pop_front();
            } else {
                break;
            }
        }

        reset
    }

    fn get_current_bandwidth(&self) -> f64 {
        if self.message_sizes.len() < 2 {
            return 0.0;
        }

        let total_size: usize = self.message_sizes.iter().map(|(_, size)| size).sum();
        let time_span_ns = match (self.message_sizes.front(), self.message_sizes.back()) {
            (Some((first, _)), Some((last, _))) => last - first,
            _ => return 0.0,
        };

        if time_span_ns <= 0 {
            return 0.0;
        }

        total_size as f64 * NANOS_PER_SECOND / time_span_ns as f64
    }

    fn get_average_bandwidth(&self) -> f64 {
        let Some(start_time_ns) = self.start_time_ns else {
            return 0.0;
        };
        let Some((last_timestamp_ns, _)) = self.message_sizes.back() else {
            return 0.0;
        };

        let total_time_ns = last_timestamp_ns - start_time_ns;
        if total_time_ns <= 0 {
            return 0.0;
        }

        self.total_bytes as f64 * NANOS_PER_SECOND / total_time_ns as f64
    }

    fn get_message_count(&self) -> usize {
        self.total_messages
    }

    fn get_window_size(&self) -> usize {
        self.message_sizes.len()
    }

    fn get_bandwidth_samples(&self) -> Vec<f64> {
        self.message_sizes
            .iter()
            .zip(self.message_sizes.iter().skip(1))
            .map(|((t1, s1), (t2, _))| {
                let time_diff_ns = *t2 - *t1;
                if time_diff_ns > 0 {
                    *s1 as f64 * NANOS_PER_SECOND / time_diff_ns as f64
                } else {
                    0.0
                }
            })
            .filter(|bandwidth| *bandwidth > 0.0)
            .collect()
    }
}

fn topic_bandwidth_clock_label(use_sim_time: bool) -> &'static str {
    if use_sim_time {
        "ros"
    } else {
        "system"
    }
}

fn topic_bandwidth_timestamp_ns(graph_context: &RclGraphContext) -> Option<i64> {
    let timestamp_ns = graph_context.node().get_clock().now().nsec;
    (timestamp_ns > 0).then_some(timestamp_ns)
}

async fn monitor_topic_bandwidth(
    topic_name: &str,
    window_size: Duration,
    spin_time: Option<&str>,
    use_sim_time: bool,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Create RCL context for subscription
    let graph_context = RclGraphContext::new_with_options(spin_time, use_sim_time)
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
            .ok_or_else(|| anyhow!("Could not determine type for topic '{}'", topic_name))?
    };

    // Wait for publishers to be available
    if !graph_context.wait_for_topic_with_publishers(topic_name, Duration::from_secs(5))? {
        blocks::eprint_warning(&format!("No publisher on [{topic_name}]"));
    }

    // Create dynamic subscription for real bandwidth monitoring
    let subscription = graph_context.create_subscription(topic_name, &topic_type)?;

    blocks::print_section("Topic Bandwidth Monitor");
    blocks::print_field("Topic", topic_name);
    blocks::print_field("Type", &topic_type);
    blocks::print_field("Window", format!("{:.2} s", window_size.as_secs_f64()));
    blocks::print_field("Clock", topic_bandwidth_clock_label(use_sim_time));
    println!();

    let bandwidth_calc = Arc::new(Mutex::new(BandwidthCalculator::new(window_size)));
    let bandwidth_calc_clone = Arc::clone(&bandwidth_calc);

    let check_interval = Duration::from_millis(10); // High frequency polling for accurate bandwidth measurement
    let mut stats_print_timer = Instant::now();
    let stats_print_interval = Duration::from_secs(1);
    let mut last_no_publisher_warning: Option<Instant> = None;
    let mut waiting_for_clock_warned = false;

    blocks::print_note("Press Ctrl+C to stop");

    // Main monitoring loop with real message reception
    while running.load(Ordering::Relaxed) {
        sleep(check_interval).await;

        // Check for new messages
        match subscription.take_message() {
            Ok(Some(received)) => {
                let Some(current_time_ns) = topic_bandwidth_timestamp_ns(&graph_context) else {
                    if use_sim_time && !waiting_for_clock_warned {
                        blocks::eprint_note("Waiting for /clock before measuring topic bandwidth");
                        waiting_for_clock_warned = true;
                    }
                    continue;
                };
                waiting_for_clock_warned = false;
                let message_size = format!("{:?}", received.message.view()).len();

                let mut calc = bandwidth_calc_clone
                    .lock()
                    .map_err(|_| anyhow!("Bandwidth calculator state poisoned"))?;
                if calc.add_message(current_time_ns, message_size) {
                    blocks::eprint_warning(
                        "Detected time moving backwards; resetting bandwidth statistics",
                    );
                }
            }
            Ok(None) => {
                // No message available, continue polling
            }
            Err(_e) => {
                // Error receiving message - continue but don't record
            }
        }

        // Print statistics periodically
        if stats_print_timer.elapsed() >= stats_print_interval {
            let calc = bandwidth_calc_clone
                .lock()
                .map_err(|_| anyhow!("Bandwidth calculator state poisoned"))?;
            let current_bw = calc.get_current_bandwidth();
            let average_bw = calc.get_average_bandwidth();
            let msg_count = calc.get_message_count();

            if msg_count > 0 {
                // Calculate min/max/mean bandwidth from recent measurements
                let bandwidth_samples = calc.get_bandwidth_samples();

                let (min_bw, max_bw) = if !bandwidth_samples.is_empty() {
                    let min_bw = bandwidth_samples
                        .iter()
                        .fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_bw = bandwidth_samples.iter().fold(0.0f64, |a, &b| a.max(b));
                    (min_bw, max_bw)
                } else {
                    (current_bw, current_bw)
                };

                // Format bandwidth output like ros2 topic bw
                blocks::print_status(
                    "Bandwidth",
                    &[
                        ("avg", format!("{current_bw:.2} B/s")),
                        ("mean", format!("{average_bw:.2} B/s")),
                        ("min", format!("{min_bw:.2} B/s")),
                        ("max", format!("{max_bw:.2} B/s")),
                        ("window", calc.get_window_size().to_string()),
                        ("messages", msg_count.to_string()),
                    ],
                );
            }

            stats_print_timer = Instant::now();
        }

        // Check if publishers are still active
        let current_publisher_count = graph_context.count_publishers(topic_name).unwrap_or(0);
        if current_publisher_count == 0 {
            // Don't spam the warning - the bandwidth display already shows no messages
            let now = Instant::now();
            let should_warn = match last_no_publisher_warning {
                Some(last_time) => now.duration_since(last_time) > Duration::from_secs(5),
                None => true,
            };

            if should_warn {
                blocks::eprint_warning(&format!("No publishers found for topic '{topic_name}'"));
                last_no_publisher_warning = Some(now);
            }
        }
    }

    let calc = bandwidth_calc
        .lock()
        .map_err(|_| anyhow!("Bandwidth calculator state poisoned"))?;
    println!();
    blocks::print_section("Bandwidth Summary");
    blocks::print_field("Topic", topic_name);
    blocks::print_field("Messages", calc.get_message_count());
    blocks::print_field(
        "Average Bandwidth",
        format!("{:.2} B/s", calc.get_average_bandwidth()),
    );

    Ok(())
}

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let topic_name = matches
        .get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;

    let window_size = matches
        .get_one::<String>("window")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    if window_size == 0 {
        return Err(anyhow!("--window must be greater than 0"));
    }

    let window_duration = Duration::from_millis(window_size * 10); // Convert to reasonable duration

    // Handle common arguments
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    if common_args.use_sim_time {
        blocks::eprint_note("Using simulation time for bandwidth calculations");
    }

    if let Some(ref spin_time_value) = common_args.spin_time {
        blocks::eprint_note(&format!("Using spin time {spin_time_value} for discovery"));
    }

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            blocks::eprint_warning(&format!("Failed to listen for ctrl+c: {error}"));
            return;
        }
        running_clone.store(false, Ordering::Relaxed);
    });

    // Start monitoring
    monitor_topic_bandwidth(
        topic_name,
        window_duration,
        common_args.spin_time.as_deref(),
        common_args.use_sim_time,
        running,
    )
    .await
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    run_async_command(run_command(matches, common_args));
}

#[cfg(test)]
mod tests {
    use super::BandwidthCalculator;
    use std::time::Duration;

    #[test]
    fn bandwidth_calculator_uses_window_timestamps() {
        let mut calc = BandwidthCalculator::new(Duration::from_secs(10));
        calc.add_message(1_000_000_000, 100);
        calc.add_message(2_000_000_000, 200);
        calc.add_message(3_000_000_000, 300);

        assert_eq!(calc.get_message_count(), 3);
        assert!((calc.get_current_bandwidth() - 300.0).abs() < 1e-6);
        assert!((calc.get_average_bandwidth() - 300.0).abs() < 1e-6);
        assert_eq!(calc.get_window_size(), 3);
    }

    #[test]
    fn bandwidth_calculator_resets_when_time_moves_backwards() {
        let mut calc = BandwidthCalculator::new(Duration::from_secs(10));
        calc.add_message(2_000_000_000, 100);
        calc.add_message(3_000_000_000, 200);

        assert!(calc.add_message(1_000_000_000, 50));
        assert_eq!(calc.get_message_count(), 1);
        assert_eq!(calc.get_window_size(), 1);
        assert_eq!(calc.get_current_bandwidth(), 0.0);
    }
}
