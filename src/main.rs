#[macro_use]
extern crate clap;
extern crate ignore;
extern crate notify;

use clap::{App, Arg};
use notify::{watcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let matches = App::new("Rocket - Pocket rewritten in rust")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("change-directory")
                .short("-C")
                .long("--change-directory")
                .value_name("directory")
                .help("Sets the working directory of the associated command"),
        )
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
        .unwrap_or(".".to_string());

    watch_directory(&dir, &gitignore_filter(&dir));
}

fn watch_directory(dir: &str, global_filter: &dyn PathFilter) {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(dir, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                match get_path(&event) {
                    Some(path) => {
                        match global_filter.exclude(&path) {
                            true => println!("ignoring {:?}", event),
                            false => println!("caring about {:?}", event),
                        };
                    },
                    None => println!("event had no path"),
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn get_path(evt: &notify::DebouncedEvent) -> Option<&Path> {
    match evt {
        notify::DebouncedEvent::NoticeWrite(path) => Some(path),
        notify::DebouncedEvent::NoticeRemove(path) => Some(path),
        notify::DebouncedEvent::Create(path) => Some(path),
        notify::DebouncedEvent::Write(path) => Some(path),
        notify::DebouncedEvent::Chmod(path) => Some(path),
        notify::DebouncedEvent::Remove(path) => Some(path),
        notify::DebouncedEvent::Rename(from, _to) => Some(from),
        notify::DebouncedEvent::Error(_, Some(path)) => Some(path),
        _ => None,
    }
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

    return GitignoreFilter { ignorers };
}

trait PathFilter {
    fn exclude(&self, path: &Path) -> bool;
}

struct GitignoreFilter {
    ignorers: Vec<ignore::gitignore::Gitignore>
}

impl PathFilter for GitignoreFilter {
    fn exclude(&self, path: &Path) -> bool {
        for ignorer in &self.ignorers {
            // todo: figure out how to distinguish files from directories
            let resp = ignorer.matched(path, true);
            println!("{:?}", resp);
            match resp {
                ignore::Match::Ignore(_) => return true,
                ignore::Match::Whitelist(_) => return false,
                _ => {},
            }
        }
        return false;
    }
}
