#[path = "file.rs"] mod file;
use octocrab::Octocrab;
use std::process::Command;
use std::str::from_utf8;

pub async fn create_pr(full_branch: String) -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octo = Octocrab::builder().personal_token(token).build()?;

    let (title, body) = get_title_and_body(full_branch.clone());
    // TODO add real info
    octo.pulls("willhug", "gg")
        .create(title, full_branch, "main")
        .body(body)
        .send()
        .await?;

    println!("RAN THE PULL");

    Ok(())
}

fn get_title_and_body(branch: String) -> (String, String) {
    let res = file::open_vim(get_git_log_for_branch(branch));
    let (title, body) = res.split_once("\n").expect("wanted at least two lines");
    (title.to_string(), body.to_string())
}

fn get_git_log_for_branch(branch: String) -> String {
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
