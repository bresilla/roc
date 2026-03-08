use crate::ui::output;
use clap::{arg, Command};

pub fn cmd() -> Command {
    Command::new("run")
        .about( "Run an executable")
        .aliases(&["r"])
        .arg_required_else_help(true)
        .arg(
            arg!(<package_name> "Name of the ROS package to run (e.g. 'demo_nodes_cpp')")
            .required(true)
            .value_parser(package_value_parser)
        )
        .arg(
            arg!(<executable_name> "Name of the ROS executable to run (e.g. 'talker')")
            .required(true)
            .value_parser(executable_value_parser)
        )
        .arg(
            arg!([argv] "Pass arbitrary arguments to the executable (e.g. '__log_level:=debug')")
        )
        .arg(arg!(--prefix <PREFIX> "Prefix command, which should go before the executable (e.g. --prefix 'gdb -ex run --args')"))
        .arg(output::arg())
}

fn package_value_parser(s: &str) -> Result<String, String> {
    // This is where we could add validation for package names
    // For now, just return the string as-is
    Ok(s.to_string())
}

fn executable_value_parser(s: &str) -> Result<String, String> {
    // This is where we could add validation for executable names
    // For now, just return the string as-is
    Ok(s.to_string())
}
