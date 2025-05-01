use crate::file;
use crate::{color, config, github::GithubRepo};
use octocrab::models::pulls::PullRequest;
use octocrab::models::IssueState;
use serde::Deserialize;
use std::str::from_utf8;
use std::{collections::HashSet, process::Command};

#[derive(Debug, Clone, Deserialize)]
pub struct Pr {
    pub closed: bool,
    pub title: String,
    #[serde(rename = "headRefName")]
    pub branch: String,
    pub url: String,
    #[serde(rename = "reviewDecision")]
    pub review_decision: Option<String>,
    pub mergeable: String,
}

impl GithubRepo {
    pub async fn create_pr(
        &self,
        full_branch: String,
        base: Option<String>,
        is_draft: bool,
    ) -> anyhow::Result<()> {
        // TODO Check if PR already exists.

        let cfg = config::get_full_config();
        let base = base.unwrap_or(cfg.saved.repo_main_branch);
        let (title, body) = self.get_title_and_body(base.clone()).await;

        let mut cmd = Command::new("gh");
        cmd.arg("pr")
            .arg("create")
            .arg("-H")
            .arg(full_branch)
            .arg("-B")
            .arg(base)
            .arg("-b")
            .arg(body)
            .arg("-t")
            .arg(title);

        if is_draft {
            cmd.arg("--draft");
        }
        let res = cmd.output().map_err(anyhow::Error::msg)?;

        let result = from_utf8(&res.stdout).expect("msg");
        let err = from_utf8(&res.stderr).expect("msg");
        println!("Out: {}", result.to_string());
        println!("Err: {}", err.to_string());

        Ok(())
    }

    async fn get_title_and_body(&self, base: String) -> (String, String) {
        let res = file::open_vim(self.get_template_for_pr(base).await);
        let (title, body) = res.split_once('\n').expect("wanted at least two lines");
        (title.to_string(), body.to_string())
    }

    async fn get_template_for_pr(&self, base: String) -> String {
        let cfg = config::get_full_config();
        let linked_issue = cfg.saved.linked_issue;
        let mut template = self.get_git_log_from_base_branch(base);

        match linked_issue {
            Some(0) => {}
            None => {}
            Some(x) => {
                let issue = self.get_issue(x).await.unwrap();
                template.push_str(
                    format!(
                        "\n\nResolves Issue: [{}](https://github.com/{}/{}/issues/{})",
                        issue.title, self.org, self.repo, x,
                    )
                    .as_str(),
                );
            }
        }
        template
    }

    fn get_git_log_from_base_branch(&self, core_branch: String) -> String {
        let out = match Command::new("git")
            .arg("log")
            .arg("--pretty=%s%n%+b")
            .arg(format!("origin/{}..HEAD", core_branch))
            .output()
        {
            Ok(output) => output,
            Err(_e) => panic!("error!"),
        };
        let x: &[_] = &[' ', '\t', '\n', '\r'];
        let result = from_utf8(&out.stdout).expect("msg").trim_end_matches(x);
        result.to_string()
    }

    pub async fn pr_status(&self, full_branch: String) -> anyhow::Result<()> {
        let pr = self.pr_for_branch(&full_branch).await?;
        self.print_pull(pr, full_branch);
        Ok(())
    }

    pub async fn pr_for_branch(&self, branch: &String) -> anyhow::Result<Option<PullRequest>> {
        let hub_head = format!("{}:{}", self.org, branch);
        let pulls = self
            .octo
            .pulls(self.org.clone(), self.repo.clone())
            .list()
            .head(hub_head)
            .per_page(1)
            .send()
            .await
            .map_err(|p| {
                println!("err: {:?}", p);
                anyhow::Error::msg(p)
            })?;

        // return Err(anyhow::Error::msg("tmp"));

        // let pulls = match pulls {
        //     Ok(p) => p,
        //     Err(e) => {
        //         let err = e.to_string();
        //         if err.contains("Not Found") {
        //             return Ok(Option::None)
        //         }
        //         return Err(anyhow::anyhow!(err));
        //     },
        // };

        if pulls.items.len() == 1 {
            return Ok(Option::Some(pulls.items.first().unwrap().clone()));
        }
        Ok(Option::None)
    }

    pub async fn prs_for_branches(&self, branches: &HashSet<String>) -> anyhow::Result<Vec<Pr>> {
        // self.prs_for_branches_octo(branches).await
        self.prs_for_branches_cmd(branches).await
    }

