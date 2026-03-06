use crate::commands::cli::{handle_anyhow_result, required_string};
use anyhow::Result;
use anyhow::anyhow;
use clap::ArgMatches;

use crate::graph::interface_operations;

fn run_command(matches: ArgMatches) -> Result<()> {
    let package_name =
        required_string(&matches, "package_name").map_err(|error| anyhow!(error.to_string()))?;
    for t in interface_operations::list_interfaces_in_package(package_name)? {
        println!("{}", t);
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
