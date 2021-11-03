mod color;
mod pr;
mod config;
mod issues;
mod file;
mod terminal;
mod status;
use std::{io::{self, Read, Write}, process::Command};
use std::str::from_utf8;
use structopt::{StructOpt};

#[derive(StructOpt)]
#[structopt(name="gg", about="A command line tool for organizing tasks and git commits/PRs")]
struct GG {
    #[structopt(subcommand)]
    cmd: Cmd
}

#[derive(StructOpt, Debug)]
#[structopt(about = "A command line tool for organizing tasks and git commits/PRs")]
enum Cmd {
    #[structopt(about = "Initialize this repo with the GG Config")]
    Init {},
    #[structopt(about = "Create a new git branch")]
    New {
        #[structopt(short,long)]
        feature: String
    },
    #[structopt(about = "Push the current branch to origin")]
    Push {
        #[structopt(short,long)]
        force: bool
    },
    #[structopt(about = "Create a github pr for the current branch")]
    Pr {},
    #[structopt(about = "Fetch the current master/main.")]
    Fetch {},
    #[structopt(about = "Run a fixup rebase on the current branch.")]
    Fixup {},
    #[structopt(about = "Show the status of the current branch (or all the branches)")]
    Log {
        #[structopt(short,long)]
        all: bool
    },
    #[structopt(about = "Land the current PR")]
    Land {},
    #[structopt(about = "Rebase the current branch onto master/main")]
    Rebase {
        #[structopt(short,long)]
        interactive: bool
    },
    #[structopt(about = "Manage issues")]
    Issue(IssueSubcommand),
    #[structopt(about = "open a tui terminal to view issues")]
    Terminal {},
    #[structopt(about = "Manage status/daily record info")]
    Status(StatusSubcommand),
    #[structopt(about = "dumps debug info")]
    Debug {}, 
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Commands for managing github issues")]
enum IssueSubcommand {
    #[structopt(about = "Create a new github issue")]
    Create {
        #[structopt(short,long)]
        title: String
    },
    #[structopt(about = "List open github issues")]
    List {},
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Commands for managing the status file")]
enum StatusSubcommand {
    #[structopt(about = "Write a new entry for today.")]
    Write {
        #[structopt(short,long)]
        body: String,

        #[structopt(short,long)]
        todo: bool,
    },
    #[structopt(about = "List the existing tasks.")]
    List {},
}

#[tokio::main]
async fn main() ->  Result<(), Box<dyn std::error::Error>> {
    let opt = GG::from_args();
    match opt.cmd {
        Cmd::New { feature } => {
            new(feature.as_str());
        },
        Cmd::Push { force} => {
            push(current_branch(), force);
        },
        Cmd::Pr {} => {
            let branch = current_branch();
            push(branch.clone(), true);
            pr::create_pr(branch).await.expect("error creating PR");
        },
        Cmd::Fetch {} => {
            fetch_main();
        },
        Cmd::Fixup {} => {
            fixup_main();
        },
        Cmd::Log { all } => {
            log(all).await;
        },
        Cmd::Land {} => {
            let cfg = config::get_full_config();
            let branch = current_branch();
            pr::land_pr(branch.clone()).await.expect("error landing PR");
            fetch_main();
            checkout_main();
            delete_branch(branch);
            let selected_issue = config::get_selected_issue_number();
            if selected_issue > 0 {
                let issue = issues::get_issue(selected_issue).await?;
                print!("Close issue '{}' github.com/{}/{}/issues/{}?\n[y/n]: ", issue.title, cfg.repo_org, cfg.repo_name, selected_issue);
                io::stdout().flush().unwrap();
                let res = std::io::stdin().bytes().next().and_then(|result| result.ok()).unwrap() as char;
                if res == 'y' {
                    issues::close_issue(selected_issue).await?;
                    config::update_selected_issue(0);
                }
            }
        },
        Cmd::Rebase { interactive } => {
            rebase(interactive);
        },
        Cmd::Issue(issue) => {
            match issue {
                IssueSubcommand::Create { title} => {
                    issues::create_issue(title.as_str(), "").await.expect("error creating");
                }
                IssueSubcommand::List {} => {
                    issues::list_issues().await.expect("error creating");
                }
            }
        },
        Cmd::Terminal {} => {
            terminal::start_terminal().await.unwrap();
        }
        Cmd::Status(cmd) => {
            match cmd {
                StatusSubcommand::Write { body, todo } => {
                    status::write_status(body, todo);
                },
                StatusSubcommand::List {  } => {
                    status::list_statuses();
                },
            }
        },
        Cmd::Debug {  } => {
            let cfg = config::get_full_config();
            dbg!(cfg);
        },
        Cmd::Init {  } => {
            config::get_full_config();
        },
    }
    Ok(())
}

async fn log(all: bool) {
    let mut branches = vec![];
    if all {
        branches = all_branches();
    } else {
        branches.push(current_branch());
    }
    for branch in branches {
        pr::pr_statuses(branch).await.expect("error seeing PR");
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

fn all_branches() -> Vec<String> {
    let cfg = config::get_full_config();
    let out = match Command::new("git")
            .arg("branch")
            .output() {
                Ok(output) => output,
                Err(_e) => panic!("error!")
    };
    let x: &[_] = &[' ', '\t', '\n', '\r', '*'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    let mut branches = vec![];
    for line in result.split('\n') {
        let trimmed_line = line.trim_matches(x);
        if trimmed_line.starts_with(cfg.saved.branch_prefix.as_str()) {
            branches.push(trimmed_line.to_string());
        }
    }
    branches
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
    result.to_string()
}

fn push(full_branch: String, force: bool) {
    let mut command = Command::new("git");
    let c = command.arg("push");

    if force {
        c.arg("-f");
    }

    let res = c.arg("origin")
     .arg(full_branch)
     .status()
     .expect("did not get successful response.");

     if res.success() {
         println!("{}", color::bold(color::green("Success!")));
     } else {
         println!("{}", color::bold(color::red("Error pushing! (try '-f')")))
     }
}

fn fetch_main() {
    let cfg = config::get_saved_config();
    Command::new("git")
            .arg("fetch")
            .arg("-p")
            .arg("origin")
            .arg(cfg.repo_main_branch)
            .output()
            .expect("failed to fetch main branch");
}

fn fixup_main() {
    // TODO USE START/END BRANCHES 
    let cfg = config::get_full_config();
    Command::new("git")
            .arg("rebase")
            .arg("-i")
            .arg(format!("origin/{}",cfg.saved.repo_main_branch))
            .status()
            .expect("failed to fixup main branch");
}

fn rebase(interactive: bool) {
    let cfg = config::get_saved_config();
    let mut com = Command::new("git");
    let c = com.arg("rebase");
    if interactive {
        c.arg("-i");
    }

    c.arg(format!("origin/{}", cfg.repo_main_branch))
            .output()
            .expect("failed to rebase");
}

fn checkout_main() {
    let cfg = config::get_saved_config();
    Command::new("git")
        .arg("checkout")
        .arg(format!("origin/{}", cfg.repo_main_branch))
        .output()
        .expect("failed to checkout main");
}

fn delete_branch(branch: String) {
    Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg(branch)
        .output()
        .expect("failed to delete branch");
}