use crate::completions::{dynamic, shells};
use clap::ArgMatches;

/// Handle `roc completion` (print or install)
pub fn handle(matches: ArgMatches) {
    let shell = matches.get_one::<String>("shell").expect("shell arg");
    let install = matches.get_flag("install");
    let print_path = matches.get_flag("print_path");
    match shell.as_str() {
        "bash" => {
            if install {
                shells::bash::install_completion();
            } else if print_path {
                shells::bash::print_install_path();
            } else {
                shells::bash::print_completions();
            }
        }
        "zsh" => {
            if install {
                shells::zsh::install_completion();
            } else if print_path {
                shells::zsh::print_install_path();
            } else {
                shells::zsh::print_completions();
            }
        }
        "fish" => {
            if install {
                shells::fish::install_completion();
            } else if print_path {
                shells::fish::print_install_path();
            } else {
                shells::fish::print_completions();
            }
        }
        other => eprintln!("Unknown shell: {}", other),
    }
}

/// Handle internal dynamic completion `_complete`
pub fn internal(matches: ArgMatches) {
    // Delegate to the new dynamic completion logic
    dynamic::handle(matches);
}
