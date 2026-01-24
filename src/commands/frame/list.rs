use anyhow::Result;
use clap::ArgMatches;
use colored::*;
use std::time::{Duration, Instant};

use crate::shared::tf2_subscriber::{TfEdgeKind, TfFrameIndex};

fn run_command(matches: ArgMatches) -> Result<()> {
    let _show_all = matches.get_flag("all");
    let count_only = matches.get_flag("count_frames");

    let index = TfFrameIndex::new()?;

    // Wait briefly to collect messages.
    // /tf_static can be delivered once at startup; give DDS a moment.
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(1500) {
        std::thread::sleep(Duration::from_millis(50));
        if index.has_any_data() {
            break;
        }
    }

    let edges = index.edges();
    if count_only {
        println!(
            "{} {}",
            "Total:".bright_green(),
            edges.len().to_string().bright_white().bold()
        );
        return Ok(());
    }

    if edges.is_empty() {
        eprintln!("{}", "No frames found.".yellow());
        return Ok(());
    }

    println!("{}", "Available Transforms:".bright_yellow().bold());
    for ((parent, child), tf, kind) in &edges {
        let lhs = format!("{} -> {}", parent, child);
        let kind_str = match kind {
            TfEdgeKind::Static => "static",
            TfEdgeKind::Dynamic => "dynamic",
        };
        let rhs = format!(
            "t=[{:.3},{:.3},{:.3}] q=[{:.3},{:.3},{:.3},{:.3}] type=[{}]",
            tf.tx, tf.ty, tf.tz, tf.qx, tf.qy, tf.qz, tf.qw, kind_str
        );
        println!("  {} {}", lhs.bright_cyan(), rhs.bright_black());
    }
    println!();
    println!(
        "{} {} transforms found",
        "Total:".bright_green(),
        edges.len().to_string().bright_white().bold()
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
