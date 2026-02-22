use clap::Command;

pub fn cmd() -> Command {
    Command::new("middleware")
        .about("Various middleware subcommands [WIP]")
        .aliases(&["m"])
        .subcommand_required(true)
        .arg_required_else_help(true)
}
