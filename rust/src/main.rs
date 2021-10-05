#[path = "pr.rs"] mod pr;
#[path = "config.rs"] mod config;
#[path = "issues.rs"] mod issues;
use clap::{App, Arg};
use std::process::Command;
use std::str::from_utf8;

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
        .subcommand(App::new("fetch")
            .about("fetches the current master/main branch")
        )
        .subcommand(App::new("status")
            .about("fetches the current status of the remote branch.")
        )
        .subcommand(App::new("land")
            .about("lands the current branch (if possible).")
        )
        .subcommand(App::new("rebase")
            .about("rebase the current branch on master (if possible).")
            .arg(Arg::new("interactive")
                .short('i')
                .about("do interactive rebase")
            )
        )
        .subcommand(App::new("issue")
            .about("handles issues")
            .subcommand(App::new("create")
                .about("creates a new issue")
                .arg(Arg::new("title")
                    .short('t')
                    .value_name("TITLE")
                    .about("sets the title for an issue")
                    .takes_value(true)
                )
            )
            .subcommand(App::new("list")
                .about("lists all issues")
            )
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
        pr::create_pr(branch).await.expect("error creating PR");
    }
    if let Some(ref _matches) = matches.subcommand_matches("status") {
        let branch = current_branch();
        pr::pr_statuses(branch).await.expect("error seeing PR");
    }
    if let Some(ref _matches) = matches.subcommand_matches("fetch") {
        fetch_main()
    }
    if let Some(ref _matches) = matches.subcommand_matches("land") {
        let branch = current_branch();
        pr::land_pr(branch).await.expect("error landing PR");
    }
    if let Some(ref matches) = matches.subcommand_matches("rebase") {
        let interactive = matches.is_present("interactive");
        rebase(interactive);
    }
    if let Some(ref matches) = matches.subcommand_matches("issue") {
        if let Some(ref matches) = matches.subcommand_matches("create") {
            if let Some(name) = matches.value_of("title") {
                issues::create_issue(name, "").await.expect("error creating");
            }
        }
        if let Some(ref _matches) = matches.subcommand_matches("list") {
            issues::list_issues().await.expect("error creating");
        }
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

fn fetch_main() {
    let cfg = config::get_config();
    Command::new("git")
            .arg("fetch")
            .arg("-p")
            .arg("origin")
            .arg(cfg.repo_main_branch)
            .output()
            .expect("failed to fetch main branch");
}

fn rebase(interactive: bool) {
    let cfg = config::get_config();
    let mut com = Command::new("git");
    let c = com.arg("rebase");
    if interactive {
        c.arg("-i");
    }

    c.arg(format!("origin/{}", cfg.repo_main_branch))
            .output()
            .expect("failed to rebase");
}