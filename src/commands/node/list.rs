use anyhow::{anyhow, Result};
use clap::ArgMatches;

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

    if matches.get_flag("count_nodes") {
        println!("{}", nodes.len());
        return Ok(());
    }

    // Print full node names, one per line.
    for (name, namespace) in nodes {
        if namespace == "/" {
            println!("/{}", name);
        } else if namespace.ends_with('/') {
            println!("{}{}", namespace, name);
        } else {
            println!("{}/{}", namespace, name);
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonNodeArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
