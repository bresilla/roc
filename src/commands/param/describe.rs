use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use serde_json::json;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{parameter_type_to_string, ParamClientContext};

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;
    let param_name = matches
        .get_one::<String>("param_name")
        .ok_or_else(|| anyhow!("param_name is required"))?;

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

    let response = ctx.describe_parameters(&node_fqn, vec![param_name.to_string()])?;
    let descriptor = response
        .descriptors
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No descriptor returned for parameter '{}'", param_name))?;

    let parameter_type = parameter_type_to_string(descriptor.type_);

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Parameter");
            blocks::print_field("Name", descriptor.name.bright_cyan());
            blocks::print_field("Type", parameter_type.bright_black());
            if !descriptor.description.is_empty() {
                blocks::print_field("Description", descriptor.description.bright_white());
            }
            blocks::print_field("Read only", descriptor.read_only);
            blocks::print_field("Dynamic typing", descriptor.dynamic_typing);
            if !descriptor.additional_constraints.is_empty() {
                blocks::print_field(
                    "Constraints",
                    descriptor.additional_constraints.bright_white(),
                );
            }

            if !descriptor.integer_range.is_empty() {
                println!();
                blocks::print_section("Integer Ranges");
                let rows = descriptor
                    .integer_range
                    .iter()
                    .map(|r| {
                        vec![
                            r.from_value.to_string(),
                            r.to_value.to_string(),
                            r.step.to_string(),
                        ]
                    })
                    .collect();
                table::print_table(&["From", "To", "Step"], rows);
            }
            if !descriptor.floating_point_range.is_empty() {
                println!();
                blocks::print_section("Floating Point Ranges");
                let rows = descriptor
                    .floating_point_range
                    .iter()
                    .map(|r| {
                        vec![
                            r.from_value.to_string(),
                            r.to_value.to_string(),
                            r.step.to_string(),
                        ]
                    })
                    .collect();
                table::print_table(&["From", "To", "Step"], rows);
            }
        }
        output::OutputMode::Plain => {
            output::print_plain_section("Parameter");
            output::print_plain_field("Name", &descriptor.name);
            output::print_plain_field("Type", &parameter_type);
            if !descriptor.description.is_empty() {
                output::print_plain_field("Description", &descriptor.description);
            }
            output::print_plain_field("Read only", descriptor.read_only);
            output::print_plain_field("Dynamic typing", descriptor.dynamic_typing);
            if !descriptor.additional_constraints.is_empty() {
                output::print_plain_field("Constraints", &descriptor.additional_constraints);
            }
            if !descriptor.integer_range.is_empty() {
                println!();
                output::print_plain_section("Integer Ranges");
                for range in descriptor.integer_range.iter() {
                    println!("{}\t{}\t{}", range.from_value, range.to_value, range.step);
                }
            }
            if !descriptor.floating_point_range.is_empty() {
                println!();
                output::print_plain_section("Floating Point Ranges");
                for range in descriptor.floating_point_range.iter() {
                    println!("{}\t{}\t{}", range.from_value, range.to_value, range.step);
                }
            }
        }
        output::OutputMode::Json => {
            output::print_json(&json!({
                "node": node_fqn,
                "parameter": {
                    "name": descriptor.name,
                    "type": parameter_type,
                    "description": descriptor.description,
                    "read_only": descriptor.read_only,
                    "dynamic_typing": descriptor.dynamic_typing,
                    "constraints": descriptor.additional_constraints,
                    "integer_ranges": descriptor.integer_range.iter().map(|r| json!({
                        "from": r.from_value,
                        "to": r.to_value,
                        "step": r.step,
                    })).collect::<Vec<_>>(),
                    "floating_point_ranges": descriptor.floating_point_range.iter().map(|r| json!({
                        "from": r.from_value,
                        "to": r.to_value,
                        "step": r.step,
                    })).collect::<Vec<_>>(),
                }
            }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
