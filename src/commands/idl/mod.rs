pub mod discovery;
pub mod protobuf;
pub mod ros2msg;

use crate::commands::cli::print_error_and_exit;
use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("protobuf", submatch)) => {
            protobuf::handle(submatch.clone());
        }
        Some(("proto", submatch)) => {
            protobuf::handle(submatch.clone());
        }
        Some(("pb", submatch)) => {
            protobuf::handle(submatch.clone());
        }
        Some(("ros2msg", submatch)) => {
            ros2msg::handle(submatch.clone());
        }
        Some(("msg", submatch)) => {
            ros2msg::handle(submatch.clone());
        }
        Some(("ros2", submatch)) => {
            ros2msg::handle(submatch.clone());
        }
        _ => print_error_and_exit("Unknown IDL subcommand"),
    }
}
