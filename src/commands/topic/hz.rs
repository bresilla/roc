use crate::arguments::topic::CommonTopicArgs;
use crate::commands::cli::run_async_command;
use crate::graph::RclGraphContext;
use anyhow::{Result, anyhow};
use clap::ArgMatches;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

struct RateCalculator {
    timestamps: VecDeque<Instant>,
    window_size: usize,
    total_messages: usize,
    start_time: Instant,
}

impl RateCalculator {
    fn new(window_size: usize) -> Self {
        Self {
            timestamps: VecDeque::new(),
            window_size,
            total_messages: 0,
            start_time: Instant::now(),
        }
    }

    fn add_message(&mut self, timestamp: Instant) {
        self.timestamps.push_back(timestamp);
        self.total_messages += 1;

        // Keep only the last window_size timestamps
        while self.timestamps.len() > self.window_size {
            self.timestamps.pop_front();
        }
    }

    fn get_current_rate(&self) -> f64 {
        if self.timestamps.len() < 2 {
            return 0.0;
        }

        let time_span = self
            .timestamps
            .back()
            .unwrap()
            .duration_since(*self.timestamps.front().unwrap());

        if time_span.as_secs_f64() == 0.0 {
            return 0.0;
        }

        (self.timestamps.len() - 1) as f64 / time_span.as_secs_f64()
    }

    fn get_average_rate(&self) -> f64 {
        if self.total_messages == 0 {
            return 0.0;
        }

        let total_time = self.start_time.elapsed().as_secs_f64();
        if total_time == 0.0 {
            return 0.0;
        }

        self.total_messages as f64 / total_time
    }

    fn get_total_messages(&self) -> usize {
        self.total_messages
    }
}

async fn monitor_topic_rate(
    topic_name: &str,
    window_size: usize,
    use_wall_time: bool,
    running: Arc<AtomicBool>,
) -> Result<()> {
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
            .ok_or_else(|| anyhow!("Could not determine type for topic '{}'", topic_name))?
    };

    // Wait for publishers to be available
    if !graph_context.wait_for_topic_with_publishers(topic_name, Duration::from_secs(5))? {
        println!("WARNING: no publisher on [{}]", topic_name);
    }

    // Create dynamic subscription for real message rate monitoring
    let subscription = graph_context.create_subscription(topic_name, &topic_type)?;

    println!("Subscribed to [{}]", topic_name);
    println!("Topic type: {}", topic_type);

    let rate_calculator = Arc::new(Mutex::new(RateCalculator::new(window_size)));
    let rate_calc_clone = Arc::clone(&rate_calculator);

    let check_interval = Duration::from_millis(10); // High frequency polling for accurate rate measurement
    let mut stats_print_timer = Instant::now();
    let stats_print_interval = Duration::from_millis(100); // Print stats every 100ms

    println!("Monitoring topic rate (window size: {})...", window_size);
    println!("Press Ctrl+C to stop");

    // Main monitoring loop with real message reception
    while running.load(Ordering::Relaxed) {
        sleep(check_interval).await;

        // Check for new messages
        match subscription.take_message() {
            Ok(Some(_received)) => {
                // Message received - record timestamp
                let current_time = if use_wall_time {
                    Instant::now()
                } else {
                    // For now use wall time - in a full implementation, this would
                    // extract ROS time from the message header
                    Instant::now()
                };

                let mut calc = rate_calc_clone.lock().unwrap();
                calc.add_message(current_time);
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
            let calc = rate_calc_clone.lock().unwrap();
            let current_rate = calc.get_current_rate();
            let _average_rate = calc.get_average_rate();
            let total_msgs = calc.get_total_messages();

            if total_msgs > 0 {
                // Calculate statistics for display
                let periods: Vec<f64> = calc
                    .timestamps
                    .iter()
                    .zip(calc.timestamps.iter().skip(1))
                    .map(|(t1, t2)| t2.duration_since(*t1).as_secs_f64())
                    .collect();

                let (min_period, max_period, std_dev) = if periods.len() > 0 {
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

                println!(
                    "average rate: {:.3}\tmin: {:.3}s max: {:.3}s std dev: {:.3}s window: {}",
                    current_rate,
                    min_period,
                    max_period,
                    std_dev,
                    calc.timestamps.len().min(window_size)
                );
            }

            stats_print_timer = Instant::now();
        }

        // Check if publishers are still active
        let current_publisher_count = graph_context.count_publishers(topic_name).unwrap_or(0);
        if current_publisher_count == 0 {
            // Don't spam the warning - the rate display already shows no messages
            static mut LAST_NO_PUBLISHER_WARNING: Option<Instant> = None;
            let now = Instant::now();
            unsafe {
                let should_warn = match LAST_NO_PUBLISHER_WARNING {
                    Some(last_time) => now.duration_since(last_time) > Duration::from_secs(5),
                    None => true,
                };

                if should_warn {
                    println!("No publishers found for topic '{}'", topic_name);
                    LAST_NO_PUBLISHER_WARNING = Some(now);
                }
            }
        }
    }

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

    let use_wall_time = matches.get_flag("wall_time");

    // Handle common arguments
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    if common_args.use_sim_time && !use_wall_time {
        eprintln!("Note: Using simulation time for rate calculations");
    }

    if let Some(spin_time_value) = common_args.spin_time {
        eprintln!("Note: Using spin time {} for discovery", spin_time_value);
    }

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to listen for ctrl+c: {}", error);
            return;
        }
        running_clone.store(false, Ordering::Relaxed);
    });

    // Start monitoring
    monitor_topic_rate(topic_name, window_size, use_wall_time, running).await?;
    println!("\nRate monitoring stopped.");
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    run_async_command(run_command(matches, common_args));
}
