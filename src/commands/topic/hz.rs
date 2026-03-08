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

const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

struct RateCalculator {
    timestamps: VecDeque<i64>,
    window_size: usize,
    total_messages: usize,
    start_time_ns: Option<i64>,
}

impl RateCalculator {
    fn new(window_size: usize) -> Self {
        Self {
            timestamps: VecDeque::new(),
            window_size,
            total_messages: 0,
            start_time_ns: None,
        }
    }

    fn add_message(&mut self, timestamp_ns: i64) -> bool {
        let reset = matches!(self.timestamps.back(), Some(last) if timestamp_ns < *last);

        if reset {
            self.timestamps.clear();
            self.total_messages = 0;
            self.start_time_ns = Some(timestamp_ns);
        } else if self.start_time_ns.is_none() {
            self.start_time_ns = Some(timestamp_ns);
        }

        self.timestamps.push_back(timestamp_ns);
        self.total_messages += 1;

        // Keep only the last window_size timestamps
        while self.timestamps.len() > self.window_size {
            self.timestamps.pop_front();
        }

        reset
    }

    fn get_current_rate(&self) -> f64 {
        if self.timestamps.len() < 2 {
            return 0.0;
        }

        let time_span_ns = match (self.timestamps.front(), self.timestamps.back()) {
            (Some(first), Some(last)) => last - first,
            _ => return 0.0,
        };

        if time_span_ns <= 0 {
            return 0.0;
        }

        (self.timestamps.len() - 1) as f64 * NANOS_PER_SECOND / time_span_ns as f64
    }

    fn get_average_rate(&self) -> f64 {
        let Some(start_time_ns) = self.start_time_ns else {
            return 0.0;
        };
        let Some(last_timestamp_ns) = self.timestamps.back() else {
            return 0.0;
        };

        let total_time_ns = last_timestamp_ns - start_time_ns;
        if total_time_ns <= 0 {
            return 0.0;
        }

        self.total_messages.saturating_sub(1) as f64 * NANOS_PER_SECOND / total_time_ns as f64
    }

    fn get_total_messages(&self) -> usize {
        self.total_messages
    }

    fn get_periods(&self) -> Vec<f64> {
        self.timestamps
            .iter()
            .zip(self.timestamps.iter().skip(1))
            .map(|(t1, t2)| (*t2 - *t1) as f64 / NANOS_PER_SECOND)
            .filter(|period| *period > 0.0)
            .collect()
    }

    fn get_window_size(&self) -> usize {
        self.timestamps.len()
    }
}

fn wall_timestamp_ns(started_at: Instant) -> i64 {
    started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64
}

fn topic_rate_clock_label(use_wall_time: bool, use_sim_time: bool) -> &'static str {
    if use_wall_time {
        "wall"
    } else if use_sim_time {
        "ros"
    } else {
        "system"
    }
}

fn topic_rate_timestamp_ns(
    graph_context: &RclGraphContext,
    started_at: Instant,
    use_wall_time: bool,
) -> Option<i64> {
    if use_wall_time {
        Some(wall_timestamp_ns(started_at))
    } else {
        let timestamp_ns = graph_context.node().get_clock().now().nsec;
        (timestamp_ns > 0).then_some(timestamp_ns)
    }
}

