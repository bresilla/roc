pub mod call;
pub mod find;
pub mod kind;
pub mod list;

use crate::arguments::service::CommonServiceArgs;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    // Extract common service arguments from the parent command
    let common_args = CommonServiceArgs::from_matches(&matches);

    match matches.subcommand() {
        Some(("call", args)) => {
            call::handle(args.clone(), common_args);
        }
        Some(("find", args)) => {
            find::handle(args.clone(), common_args);
        }
        Some(("list", args)) => {
            list::handle(args.clone(), common_args);
        }
        Some(("kind", args)) => {
            kind::handle(args.clone(), common_args);
        }
        _ => unreachable!("UNREACHABLE"),
    }
}
