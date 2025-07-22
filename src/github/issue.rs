use crate::{
    color,
    config::{self, get_full_config},
    github::GithubRepo,
    record,
};
use octocrab::models::{issues::Issue, IssueState};

impl GithubRepo {
    pub async fn create_issue(&self, title: &str, body: &str) -> octocrab::Result<()> {
        let res = self
            .octo
            .issues(self.org.clone(), self.repo.clone())
            .create(title)
            .body(body)
            .send()
            .await?;

        println!(
            "Created Issue: {}",
            color::bold(color::green(res.html_url.to_string()))
        );

        Ok(())
    }

    pub async fn list_issues(&self) -> octocrab::Result<()> {
        let res = self.get_issues().await.unwrap();

        for issue in res {
            println!(
                "{} : {}",
                color::blue(issue.html_url.to_string()),
                color::bold(issue.title)
            );
        }

        Ok(())
    }

    pub async fn get_issue(&self, number: u64) -> octocrab::Result<Issue> {
        let res = self
            .octo
            .issues(self.org.clone(), self.repo.clone())
            .get(number)
            .await?;

        Ok(res)
    }

    pub async fn get_issues(&self) -> octocrab::Result<Vec<Issue>> {
        let res = self
            .octo
            .issues(self.org.clone(), self.repo.clone())
            .list()
            .creator(self.current_user.clone())
            .per_page(100)
            .send()
            .await?;

        let list: Vec<Issue> = res
            .into_iter()
            .filter(|x| x.pull_request.is_none())
            .collect();

        Ok(list)
    }

    pub async fn close_issue(&self, number: u64) -> octocrab::Result<()> {
        let issue = self.get_issue(number).await?;

        self.octo
            .issues(self.org.clone(), self.repo.clone())
            .update(number)
            .state(IssueState::Closed)
            .send()
            .await?;

        // Triggers TODO: Formalize this.
        record::write_status(format!("Closed: {}", issue.title), false);
        let cfg = get_full_config();
        if cfg.saved.linked_issue.is_some() && cfg.saved.linked_issue.unwrap() == issue.number {
            config::clear_selected_issue();
        }

        Ok(())
    }
}
