use crate::{color, config};
use crate::file;
use octocrab::Octocrab;
use octocrab::models::IssueState;
use octocrab::models::pulls::PullRequest;
use std::process::Command;
use std::str::from_utf8;

pub async fn create_pr(full_branch: String) -> anyhow::Result<()> {
    let existing_pr = pr_for_branch(full_branch.clone()).await?;
    if let Some(pr) = existing_pr {
        println!("PR Already exists! {}", pr.html_url);
        return Ok(())
    }
    let cfg = config::get_full_config();
    let octo = Octocrab::builder().personal_token(cfg.github_token).build().map_err(anyhow::Error::msg)?;

    let (title, body) = get_title_and_body();
    let res = octo.pulls(cfg.repo_org, cfg.repo_name)
        .create(title, full_branch, cfg.saved.repo_main_branch)
        .body(body)
        .send()
        .await
        .map_err(anyhow::Error::msg)?;


    println!("Created PR: {}", res.html_url);

    Ok(())
}

fn get_title_and_body() -> (String, String) {
    let res = file::open_vim(get_template_for_pr());
    let (title, body) = res.split_once("\n").expect("wanted at least two lines");
    (title.to_string(), body.to_string())
}

fn get_template_for_pr() -> String {
    let cfg = config::get_full_config();
    let mut template = get_git_log_from_base_branch(cfg.saved.repo_main_branch);

    match cfg.saved.linked_issue {
        Some(0) => {},
        None => {},
        Some(x) => {
            template.push_str(format!("\n\nResolves Issue: github.com/{}/{}/issues/{}", cfg.repo_org, cfg.repo_name, x).as_str());
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
    result.to_string()
}

pub async fn pr_statuses(full_branch: String) -> anyhow::Result<()> {
    let pr = pr_for_branch(full_branch.clone()).await?;
    print_pull(pr, full_branch);
    Ok(())
}

async fn pr_for_branch(branch: String) -> anyhow::Result<Option<PullRequest>> {
    let cfg = config::get_full_config();
    let octo = Octocrab::builder().personal_token(cfg.github_token).build().map_err(anyhow::Error::msg)?;

    let hub_head = format!("{}:{}", cfg.repo_org, branch);
    let pulls = octo.pulls(cfg.repo_org, cfg.repo_name)
        .list()
        .head(hub_head)
        .per_page(1)
        .send()
        .await
        .map_err(anyhow::Error::msg)?;

    if pulls.items.len() == 1 {
        return Ok(Option::Some(pulls.items.first().unwrap().clone()))
    }
    Ok(Option::None)
}

fn print_pull(pull: Option<PullRequest>, branch: String) {
    let (state, url, title) = match pull {
        Some(p) => (match p.state {
            IssueState::Closed => color::red("Closed"),
            IssueState::Open => color::green("Open"),
            _ => color::red("Unknown"),
        }, color::blue(p.html_url), p.title),
        None => (color::white("N/A"), color::white("N/A"), "".to_string()),
    };
    println!(
        "{}\t{}\t{}\t{}",
        color::blue(branch),
        color::bold(state),
        url,
        title,
    )
}

pub async fn land_pr(full_branch: String) -> anyhow::Result<()> {
    let pr = pr_for_branch(full_branch).await?.expect("want be there");

    let cfg = config::get_full_config();
    let octo = Octocrab::builder().personal_token(cfg.github_token).build().map_err(anyhow::Error::msg)?;

    // TODO ADD TESTS CHECK

    let res = octo.pulls(cfg.repo_org, cfg.repo_name)
        .merge(pr.number)
        .method(octocrab::params::pulls::MergeMethod::Rebase)
        .send()
        .await
        .map_err(anyhow::Error::msg)?;

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