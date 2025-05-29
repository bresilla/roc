use std::{fs, env};
use std::path::PathBuf;

/// Original bash completion script
const SCRIPT: &str = r#"
_roc_completion() {
    local cur prev words cword
    _init_completion || return

    case "${words[1]}" in
        launch)
             case "$cword" in
                2)
                    # Complete package names
                    local packages=$(roc _complete launch "" 1 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$packages" -- "$cur"))
                    ;; 
                3)
                    # Complete launch files for the given package
                    local launch_files=$(roc _complete launch "" 2 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$launch_files" -- "$cur"))
                    ;; 
             esac
             ;; 
        run)
             case "$cword" in
                2)
                    # Complete package names
                    local packages=$(roc _complete run "" 1 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$packages" -- "$cur"))
                    ;; 
                3)
                    # Complete executables for the given package
                    local executables=$(roc _complete run "" 2 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$executables" -- "$cur"))
                    ;; 
             esac
             ;; 
         *)
            # Default completion for other commands
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
