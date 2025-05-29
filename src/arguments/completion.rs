use clap::{Command, Arg, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl From<Shell> for clap_complete::Shell {
    fn from(shell: Shell) -> Self {
        match shell {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
        }
    }
}

pub fn cmd() -> Command {
    Command::new("completion")
        .about("Generate shell completion scripts")
        .long_about("Generate shell completion scripts for roc. The output can be sourced by your shell to enable autocompletion.")
        .arg(
            Arg::new("shell")
                .help("The shell to generate completions for")
                .value_parser(clap::value_parser!(Shell))
                .required(true)
        )
        .arg(
            Arg::new("install")
                .long("install")
                .help("Install completions to the default location for the shell")
                .action(clap::ArgAction::SetTrue)
        )
}
