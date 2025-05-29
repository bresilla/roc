use clap_complete::{generate, Generator, Shell};
use std::io;
use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use std::collections::HashSet;
use std::env;

/// Generate shell completions for the given shell
pub fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

/// Generate advanced shell completion with ROS workspace awareness
pub fn print_advanced_completions(shell: Shell) {
    match shell {
        Shell::Bash => print_bash_completions(),
        Shell::Zsh => print_zsh_completions(), 
        Shell::Fish => print_fish_completions(),
        _ => {
            // Fallback to basic completion
            let mut cmd = crate::arguments::cli(false);
            print_completions(shell, &mut cmd);
        }
    }
}

fn print_bash_completions() {
    println!(r#"
_roc_completion() {{
    local cur prev words cword
    _init_completion || return

    case "${{words[1]}}" in
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
}}

complete -F _roc_completion roc
"#);
}

fn print_zsh_completions() {
    println!(r#"
#compdef roc

_roc() {{
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
                            # Complete launch files
                            local launch_files=($(roc _complete launch "" 2 2>/dev/null))
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
                            # Complete executables
                            local executables=($(roc _complete run "" 2 2>/dev/null))
                            _describe 'executables' executables
                            ;; 
                    esac
                    ;;
            esac
            ;;
    esac
}}

# ensure _roc is registered
compdef _roc roc
"#);
    // ensure the `_roc` function is registered for the `roc` command
    println!("compdef _roc roc");
}

fn print_fish_completions() {
    println!(r#"
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
"#);
}

/// Get completion scripts as strings for file installation
pub fn get_bash_completion_script() -> String {
    r#"
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
"#.to_string()
}

pub fn get_zsh_completion_script() -> String {
    r#"
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
                            # Complete launch files
                            local launch_files=($(roc _complete launch "" 2 2>/dev/null))
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
                            # Complete executables
                            local executables=($(roc _complete run "" 2 2>/dev/null))
                            _describe 'executables' executables
                            ;; 
                    esac
                    ;;
            esac
            ;;
    esac
}

"#.to_string()
}

pub fn get_fish_completion_script() -> String {
    r#"
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
"#.to_string()
}

/// Scan ROS workspaces for launch files
pub fn find_launch_files() -> Vec<String> {
    let mut launch_files = HashSet::new();
    
    // Scan common ROS workspace locations
    let workspace_paths = get_ros_workspace_paths();
    
    for workspace_path in workspace_paths {
        if workspace_path.exists() {
            // Look for launch files in src/*/launch directories
            for entry in WalkDir::new(&workspace_path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    
                    // Check if it's a launch file
                    if let Some(extension) = path.extension() {
                        if extension == "py" || extension == "launch" || extension == "xml" {
                            // Check if it's in a launch directory or contains launch keywords
                            let path_str = path.to_string_lossy();
                            if path_str.contains("/launch/") || 
                               path_str.contains("launch.py") ||
                               path_str.contains("launch.xml") {
                                if let Some(parent) = path.parent() {
                                    if let Some(package_name) = find_package_name(&parent.to_path_buf()) {
                                        if let Some(file_stem) = path.file_stem() {
                                            launch_files.insert(format!("{}:{}", 
                                                package_name, 
                                                file_stem.to_string_lossy()
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    launch_files.into_iter().collect()
}

/// Scan ROS workspaces for executable files
pub fn find_executables() -> Vec<String> {
    let mut executables = HashSet::new();
    
    // Scan common ROS installation and workspace locations
    let workspace_paths = get_ros_workspace_paths();
    
    for workspace_path in workspace_paths {
        if workspace_path.exists() {
            // Look for executables in install/*/lib/* directories
            let install_path = workspace_path.join("install");
            if install_path.exists() {
                for entry in WalkDir::new(&install_path)
                    .follow_links(true)
                    .max_depth(4)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        
                        // Check if it's in a lib directory (where executables are typically stored)
                        if path.to_string_lossy().contains("/lib/") {
                            if let Some(parent) = path.parent() {
                                if let Some(package_name) = parent.file_name() {
                                    if let Some(file_name) = path.file_name() {
                                        // Check if file is executable
                                        if is_executable(path) {
                                            executables.insert(format!("{}:{}", 
                                                package_name.to_string_lossy(),
                                                file_name.to_string_lossy()
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Also look in devel spaces (for catkin workspaces)
            let devel_path = workspace_path.join("devel/lib");
            if devel_path.exists() {
                for entry in WalkDir::new(&devel_path)
                    .follow_links(true)
                    .max_depth(3)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if let Some(parent) = path.parent() {
                            if let Some(package_name) = parent.file_name() {
                                if let Some(file_name) = path.file_name() {
                                    if is_executable(path) {
                                        executables.insert(format!("{}:{}", 
                                            package_name.to_string_lossy(),
                                            file_name.to_string_lossy()
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    executables.into_iter().collect()
}

/// Get common ROS workspace paths to search
fn get_ros_workspace_paths() -> Vec<PathBuf> {
    let mut paths = vec![];
    
    // Add ROS system installation based on ROS_DISTRO environment variable
    if let Ok(ros_distro) = env::var("ROS_DISTRO") {
        paths.push(PathBuf::from(format!("/opt/ros/{}", ros_distro)));
    }
    
    // Add current working directory and parent directories (common workspace locations)
    if let Ok(current_dir) = env::current_dir() {
        paths.push(current_dir.clone());
        
        // Check parent directories for common workspace patterns
        let mut parent = current_dir.clone();
        for _ in 0..5 { // Search up to 5 levels up
            if let Some(p) = parent.parent() {
                parent = p.to_path_buf();
                
                // Look for workspace indicators
                if parent.join("src").exists() || 
                   parent.join("install").exists() ||
                   parent.join("devel").exists() {
                    paths.push(parent.clone());
                }
            } else {
                break;
            }
        }
    }
    
    // Add paths from ROS environment variables
    if let Ok(colcon_prefix_path) = env::var("COLCON_PREFIX_PATH") {
        for path in colcon_prefix_path.split(':') {
            let path_buf = PathBuf::from(path);
            if let Some(parent) = path_buf.parent() {
                paths.push(parent.to_path_buf());
            }
        }
    }
    
    if let Ok(ament_prefix_path) = env::var("AMENT_PREFIX_PATH") {
        for path in ament_prefix_path.split(':') {
            let path_buf = PathBuf::from(path);
            if let Some(parent) = path_buf.parent() {
                paths.push(parent.to_path_buf());
            }
        }
    }
    
    // Remove duplicates and return
    paths.sort();
    paths.dedup();
    paths
}

/// Find package name by looking for package.xml in directory or parents
fn find_package_name(dir: &PathBuf) -> Option<String> {
    let mut current = dir.clone();
    
    for _ in 0..10 { // Limit search depth
        let package_xml = current.join("package.xml");
        if package_xml.exists() {
            if let Ok(content) = fs::read_to_string(&package_xml) {
                // Simple XML parsing to extract package name
                if let Some(start) = content.find("<name>") {
                    if let Some(end) = content[start + 6..].find("</name>") {
                        let name = &content[start + 6..start + 6 + end];
                        return Some(name.trim().to_string());
                    }
                }
            }
        }
        
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    None
}

/// Check if a file is executable
fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }
    
    #[cfg(not(unix))]
    {
        // On Windows, check file extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            return ext == "exe" || ext == "bat" || ext == "cmd";
        }
    }
    
    false
}

/// Get available packages in the workspace
pub fn find_packages() -> Vec<String> {
    let mut packages = HashSet::new();
    let workspace_paths = get_ros_workspace_paths();
    
    for workspace_path in workspace_paths {
        // Look for package.xml files
        for entry in WalkDir::new(&workspace_path)
            .follow_links(true)
            .max_depth(6)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.file_name() == "package.xml" {
                if let Some(package_name) = find_package_name(&entry.path().parent().unwrap().to_path_buf()) {
                    packages.insert(package_name);
                }
            }
        }
    }
    
    packages.into_iter().collect()
}

/// Get available shells for completion generation
pub fn available_shells() -> Vec<Shell> {
    vec![Shell::Bash, Shell::Zsh, Shell::Fish]
}
