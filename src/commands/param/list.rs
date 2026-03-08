use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use serde_json::json;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{
    filter_parameter_names, parameter_type_to_string, ParamClientContext,
};

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;

    if matches.get_flag("include_hidden_nodes") {
        blocks::eprint_note("--include-hidden-nodes is not yet supported in native mode");
    }
    if common_args.use_sim_time {
        blocks::eprint_note("--use-sim-time is not yet supported in native mode");
    }
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    let node_fqn = ParamClientContext::node_fqn(node_name);
    let mut ctx = ParamClientContext::new_with_spin_time(common_args.spin_time.as_deref())?;

    let prefixes: Vec<String> = matches
        .get_many::<String>("param_prefixes")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    let response = ctx.list_parameters(&node_fqn, prefixes)?;
    let mut names: Vec<String> = response
        .result
        .names
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    names.sort();
    names = filter_parameter_names(
        names,
        matches.get_one::<String>("filter").map(|s| s.as_str()),
    )?;

    if matches.get_flag("param_type") {
        let types_response = ctx.get_parameter_types(&node_fqn, names.clone())?;
        if names.is_empty() {
            match output_mode {
                output::OutputMode::Json => {
                    output::print_json(&json!({ "node": node_fqn, "parameters": [], "count": 0 }))?;
                }
                _ => {
                    blocks::eprint_warning(&format!("No parameters found. [{}]", node_fqn));
                }
            }
            return Ok(());
        }
        let entries = names
            .iter()
            .cloned()
            .zip(types_response.types.into_iter())
            .map(|(name, ty)| (name, parameter_type_to_string(ty)))
            .collect::<Vec<_>>();
        match output_mode {
            output::OutputMode::Human => {
                blocks::print_section("Parameters");
                blocks::print_field("Node", format!("[{}]", node_fqn).bright_black());
                let rows = entries
                    .iter()
                    .map(|(name, ty)| {
                        vec![
                            name.bright_cyan().to_string(),
                            ty.bright_black().to_string(),
                        ]
                    })
                    .collect();
                table::print_table(&["Parameter", "Type"], rows);
                blocks::print_total(names.len(), "parameter", "parameters");
            }
            output::OutputMode::Plain => {
                output::print_plain_section("Parameters");
                output::print_plain_field("Node", &node_fqn);
                for (name, ty) in &entries {
                    println!("{name}\t{ty}");
                }
            }
            output::OutputMode::Json => {
                let parameters = entries
                    .iter()
                    .map(|(name, ty)| json!({ "name": name, "type": ty }))
                    .collect::<Vec<_>>();
                output::print_json(&json!({
                    "node": node_fqn,
                    "parameters": parameters,
                    "count": entries.len(),
                }))?;
            }
        }
        return Ok(());
    }

    if names.is_empty() {
        match output_mode {
            output::OutputMode::Json => {
                output::print_json(&json!({ "node": node_fqn, "parameters": [], "count": 0 }))?;
            }
            _ => {
                blocks::eprint_warning(&format!("No parameters found. [{}]", node_fqn));
            }
        }
        return Ok(());
    }
    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Parameters");
            blocks::print_field("Node", format!("[{}]", node_fqn).bright_black());
            let rows = names
                .iter()
                .map(|name| vec![name.bright_cyan().to_string()])
                .collect();
            table::print_table(&["Parameter"], rows);
            blocks::print_total(names.len(), "parameter", "parameters");
        }
        output::OutputMode::Plain => {
            output::print_plain_section("Parameters");
            output::print_plain_field("Node", &node_fqn);
            for name in &names {
                println!("{name}");
            }
        }
        output::OutputMode::Json => {
            let count = names.len();
            output::print_json(&json!({ "node": node_fqn, "parameters": names, "count": count }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
