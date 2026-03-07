use anyhow::{Result, anyhow};
use clap::ArgMatches;

use crate::arguments::service::CommonServiceArgs;
use crate::graph::RclGraphContext;

fn run_command(matches: ArgMatches, common_args: CommonServiceArgs) -> Result<()> {
    let service_name = matches
        .get_one::<String>("service_name")
        .ok_or_else(|| anyhow!("service_name is required"))?;

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

    let mut types = Vec::new();
    for (name, ty) in pairs {
        if name == *service_name {
            types.push(ty);
        }
    }

    if types.is_empty() {
        return Err(anyhow!("Service '{}' not found", service_name));
    }

    types.sort();
    types.dedup();

    // Match ros2 CLI: print each type on its own line (usually only one).
    for ty in types {
        println!("{}", ty);
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
