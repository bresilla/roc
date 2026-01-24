use anyhow::{anyhow, Result};
use clap::ArgMatches;

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

    if matches.get_flag("show_types") {
        let pairs = context
            .get_service_names_and_types()
            .map_err(|e| anyhow!("Failed to query services: {}", e))?;
        for (name, ty) in pairs {
            println!("{} [{}]", name, ty);
        }
        return Ok(());
    }

    let services = context
        .get_service_names()
        .map_err(|e| anyhow!("Failed to query services: {}", e))?;

    if matches.get_flag("count_services") {
        println!("{}", services.len());
        return Ok(());
    }

    for name in services {
        println!("{}", name);
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
