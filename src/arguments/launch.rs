use clap::{arg, Command};

pub fn cmd() -> Command {
    Command::new("launch")
        .about("Launch a launch file")
        .aliases(&["l"])
        .arg_required_else_help(true)
        .arg(
            arg!(<package_name> "Name of the ROS package which contains the launch file")
            .required(true)
            .value_parser(package_value_parser)
        )
        .arg(
            arg!(<launch_file_name> "Name of the launch file")
            .required(true)
            .value_parser(launch_file_value_parser)
        )
        .arg(
            arg!([launch_arguments] "Arguments to the launch file; '<n>:=<value>' (for duplicates, last one wins)")
        )
        .arg(arg!(-n --noninteractive "Run the launch system non-interactively, with no terminal associated"))
        .arg(arg!(-d --debug "Put the launch system in debug mode, provides more verbose output."))
        .arg(arg!(-p --print "Print the launch description to the console without launching it."))
        .arg(arg!(-s --show_args "Show arguments that may be given to the launch file."))
        .arg(arg!(-a --show_all "Show all launched subprocesses' output"))
        .arg(arg!(--launch_prefix <LAUNCH_PREFIX> "Prefix command before executables (e.g. --launch-prefix 'xterm -e gdb -ex run --args')."))
        .arg(arg!(--launch_prefix_filter <LAUNCH_PREFIX_FILTER> "Regex pattern for executable filtering with --launch-prefix."))
}

fn package_value_parser(s: &str) -> Result<String, String> {
    // This is where we could add validation for package names
    // For now, just return the string as-is
    Ok(s.to_string())
}

fn launch_file_value_parser(s: &str) -> Result<String, String> {
    // This is where we could add validation for launch file names
    // For now, just return the string as-is
    Ok(s.to_string())
}