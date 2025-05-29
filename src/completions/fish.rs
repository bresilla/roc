use std::{fs, env};
use std::path::PathBuf;

/// Original fish completion script
const SCRIPT: &str = r#"
# Fish completions for roc

# Main commands
complete -c roc -f -n "__fish_use_subcommand" -a "action" -d "Various action subcommands"
complete -c roc -f -n "__fish_use_subcommand" -a "topic" -d "Various topic subcommands"
complete -c roc -f -n "__fish_use_subcommand" -a "service" -d "Various service subcommands"
complete -c roc -f -n "__fish_use_subcommand" -a "param" -d "Various param subcommands"
complete -c roc -f -n "__fish_use_subcommand" -a "node" -d "Various node subcommands"
complete -c roc -f -n "__fish_use_subcommand" -a "interface" -d "Various interface subcommands"
complete -c roc -f -n "__fish_use_subcommand" -a "frame" -d "Various transform subcommands"
complete -c roc -f -n "__fish_use_subcommand" -a "run" -d "Run an executable"
complete -c roc -f -n "__fish_use_subcommand" -a "launch" -d "Launch a launch file"
complete -c roc -f -n "__fish_use_subcommand" -a "work" -d "Packages and workspace"
complete -c roc -f -n "__fish_use_subcommand" -a "bag" -d "ROS bag tools"
complete -c roc -f -n "__fish_use_subcommand" -a "daemon" -d "Daemon and bridge"
complete -c roc -f -n "__fish_use_subcommand" -a "middleware" -d "Middleware settings"
complete -c roc -f -n "__fish_use_subcommand" -a "completion" -d "Generate shell completions"

# Launch command completions
complete -c roc -f -n "__fish_seen_subcommand_from launch; and not __fish_seen_subcommand_from (roc _complete launch '' 1 2>/dev/null)" -a "(roc _complete launch '' 1 2>/dev/null)" -d "Package"
complete -c roc -f -n "__fish_seen_subcommand_from launch; and __fish_seen_subcommand_from (roc _complete launch '' 1 2>/dev/null)" -a "(roc _complete launch '' 2 2>/dev/null)" -d "Launch file"

# Run command completions  
complete -c roc -f -n "__fish_seen_subcommand_from run; and not __fish_seen_subcommand_from (roc _complete run '' 1 2>/dev/null)" -a "(roc _complete run '' 1 2>/dev/null)" -d "Package"
complete -c roc -f -n "__fish_seen_subcommand_from run; and __fish_seen_subcommand_from (roc _complete run '' 1 2>/dev/null)" -a "(roc _complete run '' 2 2>/dev/null)" -d "Executable"
"#;

/// Print fish completions to stdout
pub fn print_completions() {
    println!("{}", SCRIPT);
}

/// Install fish completions to a default location
pub fn install_completion() {
    let install_path = find_install_path(vec![
        env::home_dir().map(|h| h.join(".config/fish/completions/roc.fish")),
        Some(PathBuf::from("/usr/share/fish/completions/roc.fish")),
    ]);
    match install_path {
        Some(path) => {
            println!("Installing fish completions to: {}", path.display());
            let script = SCRIPT;
            match fs::write(&path, script) {
                Ok(_) => {
                    println!("✅ Completions installed successfully!");
                    println!("Completions should be automatically available in new fish sessions.");
                }
                Err(e) => {
                    eprintln!("❌ Failed to install completions: {}", e);
                    eprintln!("Try running with sudo or use manual installation:");
                    eprintln!("  roc completion fish > completion_file");
                }
            }
        }
        None => {
            eprintln!("❌ Could not determine installation location for fish completions");
            eprintln!("Use manual installation:");
            eprintln!("  roc completion fish > completion_file");
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
