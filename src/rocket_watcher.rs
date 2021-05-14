use notify::{watcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub trait PathFilter {
    fn exclude(&self, path: &Path) -> bool;
}

pub struct GitignoreFilter {
    ignorers: Vec<ignore::gitignore::Gitignore>,
}

impl GitignoreFilter {
    pub fn new(ignorers: Vec<ignore::gitignore::Gitignore>) -> GitignoreFilter {
        GitignoreFilter { ignorers }
    }
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
                _ => {}
            }
        }
        false
    }
}

//He who watches
pub struct RocketWatch<T: PathFilter> {
    filter: T,
}

impl<T: PathFilter> RocketWatch<T> {
    pub fn watch_directory(self, dir: &str) {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        watcher.watch(dir, RecursiveMode::Recursive).unwrap();

        loop {
            match rx.recv() {
                Ok(event) => match get_path(&event) {
                    Some(path) => {
                        match self.filter.exclude(&path) {
                            true => println!("ignoring {:?}", event),
                            false => println!("caring about {:?}", event),
                        };
                    }
                    None => println!("event had no path"),
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }

    pub fn new(filter: T) -> RocketWatch<T> {
        RocketWatch { filter }
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
