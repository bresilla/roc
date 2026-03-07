use clap::{Arg, Command};

/// The `roc completion` subcommand arguments
pub fn cmd() -> Command {
    Command::new("completion")
        .about("Generate shell completion scripts for roc")
        .arg(
            Arg::new("shell")
                .help("The shell to generate completions for")
                .required(true)
                .value_parser(["bash", "zsh", "fish"]),
        )
        .arg(
            Arg::new("install")
                .long("install")
                .help("Install the generated completions to a default location")
                .conflicts_with("print_path")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("print_path")
                .long("print-path")
                .help("Print the default installation path for the selected shell")
                .conflicts_with("install")
                .action(clap::ArgAction::SetTrue),
        )
}
