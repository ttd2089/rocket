#[macro_use]
extern crate clap;
extern crate ignore;
extern crate notify;

use std::path::PathBuf;

mod cli_args;
mod filters;
mod rocket_watcher;

use cli_args::*;
use rocket_watcher::*;

fn main() {
    
    let args = parse_args();

    let dir = args
        .change_directory
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
        })
        .unwrap_or_else(|| ".".to_string());

    let filter = gitignore_filter(&dir);
    let watchy = RocketWatch::new(filter);
    watchy.watch_directory(&dir)
}

fn gitignore_filter(dir: &str) -> GitignoreFilter {
    // todo: is this an overridable convention we need to respect?
    const GITIGNORE_FILENAME: &str = ".gitignore";

    let mut ignorers = Vec::new();

    let mut gitignore_dir = PathBuf::from(dir);

    while gitignore_dir.parent() != None {
        let mut builder = ignore::gitignore::GitignoreBuilder::new(gitignore_dir.as_path());

        // Update the dir to have the file name instead of making a copy and we'll .pop() twice to
        // traverse upward to the parent dir.
        gitignore_dir.push(GITIGNORE_FILENAME);
        println!(
            "looking for {}",
            gitignore_dir
                .to_str()
                .expect("if you use an OS where paths aren't unicode, your mom's a hoe")
        );
        let path = gitignore_dir.as_path();
        println!("path = {:?}", path);

        match builder.add(path) {
            None => match builder.build() {
                Ok(ignorer) => ignorers.push(ignorer),
                Err(err) => println!("error building ignorer for {:?}: {:?}", path, err),
            },
            Some(err) => {
                println!("error adding {:?}: {:?}", path, err);
            }
        }
        let _ = gitignore_dir.pop();
        let _ = gitignore_dir.pop();
    }

    return GitignoreFilter::new(ignorers);
}
