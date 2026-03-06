use crate::commands::cli::{handle_anyhow_result, required_string};
use anyhow::Result;
use anyhow::anyhow;
use clap::ArgMatches;

use crate::graph::interface_operations;

fn run_command(matches: ArgMatches) -> Result<()> {
    let type_ = required_string(&matches, "type").map_err(|error| anyhow!(error.to_string()))?;
    let no_quotes = matches.get_flag("no_quotes");
    let text = interface_operations::model_interface(type_, no_quotes)?;
    print!("{}", text);
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
