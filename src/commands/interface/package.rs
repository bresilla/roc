use anyhow::Result;
use clap::ArgMatches;

use crate::graph::interface_operations;

fn run_command(matches: ArgMatches) -> Result<()> {
    let package_name = matches.get_one::<String>("package_name").unwrap();
    for t in interface_operations::list_interfaces_in_package(package_name)? {
        println!("{}", t);
    }
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
