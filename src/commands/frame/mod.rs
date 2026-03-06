use crate::commands::cli::print_error_and_exit;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("echo", args)) => {
            echo::handle(args.clone());
        }
        Some(("list", args)) => {
            list::handle(args.clone());
        }
        Some(("info", args)) => {
            info::handle(args.clone());
        }
        Some(("pub", args)) => {
            pub_::handle(args.clone());
        }
        _ => print_error_and_exit("No frame subcommand selected"),
    }
}

pub mod echo;
pub mod info;
pub mod list;
pub mod pub_;
