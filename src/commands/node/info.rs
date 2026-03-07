use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;

use crate::arguments::node::CommonNodeArgs;
use crate::graph::RclGraphContext;

fn print_names_and_types_section(title: &str, map: rclrs::TopicNamesAndTypes) {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for (name, types) in map {
        for ty in types {
            pairs.push((name.clone(), ty));
        }
    }
    pairs.sort_by(|(a, at), (b, bt)| a.cmp(b).then(at.cmp(bt)));

    println!();
    blocks::print_section(title);
    if pairs.is_empty() {
        println!("(none)");
        return;
    }
    let rows = pairs
        .into_iter()
        .map(|(name, ty)| vec![name, ty])
        .collect();
    table::print_table(&["Name", "Type"], rows);
}

fn run_command(matches: ArgMatches, common_args: CommonNodeArgs) -> Result<()> {
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;

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

    // Normalize possible inputs:
    // - "/talker"
    // - "talker"
    let requested = node_name.trim();
    let requested = requested.strip_prefix('/').unwrap_or(requested);

    let mut matches_nodes = Vec::new();
    for (name, namespace) in nodes {
        if name == requested {
            matches_nodes.push((name, namespace));
        }
    }

    if matches_nodes.is_empty() {
        return Err(anyhow!("Node '{}' not found", node_name));
    }

    for (name, namespace) in matches_nodes {
        let fqn = if namespace == "/" {
            format!("/{name}")
        } else if namespace.ends_with('/') {
            format!("{namespace}{name}")
        } else {
            format!("{namespace}/{name}")
        };

        blocks::print_section("Node");
        blocks::print_field("Name", fqn.as_str());
        blocks::print_field("Namespace", namespace.as_str());

        let publishers = context
            .node()
            .get_publisher_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query publishers for {}: {}", fqn, e))?;
        let subscriptions = context
            .node()
            .get_subscription_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query subscriptions for {}: {}", fqn, e))?;
        let services = context
            .node()
            .get_service_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query services for {}: {}", fqn, e))?;
        let clients = context
            .node()
            .get_client_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query clients for {}: {}", fqn, e))?;

        print_names_and_types_section("Subscribers", subscriptions);
        print_names_and_types_section("Publishers", publishers);
        print_names_and_types_section("Services", services);
        print_names_and_types_section("Clients", clients);
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonNodeArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
