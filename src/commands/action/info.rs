use crate::arguments::action::CommonActionArgs;
use crate::graph::{action_operations, RclGraphContext};
use crate::ui::{blocks, output};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use serde_json::json;

fn run_command(matches: ArgMatches, common_args: CommonActionArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let action_name = matches
        .get_one::<String>("action_name")
        .ok_or_else(|| anyhow!("action_name is required"))?;

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

    let ty = action_operations::get_action_type(&context, action_name)?;
    let ty = ty.unwrap_or_else(|| "<unknown>".to_string());

    match output_mode {
        output::OutputMode::Human => {
            if matches.get_flag("show_types") {
                println!(
                    "{} {}",
                    action_name.bright_cyan(),
                    format!("[{}]", ty).bright_green()
                );
                return Ok(());
            }

            blocks::print_section("Action");
            blocks::print_field("Name", action_name.bright_cyan());
            blocks::print_field("Type", ty.bright_green());
        }
        output::OutputMode::Plain => {
            if matches.get_flag("show_types") {
                println!("{action_name}\t{ty}");
            } else {
                output::print_plain_section("Action");
                output::print_plain_field("Name", action_name);
                output::print_plain_field("Type", &ty);
            }
        }
        output::OutputMode::Json => {
            output::print_json(&json!({ "name": action_name, "type": ty }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonActionArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
