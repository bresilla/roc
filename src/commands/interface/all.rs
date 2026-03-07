use anyhow::Result;
use clap::ArgMatches;
use serde_json::json;

use crate::shared::interface_operations;
use crate::ui::{blocks, output, table};

fn run_command(matches: ArgMatches) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let only_msgs = matches.get_flag("messages");
    let only_srvs = matches.get_flag("services");
    let only_actions = matches.get_flag("actions");

    let items =
        interface_operations::list_packages_with_interfaces(only_msgs, only_srvs, only_actions)?;
    if items.is_empty() {
        match output_mode {
            output::OutputMode::Human => blocks::eprint_warning("No interface packages found."),
            output::OutputMode::Json => {
                output::print_json(&json!({ "packages": [], "count": 0 }))?;
            }
            output::OutputMode::Plain => {}
        }
        return Ok(());
    }

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Interface Packages");
            let rows = items.iter().map(|item| vec![item.clone()]).collect();
            table::print_table(&["Package"], rows);
            blocks::print_total(items.len(), "package", "packages");
        }
        output::OutputMode::Plain => {
            for item in &items {
                println!("{item}");
            }
        }
        output::OutputMode::Json => {
            let count = items.len();
            output::print_json(&json!({ "packages": items, "count": count }))?;
        }
    }

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
