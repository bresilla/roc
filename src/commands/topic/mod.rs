pub mod bw;
pub mod delay;
pub mod echo;
pub mod find;
pub mod hz;
pub mod info;
pub mod kind;
pub mod list;
pub mod pub_;

use crate::arguments::topic::CommonTopicArgs;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    // Extract common topic arguments from the parent command
    let common_args = CommonTopicArgs::from_matches(&matches);

    match matches.subcommand() {
        Some(("list", args)) => {
            list::handle(args.clone(), common_args);
        }
        Some(("hz", args)) => {
            hz::handle(args.clone(), common_args);
        }
        Some(("echo", args)) => {
            echo::handle(args.clone(), common_args);
        }
        Some(("pub", args)) => {
            pub_::handle(args.clone(), common_args);
        }
        Some(("info", args)) => {
            info::handle(args.clone(), common_args);
        }
        Some(("kind", args)) => {
            kind::handle(args.clone(), common_args);
        }
        Some(("bw", args)) => {
            bw::handle(args.clone(), common_args);
        }
        Some(("find", args)) => {
            find::handle(args.clone(), common_args);
        }
        Some(("delay", args)) => {
            delay::handle(args.clone(), common_args);
        }
        _ => unreachable!("UNREACHABLE"),
    }
}
