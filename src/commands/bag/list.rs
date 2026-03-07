use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, table};
use anyhow::Result;
use clap::ArgMatches;
use colored::*;
use std::path::PathBuf;

use crate::shared::rosbag2;

fn run_command(matches: ArgMatches) -> Result<()> {
    let root = matches
        .get_one::<String>("PATH")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("."));
    let recursive = matches.get_flag("recursive");

    let bags = rosbag2::find_rosbag2_directories(&root, recursive)?;
    if bags.is_empty() {
        eprintln!("{}", "No rosbag2 recordings found.".yellow());
        return Ok(());
    }

    blocks::print_section("Rosbag2 Recordings");
    let rows = bags
        .iter()
        .map(|bag| vec![bag.display().to_string().bright_cyan().to_string()])
        .collect();
    table::print_table(&["Path"], rows);
    blocks::print_total(bags.len(), "recording", "recordings");

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
