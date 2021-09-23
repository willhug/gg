use clap::App;
use std::process::Command;


fn main() {
    let matches = App::new("gg")
        .version("0.1")
        .author("Will H")
        .about("A command line tool for organizing tasks and git commits/PRs")
        .subcommand(App::new("new")
            .about("this is a test")
            .arg("-n, --name 'branch name")
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("new") {
        if let Some(name) = matches.value_of("name") {
            new(name)
        }
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