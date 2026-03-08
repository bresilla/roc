use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use serde_json::json;

use crate::arguments::node::CommonNodeArgs;
use crate::graph::RclGraphContext;
use crate::shared::ros_names::is_hidden_node_name;

fn run_command(matches: ArgMatches, common_args: CommonNodeArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);

    if common_args.use_sim_time {
        blocks::eprint_note("--use-sim-time is not applicable to graph queries");
    }
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    let context = RclGraphContext::new_with_spin_time(common_args.spin_time.as_deref())
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;
    let nodes = context
        .get_node_names_with_namespaces()
        .map_err(|e| anyhow!("Failed to query nodes: {}", e))?;

    let mut full_names: Vec<String> = Vec::new();
    for (name, namespace) in nodes {
        if namespace == "/" {
            full_names.push(format!("/{}", name));
        } else if namespace.ends_with('/') {
            full_names.push(format!("{}{}", namespace, name));
        } else {
            full_names.push(format!("{}/{}", namespace, name));
        }
    }
    if !matches.get_flag("include_hidden_nodes") {
        full_names.retain(|name| !is_hidden_node_name(name));
    }
    full_names.sort();

    if matches.get_flag("count_nodes") {
        match output_mode {
            output::OutputMode::Human => {
                println!(
                    "{} {}",
                    "Total:".bright_green(),
                    full_names.len().to_string().bright_white().bold()
                );
            }
            output::OutputMode::Plain => println!("{}", full_names.len()),
            output::OutputMode::Json => output::print_json(&json!({ "count": full_names.len() }))?,
        }
        return Ok(());
    }

    if full_names.is_empty() {
        match output_mode {
            output::OutputMode::Json => output::print_json(&json!({ "nodes": [], "count": 0 }))?,
            _ => {
                blocks::eprint_warning(&format!(
                    "No nodes found. [{}]",
                    RclGraphContext::get_daemon_status()
                ));
            }
        }
        return Ok(());
    }

    let total = full_names.len();

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Nodes");
            let rows = full_names
                .iter()
                .map(|name| vec![name.bright_cyan().to_string()])
                .collect();
            table::print_table(&["Node"], rows);
            blocks::print_total(total, "node", "nodes");
        }
        output::OutputMode::Plain => {
            for name in &full_names {
                println!("{name}");
            }
        }
        output::OutputMode::Json => {
            output::print_json(&json!({ "nodes": full_names, "count": total }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonNodeArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
