use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, table};
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

    blocks::print_section("Rosbag");
    blocks::print_field("Path", bag_dir.display().to_string().bright_cyan());
    blocks::print_field("Storage", info.storage_identifier.bright_white());
    blocks::print_field("Version", info.version.to_string().bright_white());
    blocks::print_field(
        "Duration",
        fmt_duration(info.duration.nanoseconds).bright_white(),
    );
    blocks::print_field("Messages", info.message_count.to_string().bright_white().bold());

    if !info.compression_mode.is_empty() || !info.compression_format.is_empty() {
        blocks::print_field(
            "Compression",
            format!(
                "{} {}",
                info.compression_mode.bright_white(),
                info.compression_format.bright_white()
            ),
        );
    }

    println!();
    blocks::print_section("Topics");
    if info.topics_with_message_count.is_empty() {
        println!("{}", "<none>".bright_black());
        return Ok(());
    }

    let rows = info
        .topics_with_message_count
        .into_iter()
        .map(|t| {
            vec![
                t.topic_metadata.name.bright_cyan().to_string(),
                t.topic_metadata.r#type.green().to_string(),
                t.message_count.to_string().bright_black().to_string(),
            ]
        })
        .collect();
    table::print_table(&["Topic", "Type", "Messages"], rows);

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
