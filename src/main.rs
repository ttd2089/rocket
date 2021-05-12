extern crate notify;
#[macro_use]
extern crate clap;

use clap::{App, Arg};
use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

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
        .get_matches();

    let dir = matches
        .value_of("change-directory")
        .map(|s| s.to_string())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.into_os_string().into_string().ok())
        })
        .unwrap_or(".".to_string());

    watch_directory(&dir);
}

fn watch_directory(dir: &str) {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(dir, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}


#[cfg(target_family = "windows")]
fn get_default_shell() -> String
{
    return "pwsh.exe".into();
}

#[cfg(target_family = "unix")]
fn get_default_shell() -> String {
    return "/usr/bin/env sh"
}
