use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::json;

use crate::arguments::node::CommonNodeArgs;
use crate::graph::RclGraphContext;

fn flatten_names_and_types(map: rclrs::TopicNamesAndTypes) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for (name, types) in map {
        for ty in types {
            pairs.push((name.clone(), ty));
        }
    }
    pairs.sort_by(|(a, at), (b, bt)| a.cmp(b).then(at.cmp(bt)));
    pairs
}

fn print_names_and_types_section(title: &str, pairs: &[(String, String)]) {
    println!();
    blocks::print_section(title);
    if pairs.is_empty() {
        println!("(none)");
        return;
    }
    let rows = pairs
        .iter()
        .cloned()
        .map(|(name, ty)| vec![name, ty])
        .collect();
    table::print_table(&["Name", "Type"], rows);
}

fn run_command(matches: ArgMatches, common_args: CommonNodeArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;

    if matches.get_flag("include_hidden_nodes") {
        blocks::eprint_note("--include-hidden-nodes is not yet supported in native mode");
    }

    if common_args.use_sim_time {
        blocks::eprint_note("--use-sim-time is not applicable to graph queries");
    }
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    if let Some(spin_time_value) = common_args.spin_time {
        blocks::eprint_note(&format!(
            "--spin-time {} is not yet supported in native mode",
            spin_time_value
        ));
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

    let mut json_matches = Vec::new();

    for (name, namespace) in matches_nodes {
        let fqn = if namespace == "/" {
            format!("/{name}")
        } else if namespace.ends_with('/') {
            format!("{namespace}{name}")
        } else {
            format!("{namespace}/{name}")
        };

        let publishers = context
            .node()
            .get_publisher_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query publishers for {}: {}", fqn, e))?;
        let publishers = flatten_names_and_types(publishers);
        let subscriptions = context
            .node()
            .get_subscription_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query subscriptions for {}: {}", fqn, e))?;
        let subscriptions = flatten_names_and_types(subscriptions);
        let services = context
            .node()
            .get_service_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query services for {}: {}", fqn, e))?;
        let services = flatten_names_and_types(services);
        let clients = context
            .node()
            .get_client_names_and_types_by_node(&name, &namespace)
            .map_err(|e| anyhow!("Failed to query clients for {}: {}", fqn, e))?;
        let clients = flatten_names_and_types(clients);

        match output_mode {
            output::OutputMode::Human => {
                blocks::print_section("Node");
                blocks::print_field("Name", fqn.as_str());
                blocks::print_field("Namespace", namespace.as_str());
                print_names_and_types_section("Subscribers", &subscriptions);
                print_names_and_types_section("Publishers", &publishers);
                print_names_and_types_section("Services", &services);
                print_names_and_types_section("Clients", &clients);
            }
            output::OutputMode::Plain => {
                output::print_plain_section("Node");
                output::print_plain_field("Name", &fqn);
                output::print_plain_field("Namespace", &namespace);
                println!();
                output::print_plain_section("Subscribers");
                for (entry_name, ty) in &subscriptions {
                    println!("{entry_name}\t{ty}");
                }
                println!();
                output::print_plain_section("Publishers");
                for (entry_name, ty) in &publishers {
                    println!("{entry_name}\t{ty}");
                }
                println!();
                output::print_plain_section("Services");
                for (entry_name, ty) in &services {
                    println!("{entry_name}\t{ty}");
                }
                println!();
                output::print_plain_section("Clients");
                for (entry_name, ty) in &clients {
                    println!("{entry_name}\t{ty}");
                }
            }
            output::OutputMode::Json => {
                json_matches.push(json!({
                    "name": fqn,
                    "namespace": namespace,
                    "subscribers": subscriptions.iter().map(|(entry_name, ty)| json!({ "name": entry_name, "type": ty })).collect::<Vec<_>>(),
                    "publishers": publishers.iter().map(|(entry_name, ty)| json!({ "name": entry_name, "type": ty })).collect::<Vec<_>>(),
                    "services": services.iter().map(|(entry_name, ty)| json!({ "name": entry_name, "type": ty })).collect::<Vec<_>>(),
                    "clients": clients.iter().map(|(entry_name, ty)| json!({ "name": entry_name, "type": ty })).collect::<Vec<_>>(),
                }));
            }
        }
    }

    if output_mode == output::OutputMode::Json {
        output::print_json(&json!({ "matches": json_matches }))?;
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonNodeArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
