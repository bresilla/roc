use anyhow::Result;
use clap::ArgMatches;

use crate::graph::interface_operations;

fn run_command(matches: ArgMatches) -> Result<()> {
    let type_ = matches.get_one::<String>("type").unwrap();
    let no_quotes = matches.get_flag("no_quotes");
    let text = interface_operations::model_interface(type_, no_quotes)?;
    print!("{}", text);
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    if let Err(e) = run_command(matches) {
        if let Some(ioe) = e.downcast_ref::<std::io::Error>() {
            if ioe.kind() == std::io::ErrorKind::BrokenPipe {
                return;
            }
        }
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
