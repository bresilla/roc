use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;

use crate::arguments::node::CommonNodeArgs;
use crate::graph::RclGraphContext;

fn run_command(matches: ArgMatches, common_args: CommonNodeArgs) -> Result<()> {
    // NOTE: rclrs does not currently provide the same filtering as `ros2 node list`
    // for hidden nodes, so for now we always return what the graph exposes.
    if matches.get_flag("include_hidden_nodes") {
        eprintln!("Note: --include-hidden-nodes is not yet supported in native mode");
    }

    if common_args.use_sim_time {
        eprintln!("Note: --use-sim-time is not applicable to graph queries");
    }
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    if let Some(spin_time_value) = common_args.spin_time {
        eprintln!(
            "Note: --spin-time {} is not yet supported in native mode",
            spin_time_value
        );
    }

    let context = RclGraphContext::new()
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
    full_names.sort();

    if matches.get_flag("count_nodes") {
        println!(
            "{} {}",
            "Total:".bright_green(),
            full_names.len().to_string().bright_white().bold()
        );
        return Ok(());
    }

    if full_names.is_empty() {
        eprintln!(
            "{} {}",
            "No nodes found.".yellow(),
            format!("[{}]", RclGraphContext::get_daemon_status()).bright_black()
        );
        return Ok(());
    }

    let total = full_names.len();

    println!("{}", "Available Nodes:".bright_yellow().bold());
    for n in &full_names {
        println!("  {}", n.bright_cyan());
    }
    println!();
    println!(
        "{} {} nodes found",
        "Total:".bright_green(),
        total.to_string().bright_white().bold()
    );

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonNodeArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
