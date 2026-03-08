use crate::arguments::action::CommonActionArgs;
use crate::commands::cli::handle_anyhow_result;
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

    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    let context = RclGraphContext::new_with_options(common_args.spin_time.as_deref(), common_args.use_sim_time)
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
    handle_anyhow_result(run_command(matches, common_args));
}
