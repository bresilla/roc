use std::{fs, env};
use std::path::PathBuf;

/// Enhanced zsh completion script with support for all subcommands
const SCRIPT: &str = r#"
#compdef roc

_roc() {
    local context state line
    typeset -A opt_args
    
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
                            local packages=($(roc _complete launch "" "" 1 2>/dev/null))
                            _describe 'packages' packages
                            ;; 
                        3)
                            # Complete launch files for specific package
                            local launch_files=($(roc _complete launch "" "" 2 "${words[2]}" 2>/dev/null))
                            _describe 'launch files' launch_files
                            ;; 
                    esac
                    ;;
                run)
                    case $CURRENT in
                        2)
                            # Complete package names
                            local packages=($(roc _complete run "" "" 1 2>/dev/null))
                            _describe 'packages' packages
                            ;; 
                        3)
                            # Complete executables for specific package
                            local executables=($(roc _complete run "" "" 2 "${words[2]}" 2>/dev/null))
                            _describe 'executables' executables
                            ;; 
                    esac
                    ;;
                topic)
                    case $CURRENT in
                        2)
                            # Complete topic subcommands
                            local subcommands=($(roc _complete topic "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            # Complete based on subcommand
                            case $words[2] in
                                echo|info|bw|delay|hz|type|pub)
                                    local topics=($(roc _complete topic "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'topics' topics
                                    ;;
                                find)
                                    local msg_types=($(roc _complete topic "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'message types' msg_types
                                    ;;
                            esac
                            ;;
                        4)
                            case $words[2] in
                                pub)
                                    local msg_types=($(roc _complete topic "${words[2]}" "" 2 2>/dev/null))
                                    _describe 'message types' msg_types
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                service)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete service "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                call|type)
                                    local services=($(roc _complete service "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'services' services
                                    ;;
                                find)
                                    local service_types=($(roc _complete service "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'service types' service_types
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                param)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete param "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                get|set|describe|remove)
                                    local params=($(roc _complete param "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'parameters' params
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                node)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete node "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                info)
                                    local nodes=($(roc _complete node "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'nodes' nodes
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                action)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete action "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                info|goal)
                                    local actions=($(roc _complete action "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'actions' actions
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                interface)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete interface "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                show)
                                    local interfaces=($(roc _complete interface "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'interfaces' interfaces
                                    ;;
                                package)
                                    local packages=($(roc _complete interface "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'packages' packages
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                bag)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete bag "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                play|info)
                                    local bags=($(roc _complete bag "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'bag files' bags
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                work)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete work "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                build)
                                    local packages=($(roc _complete work "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'packages' packages
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                frame)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete frame "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                        3)
                            case $words[2] in
                                echo|info)
                                    local frames=($(roc _complete frame "${words[2]}" "" 1 2>/dev/null))
                                    _describe 'frames' frames
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                daemon)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete daemon "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                    esac
                    ;;
                middleware)
                    case $CURRENT in
                        2)
                            local subcommands=($(roc _complete middleware "" "" 1 2>/dev/null))
                            _describe 'subcommands' subcommands
                            ;;
                    esac
                    ;;
                completion)
                    case $CURRENT in
                        2)
                            local shells=('bash' 'zsh' 'fish')
                            _describe 'shells' shells
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
