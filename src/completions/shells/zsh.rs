use std::path::PathBuf;
use std::{env, fs};

/// Zsh completion script with corrected word indexing and dynamic dispatch.
const SCRIPT: &str = r#"
#compdef roc

_roc_dynamic_lines() {
    local -a items
    items=("${(@f)$("$@" 2>/dev/null)}")
    print -l -- $items
}

_roc() {
    local curcontext="$curcontext" state line
    typeset -A opt_args

    _arguments -C \
        '1:command:->command' \
        '*::arg:->args'

    case "$state" in
        command)
            local commands=(
                'action:Various action subcommands'
                'topic:Various topic subcommands'
                'service:Various service subcommands'
                'param:Various param subcommands'
                'node:Various node subcommands'
                'interface:Various interface subcommands'
                'frame:Various transform subcommands'
                'run:Run an executable'
                'launch:Launch a launch file'
                'work:Packages and workspace'
                'bag:ROS bag tools'
                'daemon:Daemon and bridge'
                'middleware:Middleware settings'
                'completion:Generate shell completions'
            )
            _describe 'command' commands
            return
            ;;
        args)
            case "$words[2]" in
                launch)
                    case $CURRENT in
                        3) _describe 'packages' "$(_roc_dynamic_lines roc _complete launch '' '' 1)" ;;
                        4) _describe 'launch files' "$(_roc_dynamic_lines roc _complete launch '' '' 2 "$words[3]")" ;;
                    esac
                    ;;
                run)
                    case $CURRENT in
                        3) _describe 'packages' "$(_roc_dynamic_lines roc _complete run '' '' 1)" ;;
                        4) _describe 'executables' "$(_roc_dynamic_lines roc _complete run '' '' 2 "$words[3]")" ;;
                    esac
                    ;;
                topic)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete topic '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete topic "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "pub" ]]; then
                                _describe 'message types' "$(_roc_dynamic_lines roc _complete topic pub '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                service)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete service '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete service "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "call" ]]; then
                                _describe 'service types' "$(_roc_dynamic_lines roc _complete service call '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                param)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete param '' '' 1)" ;;
                        4) _describe 'nodes' "$(_roc_dynamic_lines roc _complete param "$words[3]" '' 1)" ;;
                        5)
                            case "$words[3]" in
                                get|set|describe|remove)
                                    _describe 'parameters' "$(_roc_dynamic_lines roc _complete param "$words[3]" '' 2 "$words[4]")"
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                node)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete node '' '' 1)" ;;
                        4) _describe 'nodes' "$(_roc_dynamic_lines roc _complete node "$words[3]" '' 1)" ;;
                    esac
                    ;;
                action)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete action '' '' 1)" ;;
                        4) _describe 'actions' "$(_roc_dynamic_lines roc _complete action "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "goal" ]]; then
                                _describe 'action types' "$(_roc_dynamic_lines roc _complete action goal '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                interface)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete interface '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete interface "$words[3]" '' 1)" ;;
                    esac
                    ;;
                bag)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete bag '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete bag "$words[3]" '' 1)" ;;
                    esac
                    ;;
                work)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete work '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete work "$words[3]" '' 1)" ;;
                    esac
                    ;;
                frame)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete frame '' '' 1)" ;;
                        4) _describe 'frames' "$(_roc_dynamic_lines roc _complete frame "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "echo" ]]; then
                                _describe 'frames' "$(_roc_dynamic_lines roc _complete frame echo '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                daemon)
                    [[ $CURRENT -eq 3 ]] && _describe 'subcommands' "$(_roc_dynamic_lines roc _complete daemon '' '' 1)"
                    ;;
                middleware)
                    [[ $CURRENT -eq 3 ]] && _describe 'subcommands' "$(_roc_dynamic_lines roc _complete middleware '' '' 1)"
                    ;;
                completion)
                    case $CURRENT in
                        3) _describe 'shells' "bash zsh fish" ;;
                        *) _arguments '--install[Install completions to a default location]' ;;
                    esac
                    ;;
            esac
            ;;
    esac
}

compdef _roc roc
"#;

pub fn print_completions() {
    println!("{}", SCRIPT);
}

pub fn install_completion() {
    let install_path = find_install_path(vec![
        env::home_dir().map(|h| h.join(".zfunc/_roc")),
        Some(PathBuf::from("/usr/local/share/zsh/site-functions/_roc")),
        env::home_dir().map(|h| h.join(".local/share/zsh/site-functions/_roc")),
    ]);
    match install_path {
        Some(path) => {
            println!("Installing zsh completions to: {}", path.display());
            match fs::write(&path, SCRIPT) {
                Ok(_) => {
                    println!("✅ Completions installed successfully!");
                    println!("To enable completions, add this to your ~/.zshrc:");
                    if path
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or("")
                        .contains(".zfunc")
                    {
                        println!("  fpath=(~/.zfunc $fpath)");
                    }
                    println!("  autoload -U compinit && compinit");
                }
                Err(e) => {
                    eprintln!("❌ Failed to install completions: {}", e);
                    eprintln!("Try running with sudo or use manual installation:");
                    eprintln!("  roc completion zsh > completion_file");
                }
            }
        }
        None => {
            eprintln!("❌ Could not determine installation location for zsh completions");
            eprintln!("Use manual installation:");
            eprintln!("  roc completion zsh > completion_file");
        }
    }
}

fn find_install_path(locations: Vec<Option<PathBuf>>) -> Option<PathBuf> {
    for loc in locations {
        if let Some(path) = loc {
            if let Some(parent) = path.parent() {
                if parent.exists() || fs::create_dir_all(parent).is_ok() {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::SCRIPT;

    #[test]
    fn zsh_script_uses_correct_word_indexing() {
        assert!(SCRIPT.contains("case \"$words[2]\""));
        assert!(!SCRIPT.contains("case $words[1]"));
    }

    #[test]
    fn zsh_script_completes_kind_subcommands() {
        assert!(SCRIPT.contains("roc _complete topic '' '' 1"));
        assert!(SCRIPT.contains("roc _complete service '' '' 1"));
    }
}
