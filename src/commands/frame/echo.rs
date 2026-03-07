use crate::commands::cli::{handle_anyhow_result, install_ctrlc_flag};
use crate::ui::blocks;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    let mut last_missing_warning: Option<Instant> = None;
    let running = Arc::new(AtomicBool::new(true));
    install_ctrlc_flag(Arc::clone(&running))?;

    blocks::print_section("Frame Echo");
    blocks::print_field("Source", frame_id);
    blocks::print_field("Target", child_frame_id);
    blocks::print_field("Rate", format!("{rate_hz:.2} Hz"));
    println!();
    if !once {
        blocks::print_note("Press Ctrl+C to stop");
    }

    while running.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(10));
        if last_print.elapsed() < period {
            continue;
        }
        last_print = Instant::now();

        let graph = TfGraph::from_edges(index.edges());
        let Some((tf, kind)) = graph.lookup(frame_id, child_frame_id) else {
            let now = Instant::now();
            let should_warn = match last_missing_warning {
                Some(last_warning) => now.duration_since(last_warning) >= Duration::from_secs(5),
                None => true,
            };
            if should_warn {
                blocks::eprint_warning(&format!(
                    "No transform from {frame_id} to {child_frame_id}"
                ));
                last_missing_warning = Some(now);
            }
            if once {
                return Ok(());
            }
            continue;
        };

        let kind_str = match kind {
            crate::shared::tf2_subscriber::TfEdgeKind::Static => "static",
            crate::shared::tf2_subscriber::TfEdgeKind::Dynamic => "dynamic",
        };

        blocks::print_status(
            "Transform",
            &[
                ("path", format!("{frame_id} -> {child_frame_id}")),
                ("type", kind_str.to_string()),
                (
                    "translation",
                    format!("[{:.6}, {:.6}, {:.6}]", tf.tx, tf.ty, tf.tz),
                ),
                (
                    "rotation",
                    format!("[{:.6}, {:.6}, {:.6}, {:.6}]", tf.qx, tf.qy, tf.qz, tf.qw),
                ),
            ],
        );

        if once {
            return Ok(());
        }
    }

    println!();
    blocks::print_success("Frame echo stopped");
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
