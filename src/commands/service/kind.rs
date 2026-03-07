use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::json;

use crate::arguments::service::CommonServiceArgs;
use crate::graph::RclGraphContext;
use crate::ui::{blocks, output, table};

fn run_command(matches: ArgMatches, common_args: CommonServiceArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
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

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Service");
            blocks::print_field("Name", service_name);
            if types.len() == 1 {
                blocks::print_field("Type", &types[0]);
            } else {
                println!();
                blocks::print_section("Types");
                let rows = types.iter().map(|ty| vec![ty.clone()]).collect();
                table::print_table(&["Type"], rows);
            }
        }
        output::OutputMode::Plain => {
            for ty in &types {
                println!("{ty}");
            }
        }
        output::OutputMode::Json => {
            let count = types.len();
            output::print_json(&json!({ "name": service_name, "types": types, "count": count }))?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
