use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output, table};
use anyhow::Result;
use clap::ArgMatches;
use serde_json::json;
use std::path::PathBuf;

use crate::shared::rosbag2;

fn run_command(matches: ArgMatches) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let root = matches
        .get_one::<String>("PATH")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("."));
    let recursive = matches.get_flag("recursive");

    let bags = rosbag2::find_rosbag2_directories(&root, recursive)?;
    if bags.is_empty() {
        match output_mode {
            output::OutputMode::Human => {
                blocks::eprint_warning("No rosbag2 recordings found.");
            }
            output::OutputMode::Json => {
                output::print_json(&json!({
                    "root": root.display().to_string(),
                    "recursive": recursive,
                    "bags": [],
                    "count": 0
                }))?;
            }
            output::OutputMode::Plain => {}
        }
        return Ok(());
    }

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Rosbag2 Recordings");
            let rows = bags
                .iter()
                .map(|bag| vec![bag.display().to_string()])
                .collect();
            table::print_table(&["Path"], rows);
            blocks::print_total(bags.len(), "recording", "recordings");
        }
        output::OutputMode::Plain => {
            for bag in &bags {
                println!("{}", bag.display());
            }
        }
        output::OutputMode::Json => {
            let paths = bags
                .iter()
                .map(|bag| bag.display().to_string())
                .collect::<Vec<_>>();
            output::print_json(&json!({
                "root": root.display().to_string(),
                "recursive": recursive,
                "bags": paths,
                "count": bags.len()
            }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
