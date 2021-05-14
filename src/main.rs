#[macro_use]
extern crate clap;
extern crate ignore;
extern crate notify;

use std::env;

mod cli_args;
mod gitignore_filter;
mod rocket_watcher;

use cli_args::*;
use gitignore_filter::*;
use rocket_watcher::*;

fn main() {
    let args = parse_args();

    match args.change_directory {
        Some(dir) => match env::set_current_dir(&dir) {
            Err(err) => {
                println!("failed to cd to '{}': {}", dir, err);
                std::process::exit(2);
            }
            _ => {}
        },
        _ => {}
    }

    let watch_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            println!("failed to determine current director: {}", err);
            std::process::exit(3);
        }
    };

    let filter = GitignoreFilter::build(&watch_dir);
    let watchy = RocketWatch::new(filter);

    watchy.watch_directory(
        watch_dir
            .to_str()
            .expect("if you use an OS where paths aren't unicode, your mom's a hoe"),
    );
}
