pub mod pr;
pub mod issue;

use octocrab::Octocrab;
use crate::config::FullConfig;

pub struct GithubRepo {
    pub org: String,
    pub repo: String,
    pub(crate) current_user: String,
    pub(crate) octo: Octocrab,
}

impl GithubRepo {
    pub async fn new(cfg: FullConfig) -> GithubRepo {
        GithubRepo {
            org: cfg.saved.repo_org,
            repo: cfg.repo_name,
            current_user: cfg.current_github_user,
            octo: Octocrab::builder().personal_token(cfg.github_token).build().unwrap(),
        }
    }
}