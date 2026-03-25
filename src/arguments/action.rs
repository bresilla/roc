use clap::{Arg, ArgAction, ArgMatches, Command};

/// Common action arguments that are extracted from the parent action command
#[derive(Debug, Clone)]
pub struct CommonActionArgs {
    pub spin_time: Option<String>,
    pub use_sim_time: bool,
    pub no_daemon: bool,
}

impl CommonActionArgs {
    /// Extract common action arguments from the parent action command matches
    pub fn from_matches(parent_matches: &ArgMatches) -> Self {
        Self {
            spin_time: parent_matches.get_one::<String>("spin_time").cloned(),
            use_sim_time: parent_matches.get_flag("use_sim_time"),
            no_daemon: parent_matches.get_flag("no_daemon"),
        }
    }
}

pub fn cmd() -> Command {
    Command::new("action")
        .about("Various action subcommands")
        .aliases(&["a", "act"])
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
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("use_sim_time")
                .short('s')
                .long("use-sim-time")
                .aliases(&["use_sim_time", "use_simtime", "sim"])
                .help("Enable ROS simulation time")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_daemon")
                .long("no-daemon")
                .aliases(&["no_daemon"])
                .help("Don't spawn or use a running daemon")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("info")
                .about(" Print information about an action")
                .aliases(["i", "show"])
                .arg_required_else_help(true)
                .arg(
                    Arg::new("action_name")
                        .help("Name of the ROS action to get info (e.g. '/fibonacci')")
                        .required(true)
                        .value_name("ACTION_NAME"),
                )
                .arg(
                    Arg::new("show_types")
                        .short('t')
                        .long("show-types")
                        .aliases(&["show_types", "types"])
                        .help("Additionally show the action type")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("count_actions"),
                )
                .arg(
                    Arg::new("count_actions")
                        .short('c')
                        .long("count-actions")
                        .aliases(&["count_actions", "count"])
                        .help("Only display the number of actions discovered")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("show_types"),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List all actions")
                .aliases(["l", "ls"])
                .arg(
                    Arg::new("show_types")
                        .short('t')
                        .long("show-types")
                        .aliases(&["show_types", "types"])
                        .help("Additionally show the action type")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("count_actions"),
                )
                .arg(
                    Arg::new("count_actions")
                        .short('c')
                        .long("count-actions")
                        .aliases(&["count_actions", "count"])
                        .help("Only display the number of actions discovered")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("show_types"),
                ),
        )
        .subcommand(
            Command::new("goal")
                .about("Send a goal to an action server by delegating to `ros2 action send_goal`")
                .aliases(["g", "send_goal"])
                .arg_required_else_help(true)
                .arg(
                    Arg::new("action_name")
                        .help("Name of the ROS action to get info (e.g. '/fibonacci')")
                        .required(true)
                        .value_name("ACTION_NAME"),
                )
                .arg(
                    Arg::new("action_type")
                        .help("Type of the ROS action (e.g. 'example_interfaces/action/Fibonacci')")
                        .required(true)
                        .value_name("ACTION_TYPE"),
                )
                .arg(
                    Arg::new("goal")
                        .help("Goal to send to the action server (e.g. '{order: 10}')")
                        .required(true)
                        .value_name("GOAL")
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("feedback")
                        .short('f')
                        .long("feedback")
                        .help("Echo feedback messages for the goal")
                        .action(ArgAction::SetTrue),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::cmd;

    #[test]
    fn action_goal_help_marks_command_as_delegated() {
        let mut command = cmd();
        let goal = command
            .find_subcommand_mut("goal")
            .expect("action goal subcommand should exist");
        let mut buffer = Vec::new();
        goal.write_long_help(&mut buffer).unwrap();
        let help = String::from_utf8(buffer).unwrap();

        assert!(help.contains("delegating to `ros2 action send_goal`"));
    }
}
