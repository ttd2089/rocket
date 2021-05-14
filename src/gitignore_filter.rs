use crate::rocket_watcher;
use std::path::{Path, PathBuf};

pub struct GitignoreFilter {
    ignorers: Vec<ignore::gitignore::Gitignore>,
}

impl GitignoreFilter {
    fn new(ignorers: Vec<ignore::gitignore::Gitignore>) -> GitignoreFilter {
        GitignoreFilter { ignorers }
    }

    pub fn build(mut dir: PathBuf) -> GitignoreFilter {
        // todo: is this an overridable convention we need to respect?
        const GITIGNORE_FILENAME: &str = ".gitignore";

        let mut ignorers = Vec::new();

        while dir.parent() != None {
            let mut builder = ignore::gitignore::GitignoreBuilder::new(dir.as_path());

            // Update the dir to have the file name instead of making a copy and we'll .pop() twice to
            // traverse upward to the parent dir.
            dir.push(GITIGNORE_FILENAME);
            println!(
                "looking for {}",
                dir.to_str()
                    .expect("if you use an OS where paths aren't unicode, your mom's a hoe")
            );
            let path = dir.as_path();
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
            let _ = dir.pop();
            let _ = dir.pop();
        }

        return GitignoreFilter::new(ignorers);
    }
}

impl rocket_watcher::PathFilter for GitignoreFilter {
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
