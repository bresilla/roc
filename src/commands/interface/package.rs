use crate::commands::cli::{handle_anyhow_result, required_string};
use anyhow::anyhow;
use anyhow::Result;
use clap::ArgMatches;
use serde_json::json;

use crate::shared::interface_operations;
use crate::ui::{blocks, output, table};

fn run_command(matches: ArgMatches) -> Result<()> {
    let package_name =
        required_string(&matches, "package_name").map_err(|error| anyhow!(error.to_string()))?;
    let output_mode = output::OutputMode::from_matches(&matches);
    let items = interface_operations::list_interfaces_in_package(package_name)?;

    match output_mode {
        output::OutputMode::Human => {
            if items.is_empty() {
                blocks::eprint_warning(&format!("No interfaces found in package {package_name}."));
                return Ok(());
            }
            blocks::print_section("Interfaces");
            blocks::print_field("Package", package_name);
            println!();
            let rows = items.iter().map(|item| vec![item.clone()]).collect();
            table::print_table(&["Interface"], rows);
            blocks::print_total(items.len(), "interface", "interfaces");
        }
        output::OutputMode::Plain => {
            for item in &items {
                println!("{item}");
            }
        }
        output::OutputMode::Json => {
            let count = items.len();
            output::print_json(
                &json!({ "package": package_name, "interfaces": items, "count": count }),
            )?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
