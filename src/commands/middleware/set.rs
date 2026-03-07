use anyhow::{anyhow, Result};
use crate::commands::cli::handle_anyhow_result;
use crate::commands::middleware::{discover_implementations, export_command};
use crate::ui::{blocks, output};
use clap::ArgMatches;
use serde_json::json;

fn run_command(matches: ArgMatches) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let implementation = matches
        .get_one::<String>("IMPLEMENTATION")
        .ok_or_else(|| anyhow!("IMPLEMENTATION is required"))?;

    if !implementation
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(anyhow!(
            "Invalid middleware implementation '{}'",
            implementation
        ));
    }

    let discovered = discover_implementations();
    if !discovered.is_empty() && !discovered.iter().any(|item| item == implementation) {
        return Err(anyhow!(
            "Unknown middleware implementation '{}'. Available: {}",
            implementation,
            discovered.join(", ")
        ));
    }

    let shell_command = export_command(implementation);
    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Middleware Selection");
            blocks::print_field("Implementation", implementation);
            blocks::print_field("Shell Command", &shell_command);
            blocks::print_note("Run this command in your shell before starting ROS processes");
        }
        output::OutputMode::Plain => {
            println!("{shell_command}");
        }
        output::OutputMode::Json => {
            output::print_json(&json!({
                "implementation": implementation,
                "shell_command": shell_command,
                "persistent": false,
            }))?;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
