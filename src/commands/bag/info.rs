use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use std::path::PathBuf;

use crate::shared::rosbag2;

fn fmt_duration(ns: u64) -> String {
    let secs = ns as f64 / 1e9;
    format!("{:.3}s", secs)
}

fn run_command(matches: ArgMatches) -> Result<()> {
    let path = matches
        .get_one::<String>("PATH")
        .ok_or_else(|| anyhow!("PATH is required"))?;
    let bag_dir = PathBuf::from(path);
    if !rosbag2::is_rosbag2_directory(&bag_dir) {
        return Err(anyhow!(
            "Not a rosbag2 directory (missing metadata.yaml): {}",
            bag_dir.display()
        ));
    }

    let meta = rosbag2::load_metadata(&bag_dir)?;
    let info = meta.rosbag2_bagfile_information;

    println!("{}", "Rosbag Info".bright_yellow().bold());
    println!(
        "{} {}",
        "Path:".bright_yellow().bold(),
        bag_dir.display().to_string().bright_cyan()
    );
    println!(
        "{} {}",
        "Storage:".bright_yellow().bold(),
        info.storage_identifier.bright_white()
    );
    println!(
        "{} {}",
        "Version:".bright_yellow().bold(),
        info.version.to_string().bright_white()
    );
    println!(
        "{} {}",
        "Duration:".bright_yellow().bold(),
        fmt_duration(info.duration.nanoseconds).bright_white()
    );
    println!(
        "{} {}",
        "Messages:".bright_yellow().bold(),
        info.message_count.to_string().bright_white().bold()
    );

    if !info.compression_mode.is_empty() || !info.compression_format.is_empty() {
        println!(
            "{} {} {}",
            "Compression:".bright_yellow().bold(),
            info.compression_mode.bright_white(),
            info.compression_format.bright_white()
        );
    }

    println!();
    println!("{}", "Topics:".bright_yellow().bold());
    if info.topics_with_message_count.is_empty() {
        println!("  {}", "<none>".bright_black());
        return Ok(());
    }

    for t in info.topics_with_message_count {
        println!(
            "  {} {} {}",
            t.topic_metadata.name.bright_cyan(),
            format!("[{}]", t.topic_metadata.r#type).green(),
            format!("({})", t.message_count).bright_black()
        );
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
