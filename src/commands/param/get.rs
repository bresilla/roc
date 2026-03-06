use crate::commands::cli::handle_anyhow_result;
use anyhow::{Result, anyhow};
use clap::ArgMatches;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{ParamClientContext, format_parameter_value_for_display};

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

    let hide_type = matches.get_flag("hide_type");
    let node_fqn = ParamClientContext::node_fqn(node_name);
    let mut ctx = ParamClientContext::new()?;

    let response = ctx.get_parameters(&node_fqn, vec![param_name.to_string()])?;
    let value = response
        .values
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No value returned for parameter '{}'", param_name))?;

    println!("{}", format_parameter_value_for_display(&value, hide_type));
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
