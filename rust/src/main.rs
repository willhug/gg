use clap::{App, Arg};
use std::process::Command;
use std::str::from_utf8;


fn main() {
    let matches = App::new("gg")
        .version("0.1")
        .author("Will H")
        .about("A command line tool for organizing tasks and git commits/PRs")
        .subcommand(App::new("new")
            .about("creates a new branch")
            .arg(Arg::new("name")
                .short('n')
                .value_name("NAME")
                .about("sets the name for a branch")
                .takes_value(true)
            )
        )
        .subcommand(App::new("push")
            .about("pushes the current branch to master")
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("new") {
        if let Some(name) = matches.value_of("name") {
            new(name)
        }
    }
    if let Some(ref _matches) = matches.subcommand_matches("push") {
        push(current_branch(), false)
    }
}


fn new(chosen_name: &str) {
    let branch: String = ["wh", chosen_name].join("/");
    Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg(branch)
            .output()
            .expect("failed to create branch");
}

fn current_branch() -> String {
    let out = match Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output() {
                Ok(output) => output,
                Err(_e) => panic!("error!")
    };
    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    return result.to_string()
}

fn push(full_branch: String, force: bool) {
    let mut command = Command::new("git");
    let c = command.arg("push");

    if force {
        c.arg("-f");
    }

    c.arg("origin")
     .arg(full_branch)
     .output()
     .expect("failed to push branch");
}
