use std::str::from_utf8;

use std::process::Command;

use config::get_saved_config;
use crate::{color, config};

pub(crate) fn new(branch: &str) {
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

pub(crate) fn push(full_branch: &String, force: bool) {
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
    let current_branch = current_parsed_branch();
    Command::new("git")
            .arg("rebase")
            .arg("-i")
            .arg(current_branch.start())
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
    checkout(&format!("origin/{}", cfg.repo_main_branch));
}

pub(crate) fn checkout(branch: &String) {
    Command::new("git")
        .arg("checkout")
        .arg(branch)
        .output()
        .expect("failed to checkout main");
}

pub(crate) fn delete_branch(branch: String) {
    // TODO check for branch existance before deleting.
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


#[derive(Clone, PartialEq)]
pub(crate) enum CheckoutDir {
    Next,
    Prev,
    Unknown,
}

pub(crate) fn get_branch_for_dir(dir: CheckoutDir) -> Option<String> {
    let parsed_branch = current_parsed_branch();
    let branches = get_sorted_matching_branches(&parsed_branch);
    let location = branches.iter().position(|x| x.partx100 == parsed_branch.partx100)?;

    match dir {
        CheckoutDir::Next => {
            if branches.len() <= location + 1  {
                None
            } else {
                Some(branches[location + 1].full())
            }
        },
        CheckoutDir::Prev => {
            if location < 1 {
                None
            } else {
                Some(branches[location-1].full())
            }
        },
        CheckoutDir::Unknown => None,
    }
}

pub(crate) fn get_sorted_matching_branches(want: &ParsedBranch) -> Vec<ParsedBranch> {
    all_branches().into_iter().map(|b| {
        parse_branch(b)
    }).filter(|b| {
        b.base == want.base
    }).collect()
}

pub(crate) struct ParsedBranch {
    pub(crate) prefix: Option<String>,
    pub(crate) base: String,
    pub(crate) partx100: Option<u32>,
}

impl ParsedBranch {
    pub(crate) fn full(&self) -> String{
        let parts: Vec<String> = vec![
            self.prefix.clone(),
            Some(self.base.clone()),
            self.partx100.map(|p| format!("part-{:.1}", (p as f32)/100.0)),
        ].into_iter()
            .flatten()
            .collect();
        parts.join("/")
    }

    pub(crate) fn start(&self) -> String{
        let parts: Vec<String> = vec![
            self.prefix.clone(),
            Some("starts".to_string()),
            Some(self.base.clone()),
            self.partx100.map(|p| format!("part-{:.1}", (p as f32)/100.0)),
        ].into_iter()
            .flatten()
            .collect();
        parts.join("/")
    }
}

pub(crate) fn current_parsed_branch() -> ParsedBranch {
    parse_branch(current_branch())
}

pub(crate) fn parse_branch(orig_branch: String) -> ParsedBranch {
    let prefix = get_saved_config().branch_prefix;

    let mut found_prefix = None;
    let branch = match orig_branch.split_once(format!("{}/", prefix).as_str()) {
        Some(res) => {
            found_prefix = Some(prefix);
            res.1.to_string()
        }
        None => orig_branch.clone(),
    };

    match branch.split_once("/part-") {
        Some(res) => {
            ParsedBranch {
                prefix: found_prefix,
                base: res.0.to_string(),
                partx100: parse_partx100(res.1),
            }
        },
        None => {
            ParsedBranch {
                prefix: found_prefix,
                base: branch.clone(),
                partx100: None,
            }
        },
    }
}

pub(crate) fn parse_partx100(part: &str) -> Option<u32> {
    let fpart: f32 =  match part.parse::<f32>() {
        Ok(p) => p * 100.0,
        Err(_) => return None,
    };
    Some(fpart as u32)
}
