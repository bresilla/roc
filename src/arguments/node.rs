use crate::ui::output;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Common node arguments that are extracted from the parent node command
#[derive(Debug, Clone)]
pub struct CommonNodeArgs {
    pub spin_time: Option<String>,
    pub use_sim_time: bool,
    pub no_daemon: bool,
}

impl CommonNodeArgs {
    /// Extract common node arguments from the parent node command matches
    pub fn from_matches(parent_matches: &ArgMatches) -> Self {
        Self {
            spin_time: parent_matches.get_one::<String>("spin_time").cloned(),
            use_sim_time: parent_matches.get_flag("use_sim_time"),
            no_daemon: parent_matches.get_flag("no_daemon"),
        }
    }
}

pub fn cmd() -> Command {
    Command::new("node")
        .about("Various node subcommands")
        .aliases(&["n", "nod"])
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
                .about("Print information about a node")
                .aliases(["i", "show"])
                .arg_required_else_help(true)
                .arg(
                    Arg::new("node_name")
                        .help("Name of the ROS node to get info (e.g. '/talker')")
                        .required(true)
                        .value_name("NODE_NAME"),
                )
                .arg(
                    Arg::new("include_hidden_nodes")
                        .long("include-hidden-nodes")
                        .short('a')
                        .aliases(&["include_hidden_nodes", "all"])
                        .visible_aliases(&["all"])
                        .help("Consider hidden nodes as well")
                        .action(ArgAction::SetTrue),
                )
                .arg(output::arg()),
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
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("count_nodes")
                        .short('c')
                        .long("count-nodes")
                        .aliases(&["count_nodes", "count"])
                        .help("Only display the number of nodes discovered")
                        .action(ArgAction::SetTrue),
                )
                .arg(output::arg()),
        )
}
