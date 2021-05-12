#[macro_use]
extern crate clap;
extern crate ignore;
extern crate notify;

use clap::{App, Arg};
use notify::{watcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::borrow::Borrow;

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

    //let filter = gitignore_filter(&dir).unwrap_or_else(|| Box::new(NothingFilter{}));
    let other_filter = other_gitignore_filter(&dir);

    if let Some(filter) = other_filter {
        other_watch_directory(&dir, &filter)
    }
    else{
        other_watch_directory(&dir, &NothingFilter{})
    }
    
    //let filter = filter.unwrap_or_else(|| &NothingFilter{}));

    //watch_directory(&dir, filter);

}

fn watch_directory(dir: &str, global_filter: Box<dyn PathFilter>) {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(dir, RecursiveMode::Recursive).unwrap();

    // loop {
    //     match rx.recv() {
    //         Ok(event) => {
    //             match global_filter.exclude(event) {
    //                 true => println!("ignoring {:?}", event),
    //                 true => println!("caring about {:?}", event),
    //             };
    //         },
    //         Err(e) => println!("watch error: {:?}", e),
    //     }
    // }
}

fn gitignore_filter(dir: &str) -> Option<Box<dyn PathFilter>> {

    // todo: is this an overridable convention we need to respect?
    const GITIGNORE_FILENAME: &str = ".gitignore";

    let mut gitignore_dir = PathBuf::from(dir);

    while gitignore_dir.parent() != None {
        // Update the dir to have the file name instead of making a copy and we'll .pop() twice to
        // traverse upward to the parent dir.
        gitignore_dir.push(GITIGNORE_FILENAME);
        println!("looking for {}", gitignore_dir.to_str()
            .expect("if you use an OS where paths aren't unicode, your mom's a hoe"));
        let ignorer = ignore::gitignore::Gitignore::new(gitignore_dir.as_path());
        match ignorer {
            (ignorer, None) => return Some(Box::new(GitignoreFilter{ ignorer:  ignorer })),
            _ => {},
        }
        let _ = gitignore_dir.pop();
        let _ = gitignore_dir.pop();
    }

    return None;
}

fn other_gitignore_filter(dir: &str) -> Option<GitignoreFilter> {

    // todo: is this an overridable convention we need to respect?
    const GITIGNORE_FILENAME: &str = ".gitignore";

    let mut gitignore_dir = PathBuf::from(dir);

    while gitignore_dir.parent() != None {
        // Update the dir to have the file name instead of making a copy and we'll .pop() twice to
        // traverse upward to the parent dir.
        gitignore_dir.push(GITIGNORE_FILENAME);
        println!("looking for {}", gitignore_dir.to_str()
            .expect("if you use an OS where paths aren't unicode, your mom's a hoe"));
        let ignorer = ignore::gitignore::Gitignore::new(gitignore_dir.as_path());
        match ignorer {
            (ignorer, None) => return Some(GitignoreFilter{ ignorer:  ignorer }),
            _ => {},
        }
        let _ = gitignore_dir.pop();
        let _ = gitignore_dir.pop();
    }

    return None;
}

fn other_watch_directory<T : PathFilter>(dir: &str, global_filter: &T)
{
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(dir, RecursiveMode::Recursive).unwrap();
}

trait PathFilter {
    fn exclude(&self, path: &Path) -> bool;
}

struct GitignoreFilter {
    ignorer: ignore::gitignore::Gitignore
}

impl PathFilter for GitignoreFilter {
    fn exclude(&self, path: &Path) -> bool {
        match self.ignorer.matched(path, false) {
            ignore::Match::Ignore(_) => true,
            _ => false,
        }
    }
}


struct NothingFilter {}

impl PathFilter for NothingFilter {
    fn exclude(&self, _: &Path) -> bool {
        return false;
    }
}
