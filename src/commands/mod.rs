pub mod action;
pub mod frame;
pub mod idl;
pub mod interface;
pub mod node;
pub mod param;
pub mod service;
pub mod topic;

pub mod launch;
pub mod run;
pub mod work;

pub mod bag;
pub mod daemon;
pub mod middleware;

use clap::ArgMatches;

use crate::completions::handler as completion_handler;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("action", submatch)) => {
            action::handle(submatch.clone());
        }
        Some(("topic", submatch)) => {
            topic::handle(submatch.clone());
        }
        Some(("service", submatch)) => {
            service::handle(submatch.clone());
        }
        Some(("param", submatch)) => {
            param::handle(submatch.clone());
        }
        Some(("node", submatch)) => {
            node::handle(submatch.clone());
        }
        Some(("interface", submatch)) => {
            interface::handle(submatch.clone());
        }
        Some(("idl", submatch)) => {
            idl::handle(submatch.clone());
        }
        Some(("frame", submatch)) => {
            frame::handle(submatch.clone());
        }
        Some(("run", submatch)) => {
            run::handle(submatch.clone());
        }
        Some(("launch", submatch)) => {
            launch::handle(submatch.clone());
        }
        Some(("work", submatch)) => {
            work::handle(submatch.clone());
        }
        Some(("bag", submatch)) => {
            bag::handle(submatch.clone());
        }
        Some(("daemon", submatch)) => {
            daemon::handle(submatch.clone());
        }
        Some(("middleware", submatch)) => {
            middleware::handle(submatch.clone());
        }
        Some(("completion", submatch)) => {
            completion_handler::handle(submatch.clone());
        }
        Some(("_complete", submatch)) => {
            completion_handler::internal(submatch.clone());
        }
        _ => unreachable!("UNREACHABLE"),
    };
}
