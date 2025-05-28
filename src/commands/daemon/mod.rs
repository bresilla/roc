use clap::ArgMatches;

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
        _ => {
            println!("Daemon functionality is currently work in progress.");
            println!("Available subcommands: start, stop, status");
        }
    }
}

pub mod start;
pub mod stop;
pub mod status;