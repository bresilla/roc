use crate::commands::cli::handle_anyhow_result;
use anyhow::{Result, anyhow};
use clap::ArgMatches;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{ParamClientContext, parameter_type_to_string};

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

    // Keep the output close to ros2 CLI but minimal.
    println!("Parameter name: {}", descriptor.name);
    println!("  Type: {}", parameter_type_to_string(descriptor.type_));
    if !descriptor.description.is_empty() {
        println!("  Description: {}", descriptor.description);
    }
    println!("  Read only: {}", descriptor.read_only);
    println!("  Dynamic typing: {}", descriptor.dynamic_typing);
    if !descriptor.additional_constraints.is_empty() {
        println!(
            "  Additional constraints: {}",
            descriptor.additional_constraints
        );
    }

    if !descriptor.integer_range.is_empty() {
        for r in descriptor.integer_range.iter() {
            println!(
                "  Integer range: from {} to {} (step {})",
                r.from_value, r.to_value, r.step
            );
        }
    }
    if !descriptor.floating_point_range.is_empty() {
        for r in descriptor.floating_point_range.iter() {
            println!(
                "  Floating point range: from {} to {} (step {})",
                r.from_value, r.to_value, r.step
            );
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
