use clap::ArgMatches;
use crate::commands::cli::print_error_and_exit;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("start", args)) => {
            start::handle(args.clone());
        }
        Some(("stop", args)) => {
            stop::handle(args.clone());
        }
        Some(("status", args)) => {
            status::handle(args.clone());
        }
        _ => print_error_and_exit("No daemon subcommand selected"),
    }
}

pub mod start;
pub mod status;
pub mod stop;
