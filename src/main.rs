mod arguments;
mod commands;
mod completions;
mod utils;
use std::env;

fn main() {
    let show_logo = if env::args().len() > 1 { false } else { true };
    let matches = arguments::cli(show_logo).get_matches();
    commands::handle(matches);
}
