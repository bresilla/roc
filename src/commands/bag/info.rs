use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::json;
use std::path::PathBuf;

use crate::shared::rosbag2;

fn fmt_duration(ns: u64) -> String {
    let secs = ns as f64 / 1e9;
    format!("{:.3}s", secs)
}

fn run_command(matches: ArgMatches) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
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

    let topic_rows = info
        .topics_with_message_count
        .iter()
        .map(|t| {
            vec![
                t.topic_metadata.name.clone(),
                t.topic_metadata.r#type.clone(),
                t.message_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Rosbag");
            blocks::print_field("Path", bag_dir.display());
            blocks::print_field("Storage", &info.storage_identifier);
            blocks::print_field("Version", info.version);
            blocks::print_field("Duration", fmt_duration(info.duration.nanoseconds));
            blocks::print_field("Messages", info.message_count);

            if !info.compression_mode.is_empty() || !info.compression_format.is_empty() {
                blocks::print_field(
                    "Compression",
                    format!("{} {}", info.compression_mode, info.compression_format).trim(),
                );
            }

            println!();
            blocks::print_section("Topics");
            if topic_rows.is_empty() {
                println!("<none>");
            } else {
                table::print_table(&["Topic", "Type", "Messages"], topic_rows);
            }
        }
        output::OutputMode::Plain => {
            output::print_plain_section("Rosbag");
            output::print_plain_field("Path", bag_dir.display());
            output::print_plain_field("Storage", &info.storage_identifier);
            output::print_plain_field("Version", info.version);
            output::print_plain_field("Duration", fmt_duration(info.duration.nanoseconds));
            output::print_plain_field("Messages", info.message_count);
            if !info.compression_mode.is_empty() || !info.compression_format.is_empty() {
                output::print_plain_field(
                    "Compression",
                    format!("{} {}", info.compression_mode, info.compression_format).trim(),
                );
            }
            println!();
            output::print_plain_section("Topics");
            for row in &topic_rows {
                println!("{}\t{}\t{}", row[0], row[1], row[2]);
            }
        }
        output::OutputMode::Json => {
            let topics = info
                .topics_with_message_count
                .iter()
                .map(|t| {
                    json!({
                        "name": t.topic_metadata.name,
                        "type": t.topic_metadata.r#type,
                        "message_count": t.message_count,
                    })
                })
                .collect::<Vec<_>>();
            output::print_json(&json!({
                "path": bag_dir.display().to_string(),
                "storage": info.storage_identifier,
                "version": info.version,
                "duration_ns": info.duration.nanoseconds,
                "message_count": info.message_count,
                "compression_mode": info.compression_mode,
                "compression_format": info.compression_format,
                "topics": topics,
            }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
