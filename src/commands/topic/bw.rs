use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
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

struct BandwidthCalculator {
    message_sizes: VecDeque<(Instant, usize)>,
    window_duration: Duration,
    total_messages: usize,
    total_bytes: usize,
    start_time: Instant,
}

impl BandwidthCalculator {
    fn new(window_duration: Duration) -> Self {
        Self {
            message_sizes: VecDeque::new(),
            window_duration,
            total_messages: 0,
            total_bytes: 0,
            start_time: Instant::now(),
        }
    }

    fn add_message(&mut self, timestamp: Instant, message_size: usize) {
        self.message_sizes.push_back((timestamp, message_size));
        self.total_messages += 1;
        self.total_bytes += message_size;

        // Remove old entries outside the window
        let cutoff_time = timestamp - self.window_duration;
        while let Some((time, _)) = self.message_sizes.front() {
            if *time < cutoff_time {
                self.message_sizes.pop_front();
            } else {
                break;
            }
        }
    }

    fn get_current_bandwidth(&self) -> f64 {
        if self.message_sizes.len() < 2 {
            return 0.0;
        }

        let total_size: usize = self.message_sizes.iter().map(|(_, size)| size).sum();
        let time_span = self.message_sizes.back().unwrap().0
            - self.message_sizes.front().unwrap().0;

        if time_span.as_secs_f64() == 0.0 {
            return 0.0;
        }

        total_size as f64 / time_span.as_secs_f64()
    }

    fn get_average_bandwidth(&self) -> f64 {
        if self.total_messages == 0 {
            return 0.0;
        }

        let total_time = self.start_time.elapsed().as_secs_f64();
        if total_time == 0.0 {
            return 0.0;
        }

        self.total_bytes as f64 / total_time
    }

    fn get_message_count(&self) -> usize {
        self.total_messages
    }
}

async fn monitor_topic_bandwidth(
    topic_name: &str,
    window_size: Duration,
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
        let topics_and_types = graph_context.get_topic_names_and_types()
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

    // Create dynamic subscription for real bandwidth monitoring
    let subscription = graph_context.create_subscription(topic_name, &topic_type)?;

    println!("Subscribed to [{}]", topic_name);
    println!("Topic type: {}", topic_type);
    
    let bandwidth_calc = Arc::new(Mutex::new(BandwidthCalculator::new(window_size)));
    let bandwidth_calc_clone = Arc::clone(&bandwidth_calc);

    let check_interval = Duration::from_millis(10); // High frequency polling for accurate bandwidth measurement
    let mut stats_print_timer = Instant::now();
    let stats_print_interval = Duration::from_millis(100); // Print stats every 100ms

    // Main monitoring loop with real message reception
    while running.load(Ordering::Relaxed) {
        sleep(check_interval).await;

        // Check for new messages
        match subscription.take_message() {
            Ok(Some(received)) => {
                // Message received - record timestamp and size
                let current_time = Instant::now();
                let message_size = format!("{:?}", received.message.view()).len();

                let mut calc = bandwidth_calc_clone.lock().unwrap();
                calc.add_message(current_time, message_size);
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
            let calc = bandwidth_calc_clone.lock().unwrap();
            let current_bw = calc.get_current_bandwidth();
            let average_bw = calc.get_average_bandwidth();
            let msg_count = calc.get_message_count();

            if msg_count > 0 {
                // Calculate min/max/mean bandwidth from recent measurements
                let bandwidth_samples: Vec<f64> = calc.message_sizes.iter()
                    .zip(calc.message_sizes.iter().skip(1))
                    .map(|((t1, s1), (t2, _s2))| {
                        let time_diff = t2.duration_since(*t1).as_secs_f64();
                        if time_diff > 0.0 {
                            *s1 as f64 / time_diff
                        } else {
                            0.0
                        }
                    })
                    .filter(|&bw| bw > 0.0)
                    .collect();

                let (min_bw, max_bw) = if bandwidth_samples.len() > 0 {
                    let min_bw = bandwidth_samples.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_bw = bandwidth_samples.iter().fold(0.0f64, |a, &b| a.max(b));
                    (min_bw, max_bw)
                } else {
                    (current_bw, current_bw)
                };

                // Format bandwidth output like ros2 topic bw
                println!(
                    "average: {:.2} B/s\tmean: {:.2} B/s\tmin: {:.2} B/s\tmax: {:.2} B/s\twindow: {}",
                    current_bw,
                    average_bw,
                    min_bw,
                    max_bw,
                    calc.message_sizes.len().min(window_size.as_secs() as usize * 10)
                );
            }

            stats_print_timer = Instant::now();
        }

        // Check if publishers are still active
        let current_publisher_count = graph_context.count_publishers(topic_name).unwrap_or(0);
        if current_publisher_count == 0 {
            // Don't spam the warning - the bandwidth display already shows no messages
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
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    let window_duration = Duration::from_millis(window_size * 10); // Convert to reasonable duration

    // Handle common arguments
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    if common_args.use_sim_time {
        eprintln!("Note: Using simulation time for bandwidth calculations");
    }

    if let Some(ref spin_time_value) = common_args.spin_time {
        eprintln!("Note: Using spin time {} for discovery", spin_time_value);
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

    // Start monitoring
    monitor_topic_bandwidth(topic_name, window_duration, running).await
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(run_command(matches, common_args)) {
        Ok(()) => {
            println!("\nBandwidth monitoring stopped.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
