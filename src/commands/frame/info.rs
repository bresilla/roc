use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::shared::tf2_subscriber::TfFrameIndex;
use crate::shared::tf_dump;

fn run_command(matches: ArgMatches) -> Result<()> {
    let frame_name = matches
        .get_one::<String>("FRAME_NAME")
        .ok_or_else(|| anyhow!("FRAME_NAME is required"))?;

    let export_dot = matches.get_one::<String>("export_dot").map(PathBuf::from);
    let export_json = matches.get_one::<String>("export_json").map(PathBuf::from);
    let export_yaml = matches.get_one::<String>("export_yaml").map(PathBuf::from);
    let export_image = matches.get_one::<String>("export_image").map(PathBuf::from);

    if export_image.is_some() {
        // We don't generate images yet; dot export is native and can be rendered by graphviz.
        return Err(anyhow!(
            "--export-image is not supported natively yet (use --export-dot)"
        ));
    }

    let index = TfFrameIndex::new()?;
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(1500) {
        std::thread::sleep(Duration::from_millis(50));
        if index.has_any_data() {
            break;
        }
    }

    let graph = tf_dump::export_graph(index.edges());

    if let Some(path) = export_dot {
        fs::write(&path, tf_dump::export_dot(&graph))
            .map_err(|e| anyhow!("Failed to write {}: {}", path.display(), e))?;
    }
    if let Some(path) = export_json {
        let s = serde_json::to_string_pretty(&graph)?;
        fs::write(&path, s).map_err(|e| anyhow!("Failed to write {}: {}", path.display(), e))?;
    }
    if let Some(path) = export_yaml {
        let s = serde_yaml::to_string(&graph)?;
        fs::write(&path, s).map_err(|e| anyhow!("Failed to write {}: {}", path.display(), e))?;
    }

    let (parents, children) = tf_dump::build_parent_children_map(&graph.edges);
    let incoming = parents.get(frame_name).cloned().unwrap_or_default();
    let outgoing = children.get(frame_name).cloned().unwrap_or_default();

    if incoming.is_empty() && outgoing.is_empty() {
        return Err(anyhow!("Frame '{}' not found in TF graph", frame_name));
    }

    println!(
        "{} {}",
        "Frame:".bright_yellow().bold(),
        frame_name.bright_cyan()
    );
    println!();

    println!("{}", "Parents:".bright_yellow().bold());
    if incoming.is_empty() {
        println!("  {}", "<none>".bright_black());
    } else {
        for e in &incoming {
            println!(
                "  {} {}",
                format!("{} -> {}", e.parent, e.child).bright_cyan(),
                format!("type=[{}]", e.kind).bright_black()
            );
        }
    }

    println!();
    println!("{}", "Children:".bright_yellow().bold());
    if outgoing.is_empty() {
        println!("  {}", "<none>".bright_black());
    } else {
        for e in &outgoing {
            println!(
                "  {} {}",
                format!("{} -> {}", e.parent, e.child).bright_cyan(),
                format!("type=[{}]", e.kind).bright_black()
            );
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
