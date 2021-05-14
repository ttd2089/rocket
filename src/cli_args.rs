use clap::{App, Arg};

#[derive(Debug)]
pub struct CliArgs {
    pub change_directory: Option<String>,
    pub shell: String,
    pub log: bool,
    pub command: Option<String>,
}

pub fn parse_args() -> CliArgs {

    let default_shell = get_default_shell();

    let matches = App::new("Rocket - Pocket rewritten in rust")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("change-directory")
                .short("C")
                .long("--change-directory")
                .value_name("directory")
                .help("Sets the working directory of the associated command"),
        )
        .arg(
            Arg::with_name("shell")
                .short("s")
                .long("shell")
                .value_name("shell")
                .help("The shell that will be used to execute commands")
                .default_value(&default_shell),
        )
        .arg(
            Arg::with_name("log")
                .short("l")
                .long("log")
                .help("Write application logs to stderr"),
        )
        .arg(
            Arg::with_name("command")
                .index(1)
                .value_name("command")
        )
        .get_matches();

    return CliArgs {
        change_directory: matches.value_of("change-directory").map(|s| s.to_string()),
        shell: matches.value_of("shell").map(|s| s.to_string()).expect("no shell specified and no default known for current system"),
        log: matches.occurrences_of("log") > 0,
        command: matches.value_of("command").map(|s| s.to_string()),
    };
}

#[cfg(target_family = "windows")]
fn get_default_shell() -> String {
    return "pwsh.exe".into();
}

#[cfg(target_family = "unix")]
fn get_default_shell() -> String {
    return "/usr/bin/env sh".into();
}