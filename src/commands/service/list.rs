use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::*;
use serde_json::json;

use crate::arguments::service::CommonServiceArgs;
use crate::graph::RclGraphContext;
use crate::shared::ros_names::is_hidden_name;
use crate::ui::{blocks, output, table};

fn run_command(matches: ArgMatches, common_args: CommonServiceArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    let context = RclGraphContext::new_with_options(common_args.spin_time.as_deref(), common_args.use_sim_time)
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    let show_types = matches.get_flag("show_types");
    let count_only = matches.get_flag("count_services");

    let mut items: Vec<(String, Option<String>)> = Vec::new();
    if show_types {
        let pairs = context
            .get_service_names_and_types()
            .map_err(|e| anyhow!("Failed to query services: {}", e))?;
        for (name, ty) in pairs {
            if !matches.get_flag("include_hidden_services") && is_hidden_name(&name) {
                continue;
            }
            items.push((name, Some(ty)));
        }
    } else {
        let services = context
            .get_service_names()
            .map_err(|e| anyhow!("Failed to query services: {}", e))?;
        for name in services {
            if !matches.get_flag("include_hidden_services") && is_hidden_name(&name) {
                continue;
            }
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
                output::print_json(&json!({ "services": [], "count": 0 }))?;
            }
            _ => {
                blocks::eprint_warning(&format!(
                    "No services found. [{}]",
                    RclGraphContext::get_daemon_status()
                ));
            }
        }
        return Ok(());
    }

    let total = items.len();

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Services");
            let headers = if show_types {
                vec!["Service", "Type"]
            } else {
                vec!["Service"]
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
            blocks::print_total(total, "service", "services");
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
            let services = items
                .iter()
                .map(|(name, ty)| {
                    if show_types {
                        json!({ "name": name, "type": ty.as_deref().unwrap_or("unknown") })
                    } else {
                        json!({ "name": name })
                    }
                })
                .collect::<Vec<_>>();
            output::print_json(&json!({ "services": services, "count": total }))?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonServiceArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
