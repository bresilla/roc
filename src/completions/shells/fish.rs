use std::path::PathBuf;
use std::{env, fs};

/// Fish completion script with dynamic completions delegated to `roc _complete`.
const SCRIPT: &str = r#"
# Fish completions for roc

function __roc_args
    commandline -opc
end

function __roc_main_cmd
    set -l cmd (__roc_args)
    if test (count $cmd) -ge 2
        echo $cmd[2]
    end
end

function __roc_sub_cmd
    set -l cmd (__roc_args)
    if test (count $cmd) -ge 3
        echo $cmd[3]
    end
end

complete -c roc -f -n "not __fish_seen_subcommand_from action topic service param node interface frame run launch work bag daemon middleware completion" -a "action topic service param node interface frame run launch work bag daemon middleware completion"

complete -c roc -f -n "__fish_seen_subcommand_from launch; and not __roc_sub_cmd" -a "(roc _complete launch '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from launch; and __roc_sub_cmd" -a "(roc _complete launch '' '' 2 (__roc_sub_cmd) 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from run; and not __roc_sub_cmd" -a "(roc _complete run '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from run; and __roc_sub_cmd" -a "(roc _complete run '' '' 2 (__roc_sub_cmd) 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from topic; and not __roc_sub_cmd" -a "(roc _complete topic '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from topic; and __roc_sub_cmd" -a "(roc _complete topic (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub; and test (count (__roc_args)) -ge 4" -a "(roc _complete topic pub '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from service; and not __roc_sub_cmd" -a "(roc _complete service '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from service; and __roc_sub_cmd" -a "(roc _complete service (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = call; and test (count (__roc_args)) -ge 4" -a "(roc _complete service call '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from param; and not __roc_sub_cmd" -a "(roc _complete param '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from param; and __roc_sub_cmd" -a "(roc _complete param (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from param; and contains (__roc_sub_cmd) get set describe remove; and test (count (__roc_args)) -ge 4" -a "(roc _complete param (__roc_sub_cmd) '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from node; and not __roc_sub_cmd" -a "(roc _complete node '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from node; and __roc_sub_cmd" -a "(roc _complete node (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from action; and not __roc_sub_cmd" -a "(roc _complete action '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from action; and __roc_sub_cmd" -a "(roc _complete action (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from action; and test (__roc_sub_cmd) = goal; and test (count (__roc_args)) -ge 4" -a "(roc _complete action goal '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from interface; and not __roc_sub_cmd" -a "(roc _complete interface '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from interface; and __roc_sub_cmd" -a "(roc _complete interface (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from bag; and not __roc_sub_cmd" -a "(roc _complete bag '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from bag; and __roc_sub_cmd" -a "(roc _complete bag (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from work; and not __roc_sub_cmd" -a "(roc _complete work '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from work; and __roc_sub_cmd" -a "(roc _complete work (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from frame; and not __roc_sub_cmd" -a "(roc _complete frame '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from frame; and __roc_sub_cmd" -a "(roc _complete frame (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from frame; and test (__roc_sub_cmd) = echo; and test (count (__roc_args)) -ge 4" -a "(roc _complete frame echo '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from daemon; and not __roc_sub_cmd" -a "(roc _complete daemon '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from middleware; and not __roc_sub_cmd" -a "(roc _complete middleware '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from completion; and not __roc_sub_cmd" -a "bash zsh fish"
complete -c roc -f -n "__fish_seen_subcommand_from completion" -l install
"#;

pub fn print_completions() {
    println!("{}", SCRIPT);
}

pub fn install_completion() {
    let install_path = find_install_path(vec![
        env::home_dir().map(|h| h.join(".config/fish/completions/roc.fish")),
        Some(PathBuf::from("/usr/share/fish/completions/roc.fish")),
    ]);
    match install_path {
        Some(path) => {
            println!("Installing fish completions to: {}", path.display());
            match fs::write(&path, SCRIPT) {
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

#[cfg(test)]
mod tests {
    use super::SCRIPT;

    #[test]
    fn fish_script_completes_kind_subcommands() {
        assert!(SCRIPT.contains("roc _complete topic '' '' 1"));
        assert!(SCRIPT.contains("roc _complete service '' '' 1"));
    }

    #[test]
    fn fish_script_supports_install_flag() {
        assert!(SCRIPT.contains("-l install"));
    }
}
