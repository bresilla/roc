use std::path::PathBuf;
use std::{env, fs};

/// Comprehensive bash completion script
const SCRIPT: &str = r#"
_roc_completion() {
    local cur prev words cword
    _init_completion || return

    # Helper function to get current arguments for completion
    local current_args=""
    for ((i=2; i<cword; i++)); do
        current_args="$current_args ${words[i]}"
    done

    case "${words[1]}" in
        launch)
            case "$cword" in
                2)
                    # Complete package names for launch
                    local packages=$(roc _complete launch "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$packages" -- "$cur"))
                    ;;
                3)
                    # Complete launch files for the given package
                    local launch_files=$(roc _complete launch "" "" 2 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$launch_files" -- "$cur"))
                    ;;
            esac
            ;;
        run)
            case "$cword" in
                2)
                    # Complete package names for run
                    local packages=$(roc _complete run "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$packages" -- "$cur"))
                    ;;
                3)
                    # Complete executables for the given package
                    local executables=$(roc _complete run "" "" 2 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$executables" -- "$cur"))
                    ;;
            esac
            ;;
        topic)
            case "$cword" in
                2)
                    # Complete topic subcommands
                    local subcommands=$(roc _complete topic "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete topic names or message types based on subcommand
                    local completions=$(roc _complete topic "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$completions" -- "$cur"))
                    ;;
                4)
                    # Complete message types for pub command
                    if [[ "${words[2]}" == "pub" ]]; then
                        local msg_types=$(roc _complete topic "${words[2]}" "" 2 $current_args 2>/dev/null || echo "")
                        COMPREPLY=($(compgen -W "$msg_types" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        service)
            case "$cword" in
                2)
                    # Complete service subcommands
                    local subcommands=$(roc _complete service "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete service names or types
                    local completions=$(roc _complete service "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$completions" -- "$cur"))
                    ;;
            esac
            ;;
        param)
            case "$cword" in
                2)
                    # Complete param subcommands
                    local subcommands=$(roc _complete param "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete parameter names
                    local params=$(roc _complete param "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$params" -- "$cur"))
                    ;;
            esac
            ;;
        node)
            case "$cword" in
                2)
                    # Complete node subcommands
                    local subcommands=$(roc _complete node "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete node names
                    local nodes=$(roc _complete node "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$nodes" -- "$cur"))
                    ;;
            esac
            ;;
        action)
            case "$cword" in
                2)
                    # Complete action subcommands
                    local subcommands=$(roc _complete action "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete action names
                    local actions=$(roc _complete action "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$actions" -- "$cur"))
                    ;;
            esac
            ;;
        interface)
            case "$cword" in
                2)
                    # Complete interface subcommands
                    local subcommands=$(roc _complete interface "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete interface names or packages
                    local completions=$(roc _complete interface "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$completions" -- "$cur"))
                    ;;
            esac
            ;;
        bag)
            case "$cword" in
                2)
                    # Complete bag subcommands
                    local subcommands=$(roc _complete bag "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete bag files or topics
                    local completions=$(roc _complete bag "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$completions" -- "$cur"))
                    ;;
            esac
            ;;
        work)
            case "$cword" in
                2)
                    # Complete work subcommands
                    local subcommands=$(roc _complete work "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete package names for build
                    local completions=$(roc _complete work "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$completions" -- "$cur"))
                    ;;
            esac
            ;;
        frame)
            case "$cword" in
                2)
                    # Complete frame subcommands
                    local subcommands=$(roc _complete frame "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
                3)
                    # Complete frame names
                    local frames=$(roc _complete frame "${words[2]}" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$frames" -- "$cur"))
                    ;;
            esac
            ;;
        daemon)
            case "$cword" in
                2)
                    # Complete daemon subcommands
                    local subcommands=$(roc _complete daemon "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
            esac
            ;;
        middleware)
            case "$cword" in
                2)
                    # Complete middleware subcommands
                    local subcommands=$(roc _complete middleware "" "" 1 $current_args 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$subcommands" -- "$cur"))
                    ;;
            esac
            ;;
        completion)
            case "$cword" in
                2)
                    # Complete shell types
                    COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur"))
                    ;;
            esac
            ;;
        *)
            # Default completion for top-level commands
            local commands="action topic service param node interface frame run launch work bag daemon middleware completion"
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            ;;
    esac
}

complete -F _roc_completion roc
"#;

/// Print bash completions to stdout
pub fn print_completions() {
    println!("{}", SCRIPT);
}

/// Install bash completions to a default location
pub fn install_completion() {
    let install_path = find_install_path(vec![
        Some(PathBuf::from("/usr/share/bash-completion/completions/roc")),
        env::home_dir().map(|h| h.join(".bash_completion.d/roc")),
        env::home_dir().map(|h| h.join(".local/share/bash-completion/completions/roc")),
    ]);
    match install_path {
        Some(path) => {
            println!("Installing bash completions to: {}", path.display());
            let script = SCRIPT;
            match fs::write(&path, script) {
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
