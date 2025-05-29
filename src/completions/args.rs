use clap::{Command, Arg};

/// The `roc completion` subcommand arguments
pub fn cmd() -> Command {
    Command::new("completion")
        .about("Generate shell completion scripts for roc")
        .arg(Arg::new("shell")
            .help("The shell to generate completions for")
            .required(true)
            .value_parser(["bash", "zsh", "fish"]))
        .arg(Arg::new("install")
            .long("install")
            .help("Install the generated completions to a default location"))
}
