use anyhow::Result;
use clap::ArgMatches;
use colored::*;
use std::time::{Duration, Instant};

use crate::shared::tf2_subscriber::TfFrameIndex;

fn run_command(matches: ArgMatches) -> Result<()> {
    let _show_all = matches.get_flag("all");
    let count_only = matches.get_flag("count_frames");

    let index = TfFrameIndex::new()?;

    // Wait briefly to collect messages.
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(500) {
        std::thread::sleep(Duration::from_millis(50));
        if !index.frames().is_empty() {
            break;
        }
    }

    let frames = index.frames();
    if count_only {
        println!(
            "{} {}",
            "Total:".bright_green(),
            frames.len().to_string().bright_white().bold()
        );
        return Ok(());
    }

    if frames.is_empty() {
        eprintln!("{}", "No frames found.".yellow());
        return Ok(());
    }

    println!("{}", "Available Frames:".bright_yellow().bold());
    for f in &frames {
        println!("  {}", f.bright_cyan());
    }
    println!();
    println!(
        "{} {} frames found",
        "Total:".bright_green(),
        frames.len().to_string().bright_white().bold()
    );

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    if let Err(e) = run_command(matches) {
        if let Some(ioe) = e.downcast_ref::<std::io::Error>() {
            if ioe.kind() == std::io::ErrorKind::BrokenPipe {
                return;
            }
        }
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
