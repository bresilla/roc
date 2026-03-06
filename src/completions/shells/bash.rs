use std::path::PathBuf;
use std::{env, fs};

/// Bash completion script with dynamic completions delegated to `roc _complete`.
const SCRIPT: &str = r#"
_roc_completion() {
    local cur prev words cword
    _init_completion || return

    local top="${words[1]}"

    case "$top" in
        "")
            COMPREPLY=($(compgen -W "action topic service param node interface frame run launch work bag daemon middleware completion" -- "$cur"))
            ;;
        launch)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete launch '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete launch '' '' 2 "${words[2]}" 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        run)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete run '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete run '' '' 2 "${words[2]}" 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        topic)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete topic '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete topic "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "pub" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete topic pub '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        service)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete service '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete service "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "call" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete service call '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        param)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete param '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete param "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    case "${words[2]}" in
                        get|set|describe|remove)
                            COMPREPLY=($(compgen -W "$(roc _complete param "${words[2]}" '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                            ;;
                    esac
                    ;;
            esac
            ;;
        node)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete node '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete node "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        action)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete action '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete action "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "goal" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete action goal '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        interface)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete interface '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete interface "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        bag)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete bag '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete bag "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        work)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete work '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete work "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        frame)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete frame '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete frame "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "echo" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete frame echo '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        daemon)
            if [[ "$cword" == 2 ]]; then
                COMPREPLY=($(compgen -W "$(roc _complete daemon '' '' 1 2>/dev/null)" -- "$cur"))
            fi
            ;;
        middleware)
            if [[ "$cword" == 2 ]]; then
                COMPREPLY=($(compgen -W "$(roc _complete middleware '' '' 1 2>/dev/null)" -- "$cur"))
            fi
            ;;
        completion)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur")) ;;
                *) COMPREPLY=($(compgen -W "--install" -- "$cur")) ;;
            esac
            ;;
    esac
}

complete -F _roc_completion roc
"#;

pub fn print_completions() {
    println!("{}", SCRIPT);
}

pub fn install_completion() {
    let install_path = find_install_path(vec![
        Some(PathBuf::from("/usr/share/bash-completion/completions/roc")),
        env::home_dir().map(|h| h.join(".bash_completion.d/roc")),
        env::home_dir().map(|h| h.join(".local/share/bash-completion/completions/roc")),
    ]);
    match install_path {
        Some(path) => {
            println!("Installing bash completions to: {}", path.display());
            match fs::write(&path, SCRIPT) {
                Ok(_) => {
                    println!("✅ Completions installed successfully!");
                    println!("To enable completions, add this to your ~/.bashrc:");
                    println!("  source {}", path.display());
                }
                Err(e) => {
                    eprintln!("❌ Failed to install completions: {}", e);
                    eprintln!("Try running with sudo or use manual installation:");
                    eprintln!("  roc completion bash > completion_file");
                }
            }
        }
        None => {
            eprintln!("❌ Could not determine installation location for bash completions");
            eprintln!("Use manual installation:");
            eprintln!("  roc completion bash > completion_file");
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
    fn bash_script_uses_kind_not_type() {
        assert!(SCRIPT.contains("roc _complete topic"));
        assert!(!SCRIPT.contains("topic \"${words[2]}\" \"\" 1 $current_args"));
    }

    #[test]
    fn bash_script_completes_service_call_type_position() {
        assert!(SCRIPT.contains("roc _complete service call '' 2"));
    }
}