async fn monitor_topic_rate(
    topic_name: &str,
    window_size: usize,
    spin_time: Option<&str>,
    use_sim_time: bool,
    use_wall_time: bool,
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

    // Create dynamic subscription for real message rate monitoring
    let subscription = graph_context.create_subscription(topic_name, &topic_type)?;

    blocks::print_section("Topic Rate Monitor");
    blocks::print_field("Topic", topic_name);
    blocks::print_field("Type", &topic_type);
    blocks::print_field("Window", window_size);
    blocks::print_field("Clock", topic_rate_clock_label(use_wall_time, use_sim_time));
    println!();

    let rate_calculator = Arc::new(Mutex::new(RateCalculator::new(window_size)));
    let rate_calc_clone = Arc::clone(&rate_calculator);

    let check_interval = Duration::from_millis(10); // High frequency polling for accurate rate measurement
    let started_at = Instant::now();
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
            Ok(Some(_received)) => {
                let Some(current_time_ns) =
                    topic_rate_timestamp_ns(&graph_context, started_at, use_wall_time)
                else {
                    if use_sim_time && !use_wall_time && !waiting_for_clock_warned {
                        blocks::eprint_note("Waiting for /clock before measuring topic rate");
                        waiting_for_clock_warned = true;
                    }
                    continue;
                };
                waiting_for_clock_warned = false;

                let mut calc = rate_calc_clone
                    .lock()
                    .map_err(|_| anyhow!("Rate calculator state poisoned"))?;
                if calc.add_message(current_time_ns) {
                    blocks::eprint_warning(
                        "Detected time moving backwards; resetting rate statistics",
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
            let calc = rate_calc_clone
                .lock()
                .map_err(|_| anyhow!("Rate calculator state poisoned"))?;
            let current_rate = calc.get_current_rate();
            let _average_rate = calc.get_average_rate();
            let total_msgs = calc.get_total_messages();

            if total_msgs > 0 {
                // Calculate statistics for display
                let periods = calc.get_periods();

                let (min_period, max_period, std_dev) = if !periods.is_empty() {
                    let min_p = periods.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_p = periods.iter().fold(0.0f64, |a, &b| a.max(b));

                    // Calculate standard deviation
                    let mean = periods.iter().sum::<f64>() / periods.len() as f64;
                    let variance = periods.iter().map(|p| (p - mean).powi(2)).sum::<f64>()
                        / periods.len() as f64;
                    let std_dev = variance.sqrt();

                    (min_p, max_p, std_dev)
                } else {
                    (0.0, 0.0, 0.0)
                };

                blocks::print_status(
                    "Rate",
                    &[
                        ("avg", format!("{current_rate:.3} Hz")),
                        ("min", format!("{min_period:.3} s")),
                        ("max", format!("{max_period:.3} s")),
                        ("stddev", format!("{std_dev:.3} s")),
                        ("window", calc.get_window_size().to_string()),
                        ("messages", total_msgs.to_string()),
                    ],
                );
            }

            stats_print_timer = Instant::now();
        }

        // Check if publishers are still active
        let current_publisher_count = graph_context.count_publishers(topic_name).unwrap_or(0);
        if current_publisher_count == 0 {
            // Don't spam the warning - the rate display already shows no messages
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

    let calc = rate_calculator
        .lock()
        .map_err(|_| anyhow!("Rate calculator state poisoned"))?;
    println!();
    blocks::print_section("Rate Summary");
    blocks::print_field("Topic", topic_name);
    blocks::print_field("Messages", calc.get_total_messages());
    blocks::print_field("Average Rate", format!("{:.3} Hz", calc.get_average_rate()));

    Ok(())
}

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let topic_name = matches
        .get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;

    let window_size = matches
        .get_one::<String>("window")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10000);

    if window_size == 0 {
        return Err(anyhow!("--window must be greater than 0"));
    }

    let use_wall_time = matches.get_flag("wall_time");

    // Handle common arguments
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    if common_args.use_sim_time && !use_wall_time {
        blocks::eprint_note("Using simulation time for rate calculations");
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
    monitor_topic_rate(
        topic_name,
        window_size,
        common_args.spin_time.as_deref(),
        common_args.use_sim_time,
        use_wall_time,
        running,
    )
    .await
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    run_async_command(run_command(matches, common_args));
}

#[cfg(test)]
mod tests {
    use super::RateCalculator;

    #[test]
    fn rate_calculator_uses_sample_span() {
        let mut calc = RateCalculator::new(10);
        calc.add_message(1_000_000_000);
        calc.add_message(1_500_000_000);
        calc.add_message(2_000_000_000);

        assert_eq!(calc.get_total_messages(), 3);
        assert!((calc.get_current_rate() - 2.0).abs() < 1e-6);
        assert!((calc.get_average_rate() - 2.0).abs() < 1e-6);
        assert_eq!(calc.get_window_size(), 3);
    }

    #[test]
    fn rate_calculator_resets_when_time_moves_backwards() {
        let mut calc = RateCalculator::new(10);
        calc.add_message(2_000_000_000);
        calc.add_message(3_000_000_000);

        assert!(calc.add_message(1_000_000_000));
        assert_eq!(calc.get_total_messages(), 1);
        assert_eq!(calc.get_window_size(), 1);
        assert_eq!(calc.get_current_rate(), 0.0);
    }
}
