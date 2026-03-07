use crate::ui::output;
use clap::Command;

pub fn cmd() -> Command {
    Command::new("daemon")
        .about("Daemon and bridge subcommands")
        .aliases(&["d"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Show daemon startup compatibility status")
                .arg(output::arg()),
        )
        .subcommand(
            Command::new("stop")
                .about("Show daemon shutdown compatibility status")
                .arg(output::arg()),
        )
        .subcommand(
            Command::new("status")
                .about("Show daemon status for native roc commands")
                .arg(output::arg()),
        )
}
