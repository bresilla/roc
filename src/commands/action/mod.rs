pub mod goal;
pub mod info;
pub mod list;

use crate::arguments::action::CommonActionArgs;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    // Extract common action arguments from the parent command
    let common_args = CommonActionArgs::from_matches(&matches);

    match matches.subcommand() {
        Some(("info", args)) => {
            info::handle(args.clone(), common_args);
        }
        Some(("list", args)) => {
            list::handle(args.clone(), common_args);
        }
        Some(("goal", args)) => {
            goal::handle(args.clone(), common_args);
        }
        _ => unreachable!("UNREACHABLE"),
    }
}
