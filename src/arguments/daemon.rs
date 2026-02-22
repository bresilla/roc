use clap::Command;

pub fn cmd() -> Command {
    Command::new("daemon")
        .about("Deamon and bridge subcommands [WIP]")
        .aliases(&["d"])
        .subcommand_required(true)
        .arg_required_else_help(true)
}
