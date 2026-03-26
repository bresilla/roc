use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::ParamClientContext;

use rclrs::vendor::rcl_interfaces::msg::{Parameter, ParameterType, ParameterValue};

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

    // Parameters are removed by setting them to NOT_SET (same behavior as
    // `ros2 param delete`).
    let mut v = ParameterValue::default();
    v.type_ = ParameterType::PARAMETER_NOT_SET;
    let p = Parameter {
        name: param_name.to_string(),
        value: v,
    };

    let response = ctx.set_parameters(&node_fqn, vec![p])?;
    let first = response
        .results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No result returned by set_parameters"))?;

    if !first.successful {
        return Err(anyhow!(
            "Failed to delete parameter '{}': {}",
            param_name,
            first.reason
        ));
    }

    println!("Deleted parameter {}", param_name);
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
