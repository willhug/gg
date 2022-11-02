mod color;
mod config;
mod file;
mod git;
mod git_rebase;
mod github;
mod pomodoro;
mod status;
mod terminal;
use anyhow::Result;
use config::get_saved_config;
use git::{current_parsed_branch, diff, sync};
use git_rebase::{abort_rebase, continue_rebase, fixup_rebase, rebase_all_children, start_rebase};
use github::{pr::Pr, GithubRepo};
use std::{
    collections::HashSet,
    io::{self, Read, Write},
};
use structopt::StructOpt;

use crate::{
    config::{get_full_config, update_prefix_and_split},
    git::{current_branch, delete_branch_all, parse_branch},
};

#[derive(StructOpt)]
#[structopt(
    name = "gg",
    about = "A command line tool for organizing tasks and git commits/PRs"
)]
struct GG {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "A command line tool for organizing tasks and git commits/PRs")]
enum Cmd {
    #[structopt(about = "Initialize this repo with the GG Config")]
    Init {},
    #[structopt(about = "Checkout a branch", alias = "co")]
    Checkout {
        #[structopt(short, long)]
        next: bool,
        #[structopt(short, long)]
        prev: bool,
        #[structopt(short = "a", long)]
        part: Option<f32>,
        #[structopt(short, long)]
        start: bool,
    },
    #[structopt(about = "Create a new git branch")]
    New {
        #[structopt(short, long)]
        feature: Option<String>,
        #[structopt(short, long)]
        part: Option<f32>,
        #[structopt(short, long)]
        main: bool,
    },
    #[structopt(about = "Push the current branch to origin")]
    Push {
        #[structopt(short, long)]
        force: bool,
        #[structopt(short, long)]
        start: bool,
    },
    #[structopt(about = "Create a github pr for the current branch")]
    Pr {
        #[structopt(short = "s", long = "use-start")]
        use_start: bool,
        #[structopt(short = "d", long = "draft")]
        is_draft: bool,
    },
    #[structopt(about = "Fetch the current master/main.")]
    Fetch {},
    #[structopt(about = "Run a fixup rebase on the current branch.")]
    Fixup {},
    #[structopt(about = "Set the base to the main branch")]
    Setbase {},
    #[structopt(about = "Show the status of the current branch (or all the branches)")]
    Log {
        #[structopt(short, long)]
        all: bool,
    },
    #[structopt(about = "Land the current PR")]
    Land {},
    #[structopt(about = "Rebase the current branch onto master/main")]
    RebaseOld {
        #[structopt(short, long)]
        interactive: bool,
    },
    #[structopt(about = "Rebase the current branch with stacking", alias = "rs")]
    Rebase {
        #[structopt(long, about = "rebase all subsequent branches in this stack.")]
        all: bool,
        #[structopt(short, long)]
        onto: Option<String>,
        #[structopt(short, long)]
        strategy: Option<String>,
        #[structopt(short = "a", long = "abort")]
        rebase_abort: bool,
        #[structopt(short = "c", long = "continue")]
        rebase_continue: bool,
        #[structopt(long = "cleanup")]
        rebase_cleanup: bool,
    },
    #[structopt(about = "Manage issues")]
    Issue(IssueSubcommand),
    #[structopt(about = "open a tui terminal to view issues")]
    Terminal {},
    #[structopt(about = "Open a tui terminal for branches record info")]
    Branches {},
    #[structopt(name = "br", about = "List existing branches (with start info)")]
    Branch {},
    #[structopt(about = "Manage status/daily record info")]
    Status(StatusSubcommand),
    #[structopt(about = "Starts a pomodoro clock")]
    Pomodoro {
        #[structopt(short, long, default_value = "25")]
        duration_mins: u32,
    },
    #[structopt(about = "Deletes a branch (current one currently)", alias = "del")]
    Delete {
        #[structopt(about = "Branch to delete", short, long)]
        branch: Option<String>,
        #[structopt(about = "Branch to checkout", short, long)]
        dest: Option<String>,
    },
    #[structopt(about = "Shows the current diff for the branch")]
    Diff {},
    #[structopt(about = "delete closed branches")]
    Cleanup {
        #[structopt(short = "f", long = "force")]
        force: bool,
    },
    #[structopt(about = "rename current branch (and start branch)")]
    Rename {
        #[structopt(about = "new name")]
        new_name: String,
    },
    #[structopt(about = "sync current branch with remote branch (if more recent)")]
    Sync {
        #[structopt(short = "f", long = "force")]
        force: bool,
    },
    #[structopt(about = "migrate to new branch format")]
    Migrate {
        #[structopt(about = "prefix to use for new branches")]
        prefix: String,
        #[structopt(about = "separator for splitting branch name")]
        separator: String,
    },
    #[structopt(about = "dumps debug info")]
    Debug {},
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Commands for managing github issues")]
enum IssueSubcommand {
    #[structopt(about = "Create a new github issue")]
    Create {
        #[structopt(short, long)]
        title: String,
    },
    #[structopt(about = "List open github issues")]
    List {},
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Commands for managing the status file")]
enum StatusSubcommand {
    #[structopt(about = "Write a new entry for today.")]
    Write {
        #[structopt(short, long)]
        body: String,

        #[structopt(short, long)]
        todo: bool,
    },
    #[structopt(about = "List the existing tasks.")]
    List {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = GG::from_args();
    match opt.cmd {
        Cmd::New {
            feature,
            part,
            main,
        } => {
            if main {
                git::fetch_main();
                git::checkout_main();
            }
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
        }
        Cmd::Push { force, mut start } => {
            let cur = git::current_branch();
            if !start && git::get_branch_for_dir(git::CheckoutDir::Prev).is_some() {
                println!("You're trying to push a branch that has prev changes without specifying '-s', do you want to push the start branch as well?");
                start = confirm();
            }
            let mut branches = vec![cur.clone()];
            if start {
                branches.push(git::parse_branch(cur).start());
            }
            git::push(branches, force);
        }
        Cmd::Pr { use_start, is_draft } => {
            let branch = git::current_parsed_branch();
            git::push_one(branch.full(), true);
            let cfg = config::get_full_config();
            let github = GithubRepo::new(cfg).await;
            let base = match use_start {
                true => {
                    git::push_one(branch.start(), true);
                    Some(branch.start())
                }
                false => None,
            };
            github
                .create_pr(branch.full(), base, is_draft)
                .await
                .expect("error creating PR");
        }
        Cmd::Fetch {} => {
            git::fetch_main();
        }
        Cmd::Fixup {} => {
            git::fixup_main();
        }
        Cmd::Log { all } => {
            log(all).await;
        }
        Cmd::Land {} => {
            let cfg = config::get_full_config();
            let github = GithubRepo::new(cfg).await;
            let branch = git::current_parsed_branch();
            let pr = github
                .pr_for_branch(&branch.full())
                .await
                .expect("error getting PR")
                .unwrap();
            github
                .land_pr(branch.full())
                .await
                .expect("error landing PR");
            git::fetch_main();
            git::checkout_main();
            git::delete_branch_all(branch.full());
            git::delete_branch_all(branch.start());
            let selected_issue = config::get_selected_issue_number();
            if selected_issue > 0 {
                let issue = github.get_issue(selected_issue).await?;
                print!(
                    "Close issue '{}' github.com/{}/{}/issues/{}?\n[y/n]: ",
                    issue.title, github.org, github.repo, selected_issue
                );
                io::stdout().flush().unwrap();
                let res = std::io::stdin()
                    .bytes()
                    .next()
                    .and_then(|result| result.ok())
                    .unwrap() as char;
                if res == 'y' {
                    github.close_issue(selected_issue).await?;
                    config::update_selected_issue(0);
                }
            }
            status::write_status(format!("Landed: {}", pr.title), false);
        }
        Cmd::RebaseOld { interactive } => {
            git::rebase(interactive);
        }
        Cmd::Issue(issue) => {
            let cfg = config::get_full_config();
            let github = GithubRepo::new(cfg).await;
            match issue {
                IssueSubcommand::Create { title } => {
                    github
                        .create_issue(title.as_str(), "")
                        .await
                        .expect("error creating");
                }
                IssueSubcommand::List {} => {
                    github.list_issues().await.expect("error creating");
                }
            }
        }
        Cmd::Branches {} => {
            terminal::start_pr_terminal().await.unwrap();
        }
        Cmd::Branch {} => {
            let github = GithubRepo::new(get_full_config()).await;
            let branches = terminal::branches::load_branch_infos(&github).await;
            for branch in branches {
                println!(
                    "{} {} {}\t{}",
                    match branch.current {
                        true => color::bold(color::green("*")),
                        false => " ".to_string(),
                    },
                    match branch.has_start {
                        true => color::bold(color::white("w/s")),
                        false => "---".to_string(),
                    },
                    color::bold(branch.branch),
                    match branch.pr {
                        Some(pr) => format!(
                            "{} ({}) {} {}",
                            color::bold(color::white(pr.url)),
                            color::bold(match pr.closed {
                                true => color::red("Closed"),
                                false => color::green("Open"),
                            }),
                            match pr.review_decision {
                                Some(d) => {
                                    match d.as_str() {
                                        "APPROVED" => color::green('✔'),
                                        "REVIEW_REQUIRED" => color::yellow('∞'),
                                        "CHANGES_REQUESTED" => color::red('✖'),
                                        val => val.to_string(),
                                    }
                                }
                                None => color::yellow('∞'),
                            },
                            match pr.test_status.as_str() {
                                "SUCCESS" => color::green('✔'),
                                "PENDING" => color::yellow('∞'),
                                "FAILURE" => color::red('✖'),
                                "ERROR" => color::red('✖'),
                                "EXPECTED" => color::blue('?'),
                                val => val.to_string(),
                            },
                        ),
                        None => "".to_string(),
                    }
                )
            }
        }
        Cmd::Terminal {} => {
            terminal::start_terminal().await.unwrap();
        }
        Cmd::Status(cmd) => match cmd {
            StatusSubcommand::Write { body, todo } => {
                status::write_status(body, todo);
            }
            StatusSubcommand::List {} => {
                status::list_statuses();
            }
        },
        Cmd::Debug {} => {
            println!("trying change base");
            let github = GithubRepo::new(get_full_config()).await;
            github
                .change_base(
                    "wh/pr_updatebase_part-1.0".to_string(),
                    "wh/pr_starts_updatebase_part-1.0".to_string(),
                )
                .await
                .expect("error getting PRs");
        }
        Cmd::Init {} => {
            config::get_full_config();
        }
        Cmd::Pomodoro { duration_mins } => {
            pomodoro::run_pomodoro(duration_mins);
        }
        Cmd::Checkout {
            next,
            prev,
            part,
            start,
        } => {
            let dir = if let Some(part) = part {
                git::CheckoutDir::Part(part)
            } else if next {
                git::CheckoutDir::Next
            } else if prev {
                git::CheckoutDir::Prev
            } else if start {
                git::CheckoutDir::Start
            } else {
                git::CheckoutDir::Unknown
            };
            match git::get_branch_for_dir(dir) {
                Some(x) => git::checkout(&x),
                None => println!("No branch found!"),
            };
        }
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
            git::delete_branch_all(branch_to_delete.full());
            git::delete_branch_all(branch_to_delete.start());
        }
        Cmd::Rebase {
            all,
            onto,
            strategy,
            rebase_abort,
            rebase_continue,
            rebase_cleanup,
        } => {
            if rebase_cleanup {
                fixup_rebase();
                return Ok(());
            } else if rebase_abort {
                abort_rebase();
                return Ok(());
            } else if rebase_continue {
                continue_rebase();
            } else {
                start_rebase(onto, strategy.clone());
            }
            if all {
                rebase_all_children(strategy);
            }
        }
        Cmd::Diff {} => {
            diff(current_parsed_branch().start(), None);
        }
        Cmd::Cleanup { force } => {
            cleanup(force).await;
        }
        Cmd::Sync { force } => {
            sync(force);
        }
        Cmd::Migrate { prefix, separator } => {
            migrate(prefix.as_str(), separator.as_str());
        }
        Cmd::Rename { new_name } => {
            let cur = current_parsed_branch();
            let mut new = cur.clone();
            new.base = new_name;
            git::rename_branch(new.full().as_str(), cur.full().as_str());
            git::rename_branch(new.start().as_str(), cur.start().as_str());
        }
        Cmd::Setbase {} => {
            let cfg = get_full_config();
            let mainbr = cfg.saved.repo_main_branch.clone();
            let github = GithubRepo::new(cfg).await;
            let cur = current_branch();
            github
                .change_base(cur, mainbr)
                .await
                .expect("error getting PRs");
        }
    }
    Ok(())
}

fn migrate(prefix: &str, separator: &str) {
    let branches = git::all_parsed_managed_branches();
    for branch in branches {
        let mut new_branch = branch.clone();
        new_branch.prefix = Some(prefix.to_string());
        let new_full_branch = new_branch.full_with_split(separator);
        let new_start_branch = new_branch.start_with_split(separator);
        println!(
            "Rename {} to {} and {} to {}?",
            branch.full(),
            new_full_branch,
            branch.start(),
            new_start_branch
        );
        if !confirm() {
            continue;
        }
        git::rename_branch(new_full_branch.as_str(), branch.full().as_str());
        git::rename_branch(new_start_branch.as_str(), branch.start().as_str());
    }
    println!("Fixing configuration!");
    update_prefix_and_split(prefix, separator);
}

async fn cleanup(force: bool) {
    let cfg = config::get_full_config();
    let github = GithubRepo::new(cfg).await;
    let branches = git::all_branches();
    let mut br_map = HashSet::new();
    for branch in &branches {
        br_map.insert(branch.clone());
    }
    let prs = github
        .prs_for_branches(&br_map)
        .await
        .expect("error getting PRs");

    for pr in prs {
        if !pr.closed {
            continue;
        }
        if !force {
            println!(
                "Do you want to delete {}, {}, \"{}\"?",
                pr.branch, pr.url, pr.title
            );
            if !confirm() {
                continue;
            }
        }

        cleanup_closed_pr(&pr)
    }
}

fn cleanup_closed_pr(pr: &Pr) {
    let br = parse_branch(pr.branch.clone());
    println!("Deleting {} and {}", br.full(), br.start());
    delete_branch_all(br.full());
    delete_branch_all(br.start());
}

fn confirm() -> bool {
    println!("[y/n]: ");

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let line = line.trim_end_matches(x);

    line == "y"
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
