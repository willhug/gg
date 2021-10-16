use crate::{color, config};
use crate::file;
use octocrab::Octocrab;
use octocrab::models::IssueState;
use octocrab::models::pulls::PullRequest;
use std::process::Command;
use std::str::from_utf8;

pub async fn create_pr(full_branch: String) -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let (title, body) = get_title_and_body();
    let res = octo.pulls("willhug", "gg")
        .create(title, full_branch, "main")
        .body(body)
        .send()
        .await?;


    println!("Created PR: {}", res.html_url);

    Ok(())
}

fn get_title_and_body() -> (String, String) {
    let res = file::open_vim(get_template_for_pr());
    let (title, body) = res.split_once("\n").expect("wanted at least two lines");
    (title.to_string(), body.to_string())
}

fn get_template_for_pr() -> String {
    let config = config::get_config();
    let mut template = get_git_log_from_base_branch(config.repo_main_branch);

    match config.linked_issue {
        Some(0) => {},
        None => {},
        Some(x) => {
            template.push_str(format!("\n\nResolves Issue: github.com/willhug/gg/issues/{}", x).as_str());
        }
    }
    template
}

fn get_git_log_from_base_branch(core_branch: String) -> String {
    let out = match Command::new("git")
            .arg("log")
            .arg("--pretty=%s%+b")
            .arg(format!("origin/{}..HEAD", core_branch))
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

pub async fn pr_statuses(full_branch: String) -> octocrab::Result<()> {
    let pr = pr_for_branch(full_branch).await?;
    print_pull(pr);
    Ok(())
}

async fn pr_for_branch(branch: String) -> octocrab::Result<PullRequest> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let hub_head = format!("willhug:{}", branch);
    let pulls = octo.pulls("willhug", "gg")
        .list()
        .head(hub_head)
        .per_page(1)
        .send()
        .await?;

    if pulls.items.len() == 1 {
        return Ok(pulls.items.first().unwrap().clone())
    }
    panic!("no errors!")
}

fn print_pull(pull: PullRequest) {
    let state = match pull.state {
        IssueState::Closed => color::red("Closed"),
        IssueState::Open => color::green("Open"),
        _ => color::red("Unknown"),
    };
    println!("{}: {} {}", color::bold(state), color::blue(pull.html_url), pull.title)
}

pub async fn land_pr(full_branch: String) -> octocrab::Result<()> {
    let pr = pr_for_branch(full_branch).await?;

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    // TODO ADD TESTS CHECK

    let res = octo.pulls("willhug", "gg")
        .merge(pr.number)
        .method(octocrab::params::pulls::MergeMethod::Rebase)
        .send()
        .await?;

    println!("Merge was {}",
        if res.merged {
            "good"
        } else {
            "bad"
        }
    );
    if !res.merged {
        panic!("error merging time");
    }
    Ok(())
}