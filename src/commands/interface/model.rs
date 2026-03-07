use crate::commands::cli::{handle_anyhow_result, required_string};
use anyhow::anyhow;
use anyhow::Result;
use clap::ArgMatches;
use serde_json::json;

use crate::shared::interface_operations;
use crate::ui::{blocks, output};

fn run_command(matches: ArgMatches) -> Result<()> {
    let type_ = required_string(&matches, "type").map_err(|error| anyhow!(error.to_string()))?;
    let no_quotes = matches.get_flag("no_quotes");
    let output_mode = output::OutputMode::from_matches(&matches);
    let text = interface_operations::model_interface(type_, no_quotes)?;
    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Interface Model");
            blocks::print_field("Type", type_);
            blocks::print_field("Quoted", if no_quotes { "no" } else { "yes" });
            println!();
            print!("{text}");
            if !text.ends_with('\n') {
                println!();
            }
        }
        output::OutputMode::Plain => print!("{text}"),
        output::OutputMode::Json => {
            output::print_json(&json!({
                "type": type_,
                "no_quotes": no_quotes,
                "text": text,
            }))?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
