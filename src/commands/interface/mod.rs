use crate::commands::cli::print_error_and_exit;
pub mod all;
pub mod list;
pub mod model;
pub mod package;
pub mod show;

use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("list", args)) => {
            list::handle(args.clone());
        }
        Some(("package", args)) => {
            package::handle(args.clone());
        }
        Some(("all", args)) => {
            all::handle(args.clone());
        }
        Some(("model", args)) => {
            model::handle(args.clone());
        }
        Some(("show", args)) => {
            show::handle(args.clone());
        }
        _ => print_error_and_exit("No interface subcommand selected"),
    }
}
