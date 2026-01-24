use anyhow::Result;
use clap::ArgMatches;

use crate::graph::interface_operations;
use colored::*;

fn run_command(matches: ArgMatches) -> Result<()> {
    let only_msgs = matches.get_flag("messages");
    let only_srvs = matches.get_flag("services");
    let only_actions = matches.get_flag("actions");

    let items =
        interface_operations::list_packages_with_interfaces(only_msgs, only_srvs, only_actions)?;
    if items.is_empty() {
        eprintln!("{}", "No interface packages found.".yellow());
        return Ok(());
    }

    println!("{}", "Interface Packages:".bright_yellow().bold());
    for p in items.iter() {
        println!("  {}", p.bright_cyan());
    }
    println!();
    println!(
        "{} {} packages found",
        "Total:".bright_green(),
        items.len().to_string().bright_white().bold()
    );

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    if let Err(e) = run_command(matches) {
        // Allow piping to `head` etc.
        if let Some(ioe) = e.downcast_ref::<std::io::Error>() {
            if ioe.kind() == std::io::ErrorKind::BrokenPipe {
                return;
            }
        }
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
