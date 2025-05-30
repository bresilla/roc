use std::{fs, env};
use std::path::PathBuf;

/// Original zsh completion script
const SCRIPT: &str = r#"
#compdef roc

_roc() {
    local context state line
    
    _arguments -C \
        '1:command:->commands' \
        '*::arg:->args'
    
    case $state in
        commands)
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
            ;;
        args)
            case $words[1] in
               launch)
                    case $CURRENT in
                        2)
                            # Complete package names
                            local packages=($(roc _complete launch "" 1 2>/dev/null))
                            _describe 'packages' packages
                            ;; 
                        3)
                            # Complete launch files for specific package
                            local launch_files=($(roc _complete launch "" 2 "${words[2]}" 2>/dev/null))
                            _describe 'launch files' launch_files
                            ;; 
                    esac
                    ;;
               run)
                    case $CURRENT in
                        2)
                            # Complete package names
                            local packages=($(roc _complete run "" 1 2>/dev/null))
                            _describe 'packages' packages
                            ;; 
                        3)
                            # Complete executables for specific package
                            local executables=($(roc _complete run "" 2 "${words[2]}" 2>/dev/null))
                            _describe 'executables' executables
                            ;; 
                    esac
                    ;;
            esac
            ;;
    esac
}

# ensure _roc is registered
compdef _roc roc
"#;

/// Print zsh completions to stdout
pub fn print_completions() {
    println!("{}", SCRIPT);
}

/// Install zsh completions to a default location
pub fn install_completion() {
    let install_path = find_install_path(vec![
        env::home_dir().map(|h| h.join(".zfunc/_roc")),
        Some(PathBuf::from("/usr/local/share/zsh/site-functions/_roc")),
        env::home_dir().map(|h| h.join(".local/share/zsh/site-functions/_roc")),
    ]);
    match install_path {
        Some(path) => {
            println!("Installing zsh completions to: {}", path.display());
            let script = SCRIPT;
            match fs::write(&path, script) {
                Ok(_) => {
                    println!("✅ Completions installed successfully!");
                    println!("To enable completions, add this to your ~/.zshrc:");
                    if path.parent().unwrap().to_string_lossy().contains(".zfunc") {
                        println!("  fpath=(~/.zfunc $fpath)");
                        println!("  autoload -U compinit && compinit");
                    } else {
                        println!("  autoload -U compinit && compinit");
                    }
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
