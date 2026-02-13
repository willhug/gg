pub mod issue;
pub mod pr;

use crate::config::FullConfig;

#[cfg(feature = "octo")]
use octocrab::Octocrab;

pub struct GithubRepo {
    pub org: String,
    pub repo: String,
    pub(crate) current_user: String,
    #[cfg(feature = "octo")]
    pub(crate) octo: Octocrab,
}

impl GithubRepo {
    #[cfg(feature = "octo")]
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

    #[cfg(not(feature = "octo"))]
    pub async fn new(cfg: FullConfig) -> GithubRepo {
        let current_user = std::process::Command::new("gh")
            .arg("api")
            .arg("user")
            .arg("--jq")
            .arg(".login")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());
        GithubRepo {
            org: cfg.saved.repo_org,
            repo: cfg.repo_name,
            current_user,
        }
    }
}
