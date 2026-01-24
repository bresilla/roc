pub mod list;
pub mod info;

use clap::ArgMatches;
use crate::arguments::node::CommonNodeArgs;

pub fn handle(matches: ArgMatches){
    // Extract common node arguments from the parent command
    let common_args = CommonNodeArgs::from_matches(&matches);

    match matches.subcommand() {
        Some(("info", args)) => {
            info::handle(args.clone(), common_args);
        }
        Some(("list", args)) => {
            list::handle(args.clone(), common_args);
        }
        _ => unreachable!("UNREACHABLE"),
    }
}