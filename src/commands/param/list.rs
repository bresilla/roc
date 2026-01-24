use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{
    filter_parameter_names, parameter_type_to_string, ParamClientContext,
};

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;

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
            eprintln!(
                "{} {}",
                "No parameters found.".yellow(),
                format!("[{}]", node_fqn).bright_black()
            );
            return Ok(());
        }
        println!(
            "{} {}",
            "Parameters:".bright_yellow().bold(),
            format!("[{}]", node_fqn).bright_black()
        );
        for (name, ty) in names.iter().cloned().zip(types_response.types.into_iter()) {
            println!(
                "  {} {}",
                name.bright_cyan(),
                format!("({})", parameter_type_to_string(ty)).bright_black()
            );
        }
        println!();
        println!(
            "{} {} params found",
            "Total:".bright_green(),
            names.len().to_string().bright_white().bold()
        );
        return Ok(());
    }

    if names.is_empty() {
        eprintln!(
            "{} {}",
            "No parameters found.".yellow(),
            format!("[{}]", node_fqn).bright_black()
        );
        return Ok(());
    }
    println!(
        "{} {}",
        "Parameters:".bright_yellow().bold(),
        format!("[{}]", node_fqn).bright_black()
    );
    for name in names.iter() {
        println!("  {}", name.bright_cyan());
    }
    println!();
    println!(
        "{} {} params found",
        "Total:".bright_green(),
        names.len().to_string().bright_white().bold()
    );

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
