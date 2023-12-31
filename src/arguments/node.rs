use clap::{Command, Arg, ArgAction};

pub fn cmd() -> Command {
    Command::new("node")
        .about("Various node subcommands")
        .aliases(&["n", "nod"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("info")
            .about("Print information about a node")
            .aliases(["i", "show"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get info (e.g. '/talker')")
                .required(true)
                .value_name("NODE_NAME")
            )
            .arg(
                Arg::new("include_hidden_nodes")
                .long("include-hidden-nodes")
                .short('a')
                .aliases(&["include_hidden_nodes", "all"])
                .visible_aliases(&["all"])
                .help("Consider hidden nodes as well")
                .action(ArgAction::SetTrue)
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
            .arg(
                Arg::new("spin_time")
                .long("spin-time")
                .aliases(&["spin_time", "spin"])
                .value_name("SPIN_TIME")
                .num_args(1)
                .help("Spin time for discovery (if daemon not in use)")
                .action(ArgAction::Append)
            )
        )
        .subcommand(
            Command::new("list")
            .about("List all nodes")
            .aliases(["l", "ls"])
            .arg(
                Arg::new("include_hidden_nodes")
                .long("include-hidden-nodes")
                .short('a')
                .aliases(&["include_hidden_nodes", "all"])
                .visible_aliases(&["all"])
                .help("Display all nodes even hidden ones")
                .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("count_nodes")
                .short('c')
                .long("count-nodes")
                .aliases(&["count_nodes", "count"])
                .help("Only display the number of nodes discovered")
                .action(ArgAction::SetTrue)
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
            .arg(
                Arg::new("spin_time")
                .long("spin-time")
                .aliases(&["spin_time", "spin"])
                .value_name("SPIN_TIME")
                .num_args(1)
                .help("Spin time for discovery (if daemon not in use)")
                .action(ArgAction::Append)
            )
        )
}