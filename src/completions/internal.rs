use clap::{Arg, Command};

/// Internal callback for dynamic completion (invoked by shell scripts)
pub fn cmd() -> Command {
    Command::new("_complete")
        .about("Internal command for shell completion - do not use directly")
        .hide(true)
        .arg(
            Arg::new("command")
                .help("The command being completed")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("subcommand")
                .help("The subcommand being completed")
                .required(false)
                .index(2),
        )
        .arg(
            Arg::new("subsubcommand")
                .help("The sub-subcommand being completed")
                .required(false)
                .index(3),
        )
        .arg(
            Arg::new("position")
                .help("The argument position to complete")
                .required(false)
                .index(4),
        )
        .arg(
            Arg::new("current_args")
                .help("Current arguments on the command line")
                .num_args(0..)
                .index(5),
        )
}
