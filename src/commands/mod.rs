pub mod action;
pub mod topic;
pub mod service;
pub mod param;
pub mod node;
pub mod interface;
pub mod frame;

pub mod run;
pub mod launch;
pub mod work;

pub mod bag;
pub mod daemon;
pub mod middleware;
pub mod completion;
pub mod complete;

use clap::ArgMatches;

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
            completion::handle(submatch.clone());
        }
        Some(("_complete", submatch)) => {
            complete::handle(submatch.clone());
        }
        _ => unreachable!("UNREACHABLE"),
    };
}