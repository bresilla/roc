use crate::commands::cli::{handle_anyhow_result, required_string};
use anyhow::anyhow;
use anyhow::Result;
use clap::ArgMatches;
use serde_json::json;

use crate::shared::interface_operations;
use crate::ui::{blocks, output};

fn run_command(matches: ArgMatches) -> Result<()> {
    let type_ = required_string(&matches, "type").map_err(|error| anyhow!(error.to_string()))?;
    let all_comments = matches.get_flag("all_comments");
    let no_comments = matches.get_flag("no_comments");
    let output_mode = output::OutputMode::from_matches(&matches);

    let text = interface_operations::show_interface(type_, no_comments, all_comments)?;
    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Interface");
            blocks::print_field("Type", type_);
            if no_comments {
                blocks::print_field("Comments", "hidden");
            } else if all_comments {
                blocks::print_field("Comments", "all");
            }
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
                "all_comments": all_comments,
                "no_comments": no_comments,
                "text": text,
            }))?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
