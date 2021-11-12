use std::str::from_utf8;

use std::process::Command;

use crate::{color, config};

pub(crate) fn new(chosen_name: &str) {
    let branch: String = ["wh", chosen_name].join("/");
    Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg(branch)
            .output()
            .expect("failed to create branch");
}

pub(crate) fn all_branches() -> Vec<String> {
    let cfg = config::get_full_config();
    let out = match Command::new("git")
            .arg("branch")
            .output() {
                Ok(output) => output,
                Err(_e) => panic!("error!")
    };
    let x: &[_] = &[' ', '\t', '\n', '\r', '*'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    let mut branches = vec![];
    for line in result.split('\n') {
        let trimmed_line = line.trim_matches(x);
        if trimmed_line.starts_with(cfg.saved.branch_prefix.as_str()) {
            branches.push(trimmed_line.to_string());
        }
    }
    branches
}

pub(crate) fn current_branch() -> String {
    let out = match Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output() {
                Ok(output) => output,
                Err(_e) => panic!("error!")
    };
    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    result.to_string()
}

pub(crate) fn push(full_branch: String, force: bool) {
    let mut command = Command::new("git");
    let c = command.arg("push");

    if force {
        c.arg("-f");
    }

    let res = c.arg("origin")
     .arg(full_branch)
     .status()
     .expect("did not get successful response.");

     if res.success() {
         println!("{}", color::bold(color::green("Success!")));
     } else {
         println!("{}", color::bold(color::red("Error pushing! (try '-f')")))
     }
}

pub(crate) fn fetch_main() {
    let cfg = config::get_saved_config();
    Command::new("git")
            .arg("fetch")
            .arg("-p")
            .arg("origin")
            .arg(cfg.repo_main_branch)
            .output()
            .expect("failed to fetch main branch");
}

pub(crate) fn fixup_main() {
    // TODO USE START/END BRANCHES
    let cfg = config::get_full_config();
    Command::new("git")
            .arg("rebase")
            .arg("-i")
            .arg(format!("origin/{}",cfg.saved.repo_main_branch))
            .status()
            .expect("failed to fixup main branch");
}

pub(crate) fn rebase(interactive: bool) {
    let cfg = config::get_saved_config();
    let mut com = Command::new("git");
    let c = com.arg("rebase");
    if interactive {
        c.arg("-i");
    }

    c.arg(format!("origin/{}", cfg.repo_main_branch))
            .output()
            .expect("failed to rebase");
}

pub(crate) fn checkout_main() {
    let cfg = config::get_saved_config();
    Command::new("git")
        .arg("checkout")
        .arg(format!("origin/{}", cfg.repo_main_branch))
        .output()
        .expect("failed to checkout main");
}

pub(crate) fn delete_branch(branch: String) {
    Command::new("git")
        .arg("push")
        .arg("origin")
        .arg("-d")
        .arg(branch.clone())
        .status()
        .ok();
    Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg(branch.clone())
        .status()
        .expect("failed to delete branch");
    Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg("-r")
        .arg(branch)
        .status()
        .expect("failed to delete branch");
}
