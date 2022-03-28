pub mod issue;
pub mod pr;

use crate::config::FullConfig;
use octocrab::Octocrab;

pub struct GithubRepo {
    pub org: String,
    pub repo: String,
    pub(crate) current_user: String,
    pub(crate) octo: Octocrab,
}

impl GithubRepo {
    pub async fn new(cfg: FullConfig) -> GithubRepo {
        let octo = Octocrab::builder()
            .personal_token(cfg.github_token)
            .build()
            .unwrap();
        let current_user = octo.current().user().await.unwrap().login;
        GithubRepo {
            org: cfg.saved.repo_org,
            repo: cfg.repo_name,
            current_user,
            octo,
        }
    }
}
