use crate::ui::output;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Common param arguments that are extracted from the parent param command
#[derive(Debug, Clone)]
pub struct CommonParamArgs {
    pub spin_time: Option<String>,
    pub use_sim_time: bool,
    pub no_daemon: bool,
}

impl CommonParamArgs {
    /// Extract common param arguments from the parent param command matches
    pub fn from_matches(parent_matches: &ArgMatches) -> Self {
        Self {
            spin_time: parent_matches.get_one::<String>("spin_time").cloned(),
            use_sim_time: parent_matches.get_flag("use_sim_time"),
            no_daemon: parent_matches.get_flag("no_daemon"),
        }
    }
}

pub fn cmd() -> Command {
    Command::new("param")
        .about("Various param subcommands")
        .aliases(&["p", "par"])
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
            Command::new("get")
            .about("Get a parameter value")
            .aliases(["g", "read"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get parameter from (e.g. '/talker')")
                .required(true)
                .value_name("NODE_NAME")
            )
            .arg(
                Arg::new("param_name")
                .help("Name of the ROS parameter to get (e.g. 'use_sim_time')")
                .required(true)
                .value_name("PARAM_NAME")
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
                Arg::new("hide_type")
                .long("hide-type")
                .aliases(&["hide_type"])
                .help("Hide the type information")
                .action(ArgAction::SetTrue)
            )
            .arg(output::arg())
        )
        .subcommand(
            Command::new("list")
            .about("Output a list of available parameters")
            .aliases(["l", "ls"])
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get parameters from (e.g. '/talker')")
                .required(true)
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
                Arg::new("param_prefixes")
                .long("param-prefixes")
                .aliases(&["param_prefixes"])
                .value_name("PARAM_PREFIXES")
                .num_args(1)
                .help("Only list parameters with the provided prefixes")
                .action(ArgAction::Append)
            )
            .arg(
                Arg::new("param_type")
                .long("param-type")
                .aliases(&["param_type"])
                .help("Print parameter types with parameter names")
                .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("filter")
                .long("filter")
                .aliases(&["regex"])
                .value_name("REGEX")
                .help("Only list parameters matching the provided regex")
            )
            .arg(output::arg())
        )
        .subcommand(
            Command::new("set")
            .about("Set a parameter value")
            .aliases(["s", "assign"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get parameter from (e.g. '/talker')")
                .required(true)
                .value_name("NODE_NAME")
            )
            .arg(
                Arg::new("param_name")
                .help("Name of the ROS parameter to get (e.g. 'use_sim_time')")
                .required(true)
                .value_name("PARAM_NAME")
            )
            .arg(
                Arg::new("value")
                .help("Value to set the parameter to (e.g. 'true')")
                .required(true)
                .value_name("VALUE")
                .action(ArgAction::Append)
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
            .arg(output::arg())
        )
        .subcommand(
            Command::new("export")
            .about("Dump all parameters to a file")
            .aliases(["e", "dump"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get parameter from (e.g. '/talker')")
                .required(true)
                .value_name("NODE_NAME")
            )
            .arg(
                Arg::new("output_dir")
                .long("output-dir")
                .short('o')
                .value_name("OUTPUT_DIR")
                .num_args(1)
                .help("The absolute path where to dump the generated file")
                .action(ArgAction::Append)
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
            .arg(output::arg())
        )
        .subcommand(
            Command::new("remove")
            .about("Remove a parameter")
            .aliases(["r", "delete", "del", "rm"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get parameter from (e.g. '/talker')")
                .required(true)
                .value_name("NODE_NAME")
            )
            .arg(
                Arg::new("param_name")
                .help("Name of the ROS parameter to get (e.g. 'use_sim_time')")
                .required(true)
                .value_name("PARAM_NAME")
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
            .arg(output::arg())
        )
        .subcommand(
            Command::new("describe")
            .about("Show information about a parameter")
            .aliases(["d", "info"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get parameter from (e.g. '/talker')")
                .required(true)
                .value_name("NODE_NAME")
            )
            .arg(
                Arg::new("param_name")
                .help("Name of the ROS parameter to get (e.g. 'use_sim_time')")
                .required(true)
                .value_name("PARAM_NAME")
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
            .arg(output::arg())
        )
        .subcommand(
            Command::new("import")
            .about("Load parameters from a file")
            .aliases(["i", "load"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("node_name")
                .help("Name of the ROS node to get parameter from (e.g. '/talker')")
                .required(true)
                .value_name("NODE_NAME")
            )
            .arg(
                Arg::new("param_name")
                .help("Path to the file to load parameters from (e.g. '/home/user/params.yaml')")
                .required(true)
                .value_name("PARAM_FILE")
            )
            .arg(
                Arg::new("no_use_wildcard")
                .long("no-use-wildcard")
                .aliases(&["wild"])
                .help("Do not load parameters in the '/**' namespace into the node")
                .action(ArgAction::SetTrue)
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
            .arg(output::arg())
        )
}

#[cfg(test)]
mod tests {
    use super::cmd;

    #[test]
    fn param_output_mode_is_available_for_describe_remove_and_import() {
        cmd()
            .try_get_matches_from([
                "param",
                "describe",
                "/demo",
                "answer",
                "--output",
                "json",
            ])
            .expect("describe should accept --output");

        cmd()
            .try_get_matches_from([
                "param",
                "remove",
                "/demo",
                "answer",
                "--output",
                "plain",
            ])
            .expect("remove should accept --output");

        cmd()
            .try_get_matches_from([
                "param",
                "import",
                "/demo",
                "params.yaml",
                "--output",
                "human",
            ])
            .expect("import should accept --output");
    }
}
