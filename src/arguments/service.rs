use clap::{Arg, ArgAction, ArgMatches, Command};

/// Common service arguments that are extracted from the parent service command
#[derive(Debug, Clone)]
pub struct CommonServiceArgs {
    pub spin_time: Option<String>,
    pub use_sim_time: bool,
    pub no_daemon: bool,
}

impl CommonServiceArgs {
    /// Extract common service arguments from the parent service command matches
    pub fn from_matches(parent_matches: &ArgMatches) -> Self {
        Self {
            spin_time: parent_matches.get_one::<String>("spin_time").cloned(),
            use_sim_time: parent_matches.get_flag("use_sim_time"),
            no_daemon: parent_matches.get_flag("no_daemon"),
        }
    }
}

pub fn cmd() -> Command {
    Command::new("service")
        .about("Various service subcommands")
        .aliases(&["s", "ser"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        // Common flags that ONLY exist at the top level
        .arg(
            Arg::new("spin_time")
            .long("spin-time")
            .aliases(&["spin_time", "spin"])
            .value_name("SPIN_TIME")
            .num_args(1)
            .help("Spin time for discovery (if daemon not in use)")
            .action(ArgAction::Append)
        )
        .arg(
            Arg::new("use_sim_time")
            .short('s')
            .long("use-sim-time")
            .aliases(&["use_sim_time", "use_simtime", "sim"])
            .help("Enable ROS simulation time")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no_daemon")
            .long("no-daemon")
            .aliases(&["no_daemon"])
            .help("Don't spawn or use a running daemon")
            .action(ArgAction::SetTrue)
        )
        .subcommand(
            Command::new("call")
            .about("Call a service by delegating to `ros2 service call`")
            .aliases(["c", "invoke"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("service_name")
                .help("Name of the ROS service to call to (e.g. '/add_two_ints')")
                .required(true)
                .value_name("SERVICE_NAME")
            )
            .arg(
                Arg::new("service_type")
                .help("Type of the ROS service (e.g. 'std_srvs/srv/Empty')")
                .required(true)
                .value_name("SERVICE_TYPE")
            )
            .arg(
                Arg::new("values")
                .help("Values to fill the service request with in YAML format")
                .value_name("VALUES")
                .action(ArgAction::Append)
            )
            .arg(
                Arg::new("rate")
                .short('r')
                .long("rate")
                .help("Repeat the call at a specific rate in Hz")
                .value_name("RATE")
                .num_args(1)
                .action(ArgAction::Append)
            )
        )
        .subcommand(
            Command::new("find")
            .about("Output a list of available services of a given type")
            .aliases(["f", "lookup", "search"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("service_type")
                .help("Name of the ROS service type to filter for (e.g. 'std_srvs/srv/Empty')")
                .required(true)
            )
            .arg(
                Arg::new("count_services")
                .short('c')
                .long("count-services")
                .aliases(&["count_services", "count"])
                .help("Only display the number of services discovered")
                .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("include_hidden_services")
                .long("include-hidden-services")
                .short('a')
                .aliases(&["include_hidden_services", "all"])
                .help("Consider hidden services as well")
                .action(ArgAction::SetTrue)
            )
        )
        .subcommand(
            Command::new("list")
            .about("Output a list of available services")
            .aliases(["l", "ls"])
            // Allow calling without args (default behavior is to list services).
            .arg(
                Arg::new("show_types")
                .short('t')
                .long("show-types")
                .aliases(&["show_types", "types"])
                .help("Additionally show the service type")
                .action(ArgAction::SetTrue)
                .conflicts_with("count_services")
            )
            .arg(
                Arg::new("count_services")
                .short('c')
                .long("count-services")
                .aliases(&["count_services", "count"])
                .help("Only display the number of services discovered")
                .action(ArgAction::SetTrue)
                .conflicts_with("show_types")
            )
            .arg(
                Arg::new("include_hidden_services")
                .long("include-hidden-services")
                .short('a')
                .aliases(&["include_hidden_services", "all"])
                .visible_aliases(&["all"])
                .help("Consider hidden services as well")
                .action(ArgAction::SetTrue)
            )
        )
        .subcommand(
            Command::new("kind")
            .about("Print a service's type/kind")
            .aliases(["k", "type"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("service_name")
                .help("Name of the ROS service to get type (e.g. '/add_two_ints')")
                .required(true)
                .value_name("SERVICE_NAME")
            )
        )
}

#[cfg(test)]
mod tests {
    use super::cmd;

    #[test]
    fn service_call_help_marks_command_as_delegated() {
        let mut command = cmd();
        let call = command
            .find_subcommand_mut("call")
            .expect("service call subcommand should exist");
        let mut buffer = Vec::new();
        call.write_long_help(&mut buffer).unwrap();
        let help = String::from_utf8(buffer).unwrap();

        assert!(help.contains("delegating to `ros2 service call`"));
    }
}
