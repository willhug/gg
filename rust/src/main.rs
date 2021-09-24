use clap::{App, Arg};
use std::process::Command;
use std::str::from_utf8;
use octocrab::Octocrab;

#[tokio::main]
async fn main() ->  Result<(), Box<dyn std::error::Error>> {
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
            .arg(Arg::new("force")
                .short('f')
                .about("force the push")
            )
            .about("pushes the current branch to master")
        )
        .subcommand(App::new("pr")
            .about("creates a PR for the current branch")
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("new") {
        if let Some(name) = matches.value_of("name") {
            new(name)
        }
    }
    if let Some(ref matches) = matches.subcommand_matches("push") {
        let force = matches.is_present("force");
        push(current_branch(), force)
    }
    if let Some(ref _matches) = matches.subcommand_matches("pr") {
        let branch = current_branch();
        push(branch.clone(), true);
        create_pr(branch).await.expect("error creating PR");
    }
    Ok(())
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


async fn create_pr(full_branch: String) -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    // TODO add real info
    octo.pulls("willhug", "gg")
        .create("test test test", full_branch, "main")
        .body("this is a test")
        .send()
        .await?;

    println!("RAN THE PULL");

    Ok(())
}