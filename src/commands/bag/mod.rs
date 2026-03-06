use crate::commands::cli::print_error_and_exit;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("record", args)) => {
            record::handle(args.clone());
        }
        Some(("play", args)) => {
            play::handle(args.clone());
        }
        Some(("info", args)) => {
            info::handle(args.clone());
        }
        Some(("list", args)) => {
            list::handle(args.clone());
        }
        _ => print_error_and_exit("No bag subcommand selected"),
    }
}

pub mod info;
pub mod list;
pub mod play;
pub mod record;
