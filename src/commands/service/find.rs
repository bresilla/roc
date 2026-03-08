use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::json;

use crate::arguments::service::CommonServiceArgs;
use crate::graph::RclGraphContext;
use crate::ui::{blocks, output, table};

fn run_command(matches: ArgMatches, common_args: CommonServiceArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let service_type = matches
        .get_one::<String>("service_type")
        .ok_or_else(|| anyhow!("service_type is required"))?;

    if matches.get_flag("include_hidden_services") {
        blocks::eprint_note("--include-hidden-services is not yet supported in native mode");
    }
    if common_args.use_sim_time {
        blocks::eprint_note("--use-sim-time is not applicable to graph queries");
    }
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    let context = RclGraphContext::new_with_spin_time(common_args.spin_time.as_deref())
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
        match output_mode {
            output::OutputMode::Human => blocks::print_total(names.len(), "service", "services"),
            output::OutputMode::Plain => println!("{}", names.len()),
            output::OutputMode::Json => {
                let count = names.len();
                output::print_json(
                    &json!({ "service_type": service_type, "services": names, "count": count }),
                )?;
            }
        }
        return Ok(());
    }

    match output_mode {
        output::OutputMode::Human => {
            if names.is_empty() {
                blocks::eprint_warning(&format!("No services found for type {service_type}"));
                return Ok(());
            }

            blocks::print_section("Services");
            blocks::print_field("Requested Type", service_type);
            println!();
            let rows = names.iter().map(|name| vec![name.clone()]).collect();
            table::print_table(&["Service"], rows);
            blocks::print_total(names.len(), "service", "services");
        }
        output::OutputMode::Plain => {
            for name in &names {
                println!("{name}");
            }
        }
        output::OutputMode::Json => {
            let count = names.len();
            output::print_json(
                &json!({ "service_type": service_type, "services": names, "count": count }),
            )?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
