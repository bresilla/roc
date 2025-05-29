use clap::{Command, Arg};

/// Internal callback for dynamic completion (invoked by shell scripts)
pub fn cmd() -> Command {
    Command::new("_complete")
        .about("Internal command for shell completion - do not use directly")
        .hide(true)
        .arg(
            Arg::new("command")
                .help("The command being completed")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("subcommand")
                .help("The subcommand being completed")
                .required(false)
                .index(2)
        )
        .arg(
            Arg::new("position")
                .help("The argument position to complete")
                .required(false)
                .index(3)
        )
}