    pub async fn prs_for_branches_cmd(
        &self,
        branches: &HashSet<String>,
    ) -> anyhow::Result<Vec<Pr>> {
        let mut cmd = Command::new("gh");
        cmd.arg("pr")
            .arg("list")
            .arg("--author")
            .arg("@me")
            .arg("-s")
            .arg("all")
            .arg("-L")
            .arg("100")
            .arg("--json")
            .arg("closed,title,headRefName,url,reviewDecision,mergeable");
        let res = cmd.output().map_err(anyhow::Error::msg)?;
        let out = String::from_utf8(res.stdout).map_err(anyhow::Error::msg)?;
        let err = String::from_utf8(res.stderr).map_err(anyhow::Error::msg)?;
        if !err.is_empty() {
            println!("Err: {}", err);
        }
        let prs: Vec<Pr> = serde_json::from_str(&out).map_err(anyhow::Error::msg)?;
        Ok(prs)
    }

    pub async fn prs_for_branches_octo(
        &self,
        branches: &HashSet<String>,
    ) -> anyhow::Result<Vec<Pr>> {
        let query = format!(
            "
        {{
            search(first:100, query: \"is:pr author:{} org:{} repo:{}\", type: ISSUE) {{
              edges {{
                node {{
                  ... on PullRequest {{
                    closed
                    title
                    headRefName
                    url
                    state
                    reviewDecision
                    mergeable
                    autoMergeRequest {{
                        enabledAt
                    }}
                    commits(last:1) {{
                        nodes {{
                            commit {{
                                status {{
                                    state
                                }}
                            }}
                        }}
                    }}
                  }}
                }}
              }}
            }}
          }}
        ",
            self.current_user.clone(),
            self.org,
            self.repo
        );
        let pulls: serde_json::Value = self
            .octo
            .graphql(query.as_str())
            .await
            .map_err(anyhow::Error::msg)?;

        // HOLY CRAP!  GENERATED BY COPILOT (works pretty well though tbh).
        // Honestly, this should be a macro or something.
        let prs: Vec<Pr> = pulls
            .as_object()
            .unwrap()
            .get("data")
            .unwrap()
            .as_object()
            .unwrap()
            .get("search")
            .unwrap()
            .as_object()
            .unwrap()
            .get("edges")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| {
                let node = x
                    .as_object()
                    .unwrap()
                    .get("node")
                    .unwrap()
                    .as_object()
                    .unwrap();
                Pr {
                    // TODO
                    closed: node.get("closed").unwrap().as_bool().unwrap(),
                    title: node.get("title").unwrap().as_str().unwrap().to_string(),
                    branch: node
                        .get("headRefName")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    url: node.get("url").unwrap().as_str().unwrap().to_string(),
                    review_decision: node
                        .get("reviewDecision")
                        .unwrap()
                        .as_str()
                        .map(|f| f.to_string()),
                    mergeable: node.get("mergeable").unwrap().as_str().unwrap().to_string(),
                }
            })
            .filter(|x| branches.contains(x.branch.as_str()))
            .collect();
        Ok(prs)
    }

    fn print_pull(&self, pull: Option<PullRequest>, branch: String) {
        let (state, url, title) = match pull {
            Some(p) => (
                match p.state {
                    Some(IssueState::Closed) => color::red("Closed"),
                    Some(IssueState::Open) => color::green("Open"),
                    _ => color::red("Unknown"),
                },
                color::blue(p.html_url.unwrap()),
                p.title.unwrap_or("".to_string()),
            ),
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

    pub async fn land_pr(&self, full_branch: String) -> anyhow::Result<()> {
        let pr = self
            .pr_for_branch(&full_branch)
            .await?
            .expect("want be there");

        // TODO ADD TESTS CHECK

        let res = self
            .octo
            .pulls(self.org.clone(), self.repo.clone())
            .merge(pr.number)
            .method(octocrab::params::pulls::MergeMethod::Squash)
            .send()
            .await
            .map_err(anyhow::Error::msg)?;

        println!("Merge was {}", if res.merged { "good" } else { "bad" });
        if !res.merged {
            panic!("error merging time");
        }
        Ok(())
    }

    // Change the base of a branch
    pub async fn change_base(&self, full_branch: String, new_base: String) -> anyhow::Result<()> {
        let pr = self
            .pr_for_branch(&full_branch)
            .await?
            .expect("want be there");

        let query = format!(
            "
                mutation {{
                    updatePullRequest(input: {{baseRefName: \"{}\", pullRequestId:\"{}\"}}) {{
                        clientMutationId
                    }}
                }}
        ",
            new_base,
            pr.node_id.unwrap_or("".to_string()),
        );
        println!("{}", query);
        let _res: serde_json::Value = self
            .octo
            .graphql(query.as_str())
            .await
            .map_err(anyhow::Error::msg)?;
        Ok(())
    }
}
