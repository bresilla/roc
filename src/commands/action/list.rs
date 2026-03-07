use crate::arguments::action::CommonActionArgs;
use crate::graph::{action_operations, RclGraphContext};
use crate::ui::{blocks, table};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;

fn run_command(matches: ArgMatches, common_args: CommonActionArgs) -> Result<()> {
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
            "No actions found.".yellow(),
            format!("[{}]", RclGraphContext::get_daemon_status()).bright_black()
        );
        return Ok(());
    }

    let total = items.len();

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

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonActionArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
