use crate::arguments::action::CommonActionArgs;
use crate::commands::cli::handle_anyhow_result;
use crate::graph::{action_operations, RclGraphContext};
use crate::ui::{blocks, output, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use serde_json::json;

fn run_command(matches: ArgMatches, common_args: CommonActionArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    if common_args.use_sim_time {
        blocks::eprint_note("--use-sim-time is not applicable to graph queries");
    }
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    if let Some(spin_time_value) = common_args.spin_time {
        blocks::eprint_note(&format!(
            "--spin-time {} is not yet supported in native mode",
            spin_time_value
        ));
    }

    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    let mut actions = action_operations::get_action_names(&context)?;
    actions.sort();

    let show_types = matches.get_flag("show_types");
    let count_only = matches.get_flag("count_actions");

    let mut items: Vec<(String, Option<String>)> = Vec::new();
    for name in actions {
        if show_types {
            let ty = action_operations::get_action_type(&context, &name)?;
            items.push((name, ty));
        } else {
            items.push((name, None));
        }
    }
    items.sort_by(|a, b| a.0.cmp(&b.0));

    if count_only {
        match output_mode {
            output::OutputMode::Human => {
                println!(
                    "{} {}",
                    "Total:".bright_green(),
                    items.len().to_string().bright_white().bold()
                );
            }
            output::OutputMode::Plain => println!("{}", items.len()),
            output::OutputMode::Json => output::print_json(&json!({ "count": items.len() }))?,
        }
        return Ok(());
    }

    if items.is_empty() {
        match output_mode {
            output::OutputMode::Json => {
                output::print_json(&json!({ "actions": [], "count": 0 }))?;
            }
            _ => {
                blocks::eprint_warning(&format!(
                    "No actions found. [{}]",
                    RclGraphContext::get_daemon_status()
                ));
            }
        }
        return Ok(());
    }

    let total = items.len();

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Actions");
            let headers = if show_types {
                vec!["Action", "Type"]
            } else {
                vec!["Action"]
            };
            let rows = items
                .iter()
                .map(|(name, ty)| {
                    let mut row = vec![name.bright_cyan().to_string()];
                    if show_types {
                        row.push(
                            ty.as_ref()
                                .map(|value| value.green().to_string())
                                .unwrap_or_else(|| "unknown".red().to_string()),
                        );
                    }
                    row
                })
                .collect();
            table::print_table(&headers, rows);
            blocks::print_total(total, "action", "actions");
        }
        output::OutputMode::Plain => {
            for (name, ty) in &items {
                if show_types {
                    println!("{name}\t{}", ty.as_deref().unwrap_or("unknown"));
                } else {
                    println!("{name}");
                }
            }
        }
        output::OutputMode::Json => {
            let actions = items
                .iter()
                .map(|(name, ty)| {
                    if show_types {
                        json!({ "name": name, "type": ty.as_deref().unwrap_or("unknown") })
                    } else {
                        json!({ "name": name })
                    }
                })
                .collect::<Vec<_>>();
            output::print_json(&json!({ "actions": actions, "count": total }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonActionArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
