use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use std::time::{Duration, Instant};

use crate::shared::tf2_subscriber::TfFrameIndex;
use crate::shared::tf_tree::TfGraph;

fn run_command(matches: ArgMatches) -> Result<()> {
    let frame_id = matches
        .get_one::<String>("FRAME_ID")
        .ok_or_else(|| anyhow!("FRAME_ID is required"))?;
    let child_frame_id = matches
        .get_one::<String>("CHILD_FRAME_ID")
        .ok_or_else(|| anyhow!("CHILD_FRAME_ID is required"))?;

    let rate_hz = matches
        .get_one::<String>("rate")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(10.0);
    let once = matches.get_flag("once");

    let period = if rate_hz <= 0.0 {
        Duration::from_millis(100)
    } else {
        Duration::from_secs_f64(1.0 / rate_hz)
    };

    let index = TfFrameIndex::new()?;
    let mut last_print = Instant::now() - period;

    loop {
        std::thread::sleep(Duration::from_millis(10));
        if last_print.elapsed() < period {
            continue;
        }
        last_print = Instant::now();

        let graph = TfGraph::from_edges(index.edges());
        let Some((tf, kind)) = graph.lookup(frame_id, child_frame_id) else {
            eprintln!(
                "{} {} {}",
                "No transform from".yellow(),
                frame_id.bright_cyan(),
                child_frame_id.bright_cyan()
            );
            if once {
                return Ok(());
            }
            continue;
        };

        let kind_str = match kind {
            crate::shared::tf2_subscriber::TfEdgeKind::Static => "static",
            crate::shared::tf2_subscriber::TfEdgeKind::Dynamic => "dynamic",
        };

        println!("{}", "Transform:".bright_yellow().bold());
        println!(
            "  {}",
            format!("{} -> {}", frame_id, child_frame_id).bright_cyan()
        );
        println!(
            "  {}",
            format!(
                "t=[{:.6},{:.6},{:.6}] q=[{:.6},{:.6},{:.6},{:.6}] type=[{}]",
                tf.tx, tf.ty, tf.tz, tf.qx, tf.qy, tf.qz, tf.qw, kind_str
            )
            .bright_white()
        );
        println!();

        if once {
            return Ok(());
        }
    }
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
