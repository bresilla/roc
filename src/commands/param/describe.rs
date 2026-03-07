use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{parameter_type_to_string, ParamClientContext};

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;
    let param_name = matches
        .get_one::<String>("param_name")
        .ok_or_else(|| anyhow!("param_name is required"))?;

    if matches.get_flag("include_hidden_nodes") {
        eprintln!("Note: --include-hidden-nodes is not yet supported in native mode");
    }
    if common_args.use_sim_time {
        eprintln!("Note: --use-sim-time is not yet supported in native mode");
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

    let node_fqn = ParamClientContext::node_fqn(node_name);
    let mut ctx = ParamClientContext::new()?;

    let response = ctx.describe_parameters(&node_fqn, vec![param_name.to_string()])?;
    let descriptor = response
        .descriptors
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No descriptor returned for parameter '{}'", param_name))?;

    blocks::print_section("Parameter");
    blocks::print_field("Name", descriptor.name.bright_cyan());
    blocks::print_field(
        "Type",
        parameter_type_to_string(descriptor.type_).bright_black(),
    );
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

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
