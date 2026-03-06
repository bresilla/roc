use crate::commands::cli::handle_anyhow_result;
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

    println!("{}", "Rosbag2 Recordings:".bright_yellow().bold());
    for bag in &bags {
        println!("  {}", bag.display().to_string().bright_cyan());
    }
    println!();
    println!(
        "{} {} recordings found",
        "Total:".bright_green(),
        bags.len().to_string().bright_white().bold()
    );

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
