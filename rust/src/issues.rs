use octocrab::Octocrab;

pub async fn create_issue(title: &str, body: &str) -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let res = octo.issues("willhug", "gg")
        .create(title)
        .body(body)
        .send()
        .await?;


    println!("Created Issue: {}", res.html_url);

    Ok(())
}

pub async fn list_issues() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let res = octo.issues("willhug", "gg")
        .list()
        .creator("willhug")
        .per_page(100)
        .send()
        .await?;


    for issue in res {
        println!("{} : {}", issue.html_url, issue.title);
    }

    Ok(())
}
