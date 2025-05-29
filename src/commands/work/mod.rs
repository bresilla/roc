use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("create", args)) => {
            create::handle(args.clone());
        }
        Some(("list", args)) => {
            list::handle(args.clone());
        }
        Some(("info", args)) => {
            info::handle(args.clone());
        }
        Some(("build", args)) => {
            build::handle(args.clone());
        }
        _ => unreachable!("UNREACHABLE"),
    }
}

pub mod create;
pub mod list;
pub mod info;
pub mod build_command;
pub mod build;