use clap::ArgMatches;
use crate::completion;

pub fn handle(matches: ArgMatches) {
    let shell = matches.get_one::<crate::arguments::completion::Shell>("shell").unwrap();
    let install = matches.get_flag("install");
    
    if install {
        install_completion(shell.clone());
    } else {
        generate_completion(shell.clone());
    }
}

fn generate_completion(shell: crate::arguments::completion::Shell) {
    // Use advanced completions for better ROS integration
    completion::print_advanced_completions(shell.into());
}

fn install_completion(shell: crate::arguments::completion::Shell) {
    use std::path::PathBuf;
    use std::fs;
    
    let shell_name = match shell {
        crate::arguments::completion::Shell::Bash => "bash",
        crate::arguments::completion::Shell::Zsh => "zsh", 
        crate::arguments::completion::Shell::Fish => "fish",
    };
    
    // Helper function to find installation path
    let find_install_path = |locations: Vec<Option<PathBuf>>| -> Option<PathBuf> {
        for location in locations {
            if let Some(path) = location {
                if let Some(parent) = path.parent() {
                    if parent.exists() || fs::create_dir_all(parent).is_ok() {
                        return Some(path);
                    }
                }
            }
        }
        None
    };
    
    // Determine installation path
    let install_path = match shell {
        crate::arguments::completion::Shell::Bash => {
            find_install_path(vec![
                Some(PathBuf::from("/usr/share/bash-completion/completions/roc")),
                std::env::home_dir().map(|h| h.join(".bash_completion.d/roc")),
                std::env::home_dir().map(|h| h.join(".local/share/bash-completion/completions/roc")),
            ])
        },
        crate::arguments::completion::Shell::Zsh => {
            find_install_path(vec![
                std::env::home_dir().map(|h| h.join(".zfunc/_roc")),
                Some(PathBuf::from("/usr/local/share/zsh/site-functions/_roc")),
                std::env::home_dir().map(|h| h.join(".local/share/zsh/site-functions/_roc")),
            ])
        },
        crate::arguments::completion::Shell::Fish => {
            find_install_path(vec![
                std::env::home_dir().map(|h| h.join(".config/fish/completions/roc.fish")),
                Some(PathBuf::from("/usr/share/fish/completions/roc.fish")),
            ])
        },
    };
    
    match install_path {
        Some(path) => {
            println!("Installing {} completions to: {}", shell_name, path.display());
            
            // Generate completion script
            let mut output = Vec::new();
            
            // Use advanced completions for better ROS integration
            let completion_script = match shell {
                crate::arguments::completion::Shell::Bash => {
                    completion::get_bash_completion_script()
                },
                crate::arguments::completion::Shell::Zsh => {
                    completion::get_zsh_completion_script()
                },
                crate::arguments::completion::Shell::Fish => {
                    completion::get_fish_completion_script()
                },
            };
            
            output.extend_from_slice(completion_script.as_bytes());
            
            // Write to file
            match fs::write(&path, output) {
                Ok(_) => {
                    println!("✅ Completions installed successfully!");
                    
                    // Provide shell-specific instructions
                    match shell {
                        crate::arguments::completion::Shell::Bash => {
                            println!("To enable completions, add this to your ~/.bashrc:");
                            println!("  source {}", path.display());
                        },
                        crate::arguments::completion::Shell::Zsh => {
                            println!("To enable completions, add this to your ~/.zshrc:");
                            if path.parent().unwrap().to_string_lossy().contains(".zfunc") {
                                println!("  fpath=(~/.zfunc $fpath)");
                                println!("  autoload -U compinit && compinit");
                            } else {
                                println!("  autoload -U compinit && compinit");
                            }
                        },
                        crate::arguments::completion::Shell::Fish => {
                            println!("Completions should be automatically available in new fish sessions.");
                        },
                    }
                },
                Err(e) => {
                    eprintln!("❌ Failed to install completions: {}", e);
                    eprintln!("Try running with sudo or use manual installation:");
                    eprintln!("  roc completion {} > completion_file", shell_name);
                }
            }
        },
        None => {
            eprintln!("❌ Could not determine installation location for {} completions", shell_name);
            eprintln!("Use manual installation:");
            eprintln!("  roc completion {} > completion_file", shell_name);
        }
    }
}
