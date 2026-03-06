use crate::commands::cli::{handle_anyhow_result, required_string};
use anyhow::Result;
use anyhow::anyhow;
use clap::ArgMatches;

use crate::graph::interface_operations;

fn run_command(matches: ArgMatches) -> Result<()> {
    let type_ = required_string(&matches, "type").map_err(|error| anyhow!(error.to_string()))?;
    let all_comments = matches.get_flag("all_comments");
    let no_comments = matches.get_flag("no_comments");

    let text = interface_operations::show_interface(type_, no_comments, all_comments)?;
    print!("{}", text);
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
