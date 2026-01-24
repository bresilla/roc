use anyhow::{anyhow, Result};
use clap::ArgMatches;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{parse_value_tokens_to_parameter_value, ParamClientContext};

use rclrs::vendor::rcl_interfaces::msg::Parameter;

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;
    let param_name = matches
        .get_one::<String>("param_name")
        .ok_or_else(|| anyhow!("param_name is required"))?;
    let value_tokens: Vec<String> = matches
        .get_many::<String>("value")
        .ok_or_else(|| anyhow!("value is required"))?
        .cloned()
        .collect();

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

    let value = parse_value_tokens_to_parameter_value(&value_tokens)?;
    let param = Parameter {
        name: param_name.to_string(),
        value,
    };

    let response = ctx.set_parameters(&node_fqn, vec![param])?;
    let first = response
        .results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No result returned by set_parameters"))?;

    if !first.successful {
        return Err(anyhow!(
            "Failed to set parameter '{}': {}",
            param_name,
            first.reason
        ));
    }

    println!("Set parameter {} successful", param_name);
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
