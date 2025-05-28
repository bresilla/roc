use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("list", args)) => {
            list::handle(args.clone());
        }
        Some(("set", args)) => {
            set::handle(args.clone());
        }
        Some(("get", args)) => {
            get::handle(args.clone());
        }
        _ => {
            println!("Middleware functionality is currently work in progress.");
            println!("Available subcommands: list, set, get");
        }
    }
}

pub mod list;
pub mod set;
pub mod get;