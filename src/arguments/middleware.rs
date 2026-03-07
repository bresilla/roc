use crate::ui::output;
use clap::{arg, Command};

pub fn cmd() -> Command {
    Command::new("middleware")
        .about("Various middleware subcommands")
        .aliases(&["m"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
                .about("List discovered RMW implementations")
                .arg(output::arg()),
        )
        .subcommand(
            Command::new("get")
                .about("Show the current RMW implementation")
                .arg(output::arg()),
        )
        .subcommand(
            Command::new("set")
                .about("Print a shell command to select an RMW implementation")
                .arg(arg!(<IMPLEMENTATION> "RMW implementation to select"))
                .arg(output::arg()),
        )
}
