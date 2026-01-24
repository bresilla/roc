pub mod list;
pub mod info;
pub mod goal;

use clap::ArgMatches;
use crate::arguments::action::CommonActionArgs;

pub fn handle(matches: ArgMatches){
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