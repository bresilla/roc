pub mod info;
pub mod list;

use crate::arguments::node::CommonNodeArgs;
use crate::commands::cli::print_error_and_exit;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    // Extract common node arguments from the parent command
    let common_args = CommonNodeArgs::from_matches(&matches);

    match matches.subcommand() {
        Some(("info", args)) => {
            info::handle(args.clone(), common_args);
        }
        Some(("list", args)) => {
            list::handle(args.clone(), common_args);
        }
        _ => print_error_and_exit("No node subcommand selected"),
    }
}
