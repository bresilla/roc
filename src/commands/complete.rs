use clap::ArgMatches;
use crate::completion;

pub fn handle(matches: ArgMatches) {
    let command = matches.get_one::<String>("command").unwrap();
    let subcommand = matches.get_one::<String>("subcommand");
    let position = matches.get_one::<String>("position")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(0);
    
    match (command.as_str(), subcommand.map(|s| s.as_str())) {
        ("launch", None) if position == 1 => {
            // Complete package names for launch command
            let packages = completion::find_packages();
            for package in packages {
                println!("{}", package);
            }
        },
        ("launch", None) if position == 2 => {
            // Complete launch files - this would need the package name from previous arg
            let launch_files = completion::find_launch_files();
            for launch_file in launch_files {
                if let Some((_, file_name)) = launch_file.split_once(':') {
                    println!("{}", file_name);
                }
            }
        },
        ("run", None) if position == 1 => {
            // Complete package names for run command
            let packages = completion::find_packages();
            for package in packages {
                println!("{}", package);
            }
        },
        ("run", None) if position == 2 => {
            // Complete executable names - this would need the package name from previous arg
            let executables = completion::find_executables();
            for executable in executables {
                if let Some((_, exec_name)) = executable.split_once(':') {
                    println!("{}", exec_name);
                }
            }
        },
        _ => {
            // Default case - no completions
        }
    }
}
