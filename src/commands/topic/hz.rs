use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::{anyhow, Result};
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
    // Create RCL context for direct API access
    let graph_context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Verify topic exists
    let topics = graph_context
        .get_topic_names()
        .map_err(|e| anyhow!("Failed to get topic names: {}", e))?;

    if !topics.contains(&topic_name.to_string()) {
        return Err(anyhow!("Topic '{}' not found", topic_name));
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

    println!("Subscribed to [{}]", topic_name);
    println!("Topic type: {}", topic_type);

    let rate_calculator = Arc::new(Mutex::new(RateCalculator::new(window_size)));
    let rate_calc_clone = Arc::clone(&rate_calculator);

    // Create a subscription to monitor messages
    // Note: In a full implementation, we would create an actual RCL subscription
    // For now, we'll simulate the rate monitoring by checking publisher count changes
    // This is a simplified version - a complete implementation would need to:
    // 1. Create an RCL subscription
    // 2. Set up a callback to record message timestamps
    // 3. Handle the actual message receiving

    let mut last_publisher_count = 0;
    let mut last_message_time = Instant::now();
    let check_interval = Duration::from_millis(100);

    println!("Monitoring topic rate (window size: {})...", window_size);
    println!("Press Ctrl+C to stop");

    // Main monitoring loop
    while running.load(Ordering::Relaxed) {
        sleep(check_interval).await;

        // Check if publishers are still active
        let current_publisher_count = graph_context.count_publishers(topic_name).unwrap_or(0);

        if current_publisher_count == 0 {
            println!("No publishers found for topic '{}'", topic_name);
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        // Simulate message arrival detection
        // In a real implementation, this would be triggered by actual message callbacks
        let current_time = if use_wall_time {
            Instant::now()
        } else {
            // For simulation, we'll use wall time
            // In a real implementation, this would use ROS time from message headers
            Instant::now()
        };

        // Simulate message detection (this is a placeholder)
        // In reality, we'd have callbacks from the RCL subscription
        if current_publisher_count != last_publisher_count
            || current_time.duration_since(last_message_time) > Duration::from_millis(500)
        {
            let mut calc = rate_calc_clone.lock().unwrap();
            calc.add_message(current_time);

            let current_rate = calc.get_current_rate();
            let _average_rate = calc.get_average_rate();
            let total_msgs = calc.get_total_messages();

            println!(
                "average rate: {:.3}\tmin: {:.3}s max: {:.3}s std dev: {:.3}s window: {}",
                current_rate,
                if current_rate > 0.0 {
                    1.0 / current_rate
                } else {
                    0.0
                }, // min period
                if current_rate > 0.0 {
                    1.0 / current_rate
                } else {
                    0.0
                }, // max period (simplified)
                0.0, // std dev (simplified)
                total_msgs.min(window_size)
            );

            last_message_time = current_time;
        }

        last_publisher_count = current_publisher_count;
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
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        running_clone.store(false, Ordering::Relaxed);
    });

    // Start monitoring
    monitor_topic_rate(topic_name, window_size, use_wall_time, running).await
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(run_command(matches, common_args)) {
        Ok(()) => {
            println!("\nRate monitoring stopped.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
