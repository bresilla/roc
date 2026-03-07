use anyhow::{Result, anyhow};
use clap::ArgMatches;
use colored::*;

use crate::arguments::service::CommonServiceArgs;
use crate::graph::RclGraphContext;

fn run_command(matches: ArgMatches, common_args: CommonServiceArgs) -> Result<()> {
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

    let show_types = matches.get_flag("show_types");
    let count_only = matches.get_flag("count_services");

    let mut items: Vec<(String, Option<String>)> = Vec::new();
    if show_types {
        let pairs = context
            .get_service_names_and_types()
            .map_err(|e| anyhow!("Failed to query services: {}", e))?;
        for (name, ty) in pairs {
            items.push((name, Some(ty)));
        }
    } else {
        let services = context
            .get_service_names()
            .map_err(|e| anyhow!("Failed to query services: {}", e))?;
        for name in services {
            items.push((name, None));
        }
    }

    items.sort_by(|a, b| a.0.cmp(&b.0));

    if count_only {
        println!(
            "{} {}",
            "Total:".bright_green(),
            items.len().to_string().bright_white().bold()
        );
        return Ok(());
    }

    if items.is_empty() {
        eprintln!(
            "{} {}",
            "No services found.".yellow(),
            format!("[{}]", RclGraphContext::get_daemon_status()).bright_black()
        );
        return Ok(());
    }

    let total = items.len();

    println!("{}", "Available Services:".bright_yellow().bold());
    for (name, ty) in &items {
        match ty {
            Some(t) => println!("  {} {}", name.bright_cyan(), format!("[{}]", t).green()),
            None => println!("  {}", name.bright_cyan()),
        }
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
