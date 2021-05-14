#[macro_use]
extern crate clap;
extern crate ignore;
extern crate notify;

use clap::{App, Arg};
use std::path::{PathBuf};

mod rocket_watcher;
mod filters;

use rocket_watcher::*;

fn main() {

    let default_shell = get_default_shell();

    let matches = App::new("Rocket - Pocket rewritten in rust")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("change-directory")
            .short("C")
            .long("--change-directory")
            .value_name("directory")
            .help("Sets the working directory of the associated command"))
        .arg(Arg::with_name("shell")
            .short("s")
            .long("shell")
            .value_name("shell")
            .help("The shell that will be used to execute commands")
            .default_value(&default_shell))
        .arg(Arg::with_name("log")
            .short("l")
            .long("log")
            .help("Write application logs to stderr"))
        .arg(Arg::with_name("command")
            .index(1)
            .value_name("command")
            .required(true))
        .get_matches();

    let dir = matches
        .value_of("change-directory")
        // todo: check that this is valid path and that the directory exists, right now we'll just
        // fail when the first thing tries to use it.
        .map(|s| s.to_string())
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

#[cfg(target_family = "windows")]
fn get_default_shell() -> String
{
    return "pwsh.exe".into();
}

#[cfg(target_family = "unix")]
fn get_default_shell() -> String {
    return "/usr/bin/env sh".into();
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
        println!("looking for {}", gitignore_dir.to_str()
            .expect("if you use an OS where paths aren't unicode, your mom's a hoe"));
        let path = gitignore_dir.as_path();
        println!("path = {:?}", path);

        match builder.add(path) {
            None => {
                match builder.build() {
                    Ok(ignorer) => ignorers.push(ignorer),
                    Err(err) => println!("error building ignorer for {:?}: {:?}", path, err),
                }
            },
            Some(err) => {
                println!("error adding {:?}: {:?}", path, err);
            },
        }
        let _ = gitignore_dir.pop();
        let _ = gitignore_dir.pop();
    }

    return GitignoreFilter::new(ignorers);
}
