pub mod about;
pub mod action;
pub mod frame;
pub mod interface;
pub mod node;
pub mod param;
pub mod service;
pub mod topic;

pub mod idl;
pub mod launch;
pub mod run;
pub mod work;

pub mod bag;
pub mod daemon;
pub mod middleware;
// completion subcommands consolidated under src/completions
pub use crate::completions::args as completion;
pub use crate::completions::internal as complete;

use clap::{arg, builder::styling, Command};
use colored::Colorize;

pub fn letter_str(letter: &str) -> String {
    let mut wrapped = "[".bright_green().to_string();
    wrapped.push_str(&letter.bright_green().italic().to_string());
    wrapped.push_str(&"]".bright_green().to_string());
    wrapped.push_str(&"  ".to_string());
    wrapped
}

pub fn command_str(word: &str) -> String {
    word.bright_green().bold().to_string()
}

pub fn descriptin_str(word: &str) -> String {
    word.bright_white().to_string()
}

const ABOUT_STR: &str = "a wannabe ros2 command line tool alternative";

pub fn cli(logo: bool) -> Command {
    let _logo_1: String = "
        ▄▄▄   "
        .bright_blue()
        .to_string()
        .to_owned()
        + "     ▄▄▄   ".bright_blue().to_string().as_str()
        + "     ▄▄▄     ".bright_green().to_string().as_str()
        + " 
      ▟█████▙ "
            .bright_blue()
            .to_string()
            .as_str()
        + "   ▟█████▙ ".bright_blue().to_string().as_str()
        + "   ▟█████▙   ".bright_green().to_string().as_str()
        + "   
      ▜█████▛ "
            .bright_blue()
            .to_string()
            .as_str()
        + "   ▜█████▛ ".bright_blue().to_string().as_str()
        + "   ▜█████▛   ".bright_green().to_string().as_str()
        + "   
        ▀▀▀   "
            .bright_blue()
            .to_string()
            .as_str()
        + "     ▀▀▀   ".bright_blue().to_string().as_str()
        + "     ▀▀▀     ".bright_green().to_string().as_str()
        + "   
        ▄▄▄   "
            .bright_blue()
            .to_string()
            .as_str()
        + "     ▄▄▄   ".bright_green().to_string().as_str()
        + "     ▄▄▄     ".bright_blue().to_string().as_str()
        + "   
      ▟█████▙ "
            .bright_blue()
            .to_string()
            .as_str()
        + "   ▟█████▙ ".bright_green().to_string().as_str()
        + "   ▟█████▙   ".bright_blue().to_string().as_str()
        + "   
      ▜█████▛ "
            .bright_blue()
            .to_string()
            .as_str()
        + "   ▜█████▛ ".bright_green().to_string().as_str()
        + "   ▜█████▛   ".bright_blue().to_string().as_str()
        + "   
        ▀▀▀   "
            .bright_blue()
            .to_string()
            .as_str()
        + "     ▀▀▀   ".bright_green().to_string().as_str()
        + "     ▀▀▀     ".bright_blue().to_string().as_str()
        + "   
        ▄▄▄   "
            .bright_blue()
            .to_string()
            .as_str()
        + "     ▄▄▄   ".bright_blue().to_string().as_str()
        + "     ▄▄▄     ".bright_green().to_string().as_str()
        + "   
      ▟█████▙ "
            .bright_blue()
            .to_string()
            .as_str()
        + "   ▟█████▙ ".bright_blue().to_string().as_str()
        + "   ▟█████▙   ".bright_green().to_string().as_str()
        + "   
      ▜█████▛ "
            .bright_blue()
            .to_string()
            .as_str()
        + "   ▜█████▛ ".bright_blue().to_string().as_str()
        + "   ▜█████▛   ".bright_green().to_string().as_str()
        + "   
        ▀▀▀   "
            .bright_blue()
            .to_string()
            .as_str()
        + "     ▀▀▀   ".bright_blue().to_string().as_str()
        + "     ▀▀▀     ".bright_green().to_string().as_str()
        + "\n";

    let logo_str: String = if logo { _logo_1 } else { String::new() };

    let help_str: String = " ".to_string().to_owned()
        + "
Usage:"
            .bright_blue()
            .bold()
            .to_string()
            .as_str()
        + "  roc".bright_green().bold().to_string().as_str()
        + " <COMMAND>".green().to_string().as_str()
        + "
      "
        .bright_blue()
        .bold()
        .to_string()
        .as_str()
        + "  roc".bright_green().bold().to_string().as_str()
        + " <C>".green().to_string().as_str()
        + "

Utilities Commands:"
            .bright_blue()
            .bold()
            .to_string()
            .as_str()
        + "
  " + &command_str("action")
        + "      "
        + &letter_str("a")
        + &descriptin_str("Various action subcommands")
        + "
  " + &command_str("topic")
        + "       "
        + &letter_str("t")
        + &descriptin_str("Various topic subcommands")
        + "
  " + &command_str("service")
        + "     "
        + &letter_str("s")
        + &descriptin_str("Various service subcommands")
        + "
  " + &command_str("param")
        + "       "
        + &letter_str("p")
        + &descriptin_str("Various param subcommands")
        + "
  " + &command_str("node")
        + "        "
        + &letter_str("n")
        + &descriptin_str("Various node subcommands")
        + "
  " + &command_str("interface")
        + "   "
        + &letter_str("i")
        + &descriptin_str("Various interface subcommands")
        + "
  " + &command_str("frame")
        + "       "
        + &letter_str("f")
        + &descriptin_str("Various transform subcommands [WIP]")
        + "

Workspace Commands:"
            .bright_blue()
            .bold()
            .to_string()
            .as_str()
        + "
  " + &command_str("run")
        + "         "
        + &letter_str("r")
        + &descriptin_str("Run an executable file")
        + "
  " + &command_str("launch")
        + "      "
        + &letter_str("l")
        + &descriptin_str("Run a launch file")
        + "
  " + &command_str("work")
        + "        "
        + &letter_str("w")
        + &descriptin_str("Packages and workspace")
        + "
  " + &command_str("idl")
        + "         "
        + &letter_str("d")
        + &descriptin_str("Various idl subcommands [WIP]")
        + "

Communication Commands:"
            .bright_blue()
            .bold()
            .to_string()
            .as_str()
        + "     
  " + &command_str("bag")
        + "         "
        + &letter_str("b")
        + &descriptin_str("ROS bag tools")
        + "
  " + &command_str("daemon")
        + "      "
        + &letter_str("d")
        + &descriptin_str("Daemon compatibility commands")
        + "
  " + &command_str("middleware")
        + "  "
        + &letter_str("m")
        + &descriptin_str("Middleware inspection and selection")
        + "

Shell Integration:"
            .bright_blue()
            .bold()
            .to_string()
            .as_str()
        + "
  " + &command_str("completion")
        + "  "
        + &letter_str("c")
        + &descriptin_str("Generate shell completions");

    let styles = styling::Styles::styled()
        .header(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .error(styling::AnsiColor::Red.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::Green.on_default());

    Command::new("roc")
        .styles(styles)
        .about(&ABOUT_STR) 
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_about("A ROS2 command line tool replacer that aims to be more user friendly and more powerful. [ALPHA STATE]")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(false)
        .disable_help_subcommand(true)
        .override_help(logo_str + &help_str)
        .subcommand(action::cmd())
        .subcommand(topic::cmd())
        .subcommand(service::cmd())
        .subcommand(param::cmd())
        .subcommand(node::cmd())
        .subcommand(interface::cmd())
        .subcommand(idl::cmd())
        .subcommand(frame::cmd())
        .subcommand(run::cmd())
        .subcommand(launch::cmd())
        .subcommand(work::cmd())
        .subcommand(bag::cmd())
        .subcommand(daemon::cmd())
        .subcommand(middleware::cmd())
        .subcommand(completion::cmd())
        .subcommand(complete::cmd())
        .arg(arg!(--about "about"))
}
