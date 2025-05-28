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
        _ => {
            println!("ROSbag functionality is currently work in progress.");
            println!("Available subcommands: record, play, info, list");
        }
    }
}

pub mod record;
pub mod play;
pub mod info;
pub mod list;