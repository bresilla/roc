use anyhow::Result;
use clap::ArgMatches;

use crate::graph::interface_operations;
use crate::ui::{blocks, table};
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

    blocks::print_section("Interface Packages");
    let rows = items
        .iter()
        .map(|item| vec![item.bright_cyan().to_string()])
        .collect();
    table::print_table(&["Package"], rows);
    blocks::print_total(items.len(), "package", "packages");

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
