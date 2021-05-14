use crate::rocket_watcher;
use std::fs;
use std::path::{Path, PathBuf};

// todo: are these an overridable conventions we need to respect?
const GITIGNORE_FILENAME: &str = ".gitignore";
const GIT_METADATA_DIR_NAME: &str = ".git";

pub struct GitignoreFilter {
    ignorers: Vec<ignore::gitignore::Gitignore>,
}

impl GitignoreFilter {
    fn new(ignorers: Vec<ignore::gitignore::Gitignore>) -> GitignoreFilter {
        GitignoreFilter { ignorers }
    }

    pub fn build(mut dir: PathBuf) -> GitignoreFilter {
        // Get all the child gitignores then add the parents.

        let mut ignorers = get_all_gitignores(dir.as_path());

        // We only want ancestors up to and including the root of the repo. If we don't find a root
        // we'll ignore all the ancestors we found.
        let mut ancestors = Vec::new();
        let mut found_root = false;
        while dir.pop() {
            match get_gitignore(dir.as_path()) {
                Some(ignorer) => ancestors.push(ignorer),
                None => {}
            }
            if is_git_repo_root(&dir) {
                found_root = true;
                break;
            }
        }
        if found_root {
            ignorers.append(&mut ancestors);
        }

        return GitignoreFilter::new(ignorers);
    }
}

// todo: Handle errors better.
// Right now any error is (logged, then) treated like there is no .gitignore. We should probably
// return any non-not-found errors instead since we're not sure we're accurately representing the
// filters described by the .gitignores.

fn get_gitignore(dir: &Path) -> Option<ignore::gitignore::Gitignore> {
    if path_in_git_metadata(dir) {
        return None;
    }
    let mut builder = ignore::gitignore::GitignoreBuilder::new(dir);
    let mut filepath = dir.to_path_buf();
    filepath.push(GITIGNORE_FILENAME);
    match builder.add(filepath) {
        None => match builder.build() {
            Ok(ignorer) => return Some(ignorer),
            Err(err) => {
                println!("error building ignorer for {:?}: {:?}", dir, err);
                return None;
            }
        },
        Some(err) => {
            println!("error adding {:?}: {:?}", dir, err);
            return None;
        }
    }
}

fn get_all_gitignores(dir: &Path) -> Vec<ignore::gitignore::Gitignore> {
    let mut ignores = Vec::new();
    if !dir.is_dir() || path_in_git_metadata(dir) {
        return ignores;
    }
    let readdir = match fs::read_dir(dir) {
        Ok(readdir) => readdir,
        Err(err) => {
            println!("error reading filesystem entries for '{:?}': {}", dir, err);
            return ignores;
        }
    };
    for entry in readdir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                println!("error reading filesystem entry: {}", err);
                continue;
            }
        };
        ignores.append(&mut get_all_gitignores(&entry.path()));
    }
    match get_gitignore(dir) {
        Some(ignorer) => ignores.push(ignorer),
        None => {}
    }
    return ignores;
}

fn path_in_git_metadata(path: &Path) -> bool {
    return path.iter().any(|e| e == GIT_METADATA_DIR_NAME);
}

fn is_git_repo_root(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    let readdir = match fs::read_dir(path) {
        Ok(readdir) => readdir,
        Err(err) => {
            println!("error reading filesystem entries for '{:?}': {}", path, err);
            return false;
        }
    };
    for entry in readdir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                println!("error reading filesystem entry: {}", err);
                return false;
            }
        };
        if entry.file_name() == GIT_METADATA_DIR_NAME {
            return true;
        }
    }
    return false;
}

impl rocket_watcher::PathFilter for GitignoreFilter {
    fn exclude(&self, path: &Path) -> bool {
        if path_in_git_metadata(path) {
            return true;
        }
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
