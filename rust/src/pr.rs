#[path = "file.rs"] mod file;
use octocrab::Octocrab;
use octocrab::models::IssueState;
use octocrab::models::pulls::PullRequest;
use std::process::Command;
use std::str::from_utf8;

pub async fn create_pr(full_branch: String) -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let (title, body) = get_title_and_body(full_branch.clone());
    let res = octo.pulls("willhug", "gg")
        .create(title, full_branch, "main")
        .body(body)
        .send()
        .await?;


    println!("Created PR: {}", res.html_url);

    Ok(())
}

fn get_title_and_body(branch: String) -> (String, String) {
    let res = file::open_vim(get_git_log_for_branch(branch));
    let (title, body) = res.split_once("\n").expect("wanted at least two lines");
    (title.to_string(), body.to_string())
}

fn get_git_log_for_branch(_branch: String) -> String {
    let out = match Command::new("git")
            .arg("log")
            .arg("--pretty=\"%s%+b\"")
            .arg("origin/main..HEAD")
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
        IssueState::Closed => "Closed",
        IssueState::Open => "Open",
        _ => "Unknown",
    };
    println!("{}: {}", state, pull.title)
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
    Ok(())
}