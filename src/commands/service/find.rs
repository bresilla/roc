use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;

use crate::arguments::service::CommonServiceArgs;
use crate::graph::RclGraphContext;

fn run_command(matches: ArgMatches, common_args: CommonServiceArgs) -> Result<()> {
    let service_type = matches
        .get_one::<String>("service_type")
        .ok_or_else(|| anyhow!("service_type is required"))?;

    if matches.get_flag("include_hidden_services") {
        eprintln!("Note: --include-hidden-services is not yet supported in native mode");
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

    let pairs = context
        .get_service_names_and_types()
        .map_err(|e| anyhow!("Failed to query services: {}", e))?;

    let mut names = Vec::new();
    for (name, ty) in pairs {
        if ty == *service_type {
            names.push(name);
        }
    }
    names.sort();
    names.dedup();

    if matches.get_flag("count_services") {
        println!(
            "{} {}",
            "Total:".bright_green(),
            names.len().to_string().bright_white().bold()
        );
        return Ok(());
    }

    if names.is_empty() {
        eprintln!(
            "{} {}",
            "No services found for type".yellow(),
            format!("[{}]", service_type).bright_cyan()
        );
        return Ok(());
    }

    let total = names.len();

    println!(
        "{} {}",
        "Services with type".bright_yellow().bold(),
        format!("[{}]", service_type).bright_cyan()
    );
    for name in &names {
        println!("  {}", name.bright_cyan());
    }
    println!();
    println!(
        "{} {} services found",
        "Total:".bright_green(),
        total.to_string().bright_white().bold()
    );
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
