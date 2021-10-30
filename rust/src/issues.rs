use crate::{color, config, status};
use octocrab::{Octocrab, models::{IssueState, issues::Issue}};

pub async fn create_issue(title: &str, body: &str) -> octocrab::Result<()> {
    let cfg = config::get_full_config();
    let octo = Octocrab::builder().personal_token(cfg.github_token).build()?;

    let res = octo.issues(cfg.repo_org, cfg.repo_name)
        .create(title)
        .body(body)
        .send()
        .await?;


    println!("Created Issue: {}", color::bold(color::green(res.html_url.to_string())));

    Ok(())
}

pub async fn list_issues() -> octocrab::Result<()> {
    let res = get_issues().await.unwrap();

    for issue in res {
        println!("{} : {}", color::blue(issue.html_url.to_string()), color::bold(issue.title));
    }

    Ok(())
}

pub async fn get_issue(number: i64) -> octocrab::Result<Issue> {
    let cfg = config::get_full_config();
    let octo = Octocrab::builder().personal_token(cfg.github_token).build()?;

    let res = octo.issues(cfg.repo_org, cfg.repo_name)
        .get(number as u64)
        .await?;

    Ok(res)
}

pub async fn get_issues() -> octocrab::Result<Vec<Issue>> {
    let cfg = config::get_full_config();
    let octo = Octocrab::builder().personal_token(cfg.github_token).build()?;

    let res = octo.issues(cfg.repo_org, cfg.repo_name)
        .list()
        .creator(cfg.current_github_user)
        .per_page(100)
        .send()
        .await?;

    let list: Vec<Issue> = res.into_iter().collect();

    Ok(list)
}

pub async fn close_issue(number: i64) -> octocrab::Result<()> {
    let cfg = config::get_full_config();
    let octo = Octocrab::builder().personal_token(cfg.github_token).build()?;

    let issue = get_issue(number).await?;

    octo.issues(cfg.repo_org, cfg.repo_name)
        .update(number as u64)
        .state(IssueState::Closed)
        .send()
        .await?;

    // Triggers
    status::write_status(format!("Closed: {}", issue.title), false);
    if cfg.saved.linked_issue.is_some() && cfg.saved.linked_issue.unwrap() == issue.number {
        config::clear_selected_issue();
    }

    Ok(())
}