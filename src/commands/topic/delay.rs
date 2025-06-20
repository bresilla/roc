use crate::arguments::topic::CommonTopicArgs;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct DelayOptions {
    topic_name: String,
    delay_duration: Duration,
    verbose: bool,
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

        Ok(DelayOptions {
            topic_name,
            delay_duration,
            verbose,
        })
    }
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

async fn delay_topic(options: DelayOptions, running: Arc<AtomicBool>) -> Result<()> {
    println!(
        "Delaying topic '{}' for {:?}...", 
        options.topic_name, 
        options.delay_duration
    );

    if options.verbose {
        println!("This will pause any processing or forwarding of this topic");
        println!("Press Ctrl+C to cancel the delay early");
    }

    // Create a countdown if the delay is longer than 5 seconds
    if options.delay_duration.as_secs() > 5 {
        let total_seconds = options.delay_duration.as_secs();
        
        for remaining in (1..=total_seconds).rev() {
            if !running.load(Ordering::Relaxed) {
                println!("\nDelay cancelled!");
                return Ok(());
            }
            
            if options.verbose || remaining % 10 == 0 || remaining <= 10 {
                println!("Delaying '{}' - {} seconds remaining...", options.topic_name, remaining);
            }
            
            sleep(Duration::from_secs(1)).await;
        }
    } else {
        // For short delays, just sleep the full duration
        let mut elapsed = Duration::ZERO;
        let check_interval = Duration::from_millis(100);
        
        while elapsed < options.delay_duration {
            if !running.load(Ordering::Relaxed) {
                println!("\nDelay cancelled!");
                return Ok(());
            }
            
            sleep(check_interval).await;
            elapsed += check_interval;
        }
    }

    if running.load(Ordering::Relaxed) {
        println!("✓ Delay completed for topic '{}'", options.topic_name);
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    // Setup signal handling for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler");

    // Parse options
    let options = match DelayOptions::from_matches(&matches, &common_args) {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            return;
        }
    };

    // Create async runtime and run the delay
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    if let Err(e) = rt.block_on(delay_topic(options, running)) {
        eprintln!("Error during topic delay: {}", e);
    }
}