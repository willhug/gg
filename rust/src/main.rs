mod color;
mod config;
mod git;
mod git_rebase;
mod github;
mod file;
mod terminal;
mod status;
mod pomodoro;
use std::io::{self, Read, Write};
use anyhow::Result;
use config::get_saved_config;
use git::{current_parsed_branch, diff};
use git_rebase::{abort_rebase, continue_rebase, start_rebase};
use github::GithubRepo;
use structopt::{StructOpt};

use crate::git::parse_branch;

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
    #[structopt(about = "Checkout a branch", alias = "co")]
    Checkout {
        #[structopt(short,long)]
        next: bool,
        #[structopt(short,long)]
        prev: bool,
    },
    #[structopt(about = "Create a new git branch")]
    New {
        #[structopt(short,long)]
        feature: Option<String>,
        #[structopt(short,long)]
        part: Option<f32>
    },
    #[structopt(about = "Push the current branch to origin")]
    Push {
        #[structopt(short,long)]
        force: bool,
        #[structopt(short,long)]
        start: bool
    },
    #[structopt(about = "Create a github pr for the current branch")]
    Pr {
        #[structopt(short = "s", long = "use-start")]
        use_start: bool
    },
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
    RebaseOld {
        #[structopt(short,long)]
        interactive: bool
    },
    #[structopt(about = "Rebase the current branch with stacking", alias="rs")]
    Rebase {
        #[structopt(short,long)]
        onto: Option<String>,
        #[structopt(short,long)]
        strategy: Option<String>,
        #[structopt(short="a",long="abort")]
        rebase_abort: bool,
        #[structopt(short="c",long="continue")]
        rebase_continue: bool,
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
    #[structopt(about = "Deletes a branch (current one currently)", alias = "del")]
    Delete {
        #[structopt(about = "Branch to delete", short,long)]
        branch: Option<String>,
        #[structopt(about = "Branch to checkout", short,long)]
        dest: Option<String>,
    },
    #[structopt(about = "Shows the current diff for the branch")]
    Diff {},
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
        Cmd::New { feature, part } => {
            let mut branch = git::current_parsed_branch();
            if branch.prefix.is_none() {
                branch.prefix = Some(config::get_saved_config().branch_prefix);
            }
            if let Some(feature) = feature {
                branch.base = feature;
            }
            if let Some(part) = part {
                let partx100 = (part * 100.0) as u32;
                branch.partx100 = Some(partx100);
            } else {
                branch.partx100 = match branch.partx100 {
                    Some(p) => Some(p + 100),
                    None => Some(100), // Default is 1
                };
            }
            git::new(branch.start().as_str());
            git::new(branch.full().as_str());
        },
        Cmd::Push { force, start} => {
            let cur = git::current_branch();
            git::push(&cur, force);
            if start {
                let parsed = git::parse_branch(cur);
                git::push(&parsed.start(), force);
            }
        },
        Cmd::Pr { use_start} => {
            let branch = git::current_parsed_branch();
            git::push(&branch.full(), true);
            let cfg = config::get_full_config();
            let github = GithubRepo::new(cfg).await;
            let base = match use_start {
                true => {
                    git::push(&branch.start(), true);
                    Some(branch.start())
                },
                false => None,
            };
            github.create_pr(branch.full(), base).await.expect("error creating PR");
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
            let branch = git::current_parsed_branch();
            github.land_pr(branch.full()).await.expect("error landing PR");
            git::fetch_main();
            git::checkout_main();
            git::delete_branch(branch.full());
            git::delete_branch(branch.start());
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
        Cmd::RebaseOld { interactive } => {
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
            println!("origin/master");
            git::get_commit_hash("origin/master".to_string());
            println!("origin/main");
            git::get_commit_hash("origin/main".to_string());
        },
        Cmd::Init {  } => {
            config::get_full_config();
        },
        Cmd::Pomodoro { duration_mins } => {
            pomodoro::run_pomodoro(duration_mins);
        },
        Cmd::Checkout { next, prev } => {
            let dir = if next {
                git::CheckoutDir::Next
            } else if prev {
                git::CheckoutDir::Prev
            } else {
                git::CheckoutDir::Unknown
            };
            match git::get_branch_for_dir(dir) {
                Some(x) => git::checkout(&x),
                None => println!("No branch found!"),
            };
        },
        Cmd::Delete { branch, dest } => {
            let no_branch = branch.is_none();
            let branch_to_delete = match branch {
                Some(branch) => parse_branch(branch),
                None => current_parsed_branch(),
            };
            if no_branch {
                // Checkout a different branch before deleting ourself.
                git::checkout(&match dest {
                    Some(dest) => dest,
                    None => match git::get_branch_for_dir(git::CheckoutDir::Prev) {
                        Some(x) => x,
                        None => format!("origin/{}", get_saved_config().repo_main_branch),
                    },
                });
            }
            git::delete_branch(branch_to_delete.full());
            git::delete_branch(branch_to_delete.start());
        },
        Cmd::Rebase {
            onto,
            strategy,
            rebase_abort,
            rebase_continue ,
        } => {
            if rebase_abort {
                abort_rebase();
            } else if rebase_continue {
                continue_rebase();
            } else {
                start_rebase(onto, strategy);
            }
        },
        Cmd::Diff {  } => {
            diff(current_parsed_branch().start(), None);
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

