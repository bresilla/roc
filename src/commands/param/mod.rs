pub mod describe;
pub mod export;
pub mod get;
pub mod import;
pub mod list;
pub mod remove;
pub mod set;

use crate::arguments::param::CommonParamArgs;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    // Extract common param arguments from the parent command
    let common_args = CommonParamArgs::from_matches(&matches);

    match matches.subcommand() {
        Some(("get", args)) => {
            get::handle(args.clone(), common_args);
        }
        Some(("list", args)) => {
            list::handle(args.clone(), common_args);
        }
        Some(("set", args)) => {
            set::handle(args.clone(), common_args);
        }
        Some(("export", args)) => {
            export::handle(args.clone(), common_args);
        }
        Some(("remove", args)) => {
            remove::handle(args.clone(), common_args);
        }
        Some(("describe", args)) => {
            describe::handle(args.clone(), common_args);
        }
        Some(("import", args)) => {
            import::handle(args.clone(), common_args);
        }
        _ => unreachable!("UNREACHABLE"),
    }
}
