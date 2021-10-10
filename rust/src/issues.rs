use crate::color;
use octocrab::{Octocrab, models::issues::Issue};

pub async fn create_issue(title: &str, body: &str) -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let res = octo.issues("willhug", "gg")
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

pub async fn get_issues() -> octocrab::Result<Vec<Issue>> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let res = octo.issues("willhug", "gg")
        .list()
        .creator("willhug")
        .per_page(100)
        .send()
        .await?;

    let list: Vec<Issue> = res.into_iter().collect();

    Ok(list)
}