use clap::ArgMatches;
use crate::completions::{bash, zsh, fish, dynamic};

/// Handle `roc completion` (print or install)
pub fn handle(matches: ArgMatches) {
    let shell = matches.get_one::<String>("shell").expect("shell arg");
    let install = matches.get_flag("install");
    match shell.as_str() {
        "bash" => {
            if install {
                bash::install_completion();
            } else {
                bash::print_completions();
            }
        }
        "zsh" => {
            if install {
                zsh::install_completion();
            } else {
                zsh::print_completions();
            }
        }
        "fish" => {
            if install {
                fish::install_completion();
            } else {
                fish::print_completions();
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
