mod color;
mod config;
mod git;
mod github;
mod file;
mod terminal;
mod status;
mod pomodoro;
use std::{io::{self, Read, Write}};
use github::GithubRepo;
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
    #[structopt(about = "Open a tui terminal for branches record info")]
    Branches {},
    #[structopt(about = "Manage status/daily record info")]
    Status(StatusSubcommand),
    #[structopt(about = "Starts a pomodoro clock")]
    Pomodoro {
        #[structopt(short,long, default_value = "25")]
        duration_mins: u32
    },
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
            git::new(feature.as_str());
        },
        Cmd::Push { force} => {
            git::push(git::current_branch(), force);
        },
        Cmd::Pr {} => {
            let branch = git::current_branch();
            git::push(branch.clone(), true);
            let cfg = config::get_full_config();
            let github = GithubRepo::new(cfg).await;
            github.create_pr(branch).await.expect("error creating PR");
        },
        Cmd::Fetch {} => {
            git::fetch_main();
        },
        Cmd::Fixup {} => {
            git::fixup_main();
        },
        Cmd::Log { all } => {
            log(all).await;
        },
        Cmd::Land {} => {
            let cfg = config::get_full_config();
            let github = GithubRepo::new(cfg).await;
            let branch = git::current_branch();
            github.land_pr(branch.clone()).await.expect("error landing PR");
            git::fetch_main();
            git::checkout_main();
            git::delete_branch(branch);
            let selected_issue = config::get_selected_issue_number();
            if selected_issue > 0 {
                let issue = github.get_issue(selected_issue).await?;
                print!("Close issue '{}' github.com/{}/{}/issues/{}?\n[y/n]: ", issue.title, github.org, github.repo, selected_issue);
                io::stdout().flush().unwrap();
                let res = std::io::stdin().bytes().next().and_then(|result| result.ok()).unwrap() as char;
                if res == 'y' {
                    github.close_issue(selected_issue).await?;
                    config::update_selected_issue(0);
                }
            }
        },
        Cmd::Rebase { interactive } => {
            git::rebase(interactive);
        },
        Cmd::Issue(issue) => {
            let cfg = config::get_full_config();
            let github = GithubRepo::new(cfg).await;
            match issue {
                IssueSubcommand::Create { title} => {
                    github.create_issue(title.as_str(), "").await.expect("error creating");
                }
                IssueSubcommand::List {} => {
                    github.list_issues().await.expect("error creating");
                }
            }
        },
        Cmd::Branches {  } => {
            terminal::start_pr_terminal().await.unwrap();
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
        Cmd::Pomodoro { duration_mins } => {
            pomodoro::run_pomodoro(duration_mins);
        },
    }
    Ok(())
}

async fn log(all: bool) {
    let mut branches = vec![];
    if all {
        branches = git::all_branches();
    } else {
        branches.push(git::current_branch());
    }
    let cfg = config::get_full_config();
    let github = GithubRepo::new(cfg).await;
    for branch in branches {
        github.pr_status(branch).await.expect("error seeing PR");
    }
}

