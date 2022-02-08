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
        branches.push(trimmed_line.to_string());
    }
    branches
}


pub(crate) fn all_managed_branches() -> Vec<String> {
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
    fetch_branch(cfg.repo_main_branch.as_str());
}

pub(crate) fn fetch_branch(br: &str) {
    Command::new("git")
            .arg("fetch")
            .arg("-p")
            .arg("origin")
            .arg(br)
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

    assert!(
        c.arg(format!("origin/{}", cfg.repo_main_branch))
            .output()
            .expect("failed to rebase")
            .status.success(),
    );
}

pub(crate) fn checkout_main() {
    let cfg = config::get_saved_config();
    checkout(&format!("origin/{}", cfg.repo_main_branch));
}

pub(crate) fn checkout(branch: &String) {
    assert!(
        Command::new("git")
            .arg("checkout")
            .arg(branch)
            .output()
            .expect("failed to checkout main")
            .status.success()
    );
}

pub(crate) fn reset(branch: String, hard: bool) {
    let mut c = Command::new("git");
    c.arg("reset");
    if hard {
        c.arg("--hard");
    }
    c.arg(branch)
        .output()
        .expect("failed to reset");
}

pub(crate) fn force_branch_to_be(branch: &str, start_point: &str) {
    Command::new("git")
        .arg("branch")
        .arg("-f")
        .arg(branch)
        .arg(start_point)
        .status()
        .expect("failed to delete branch");
}

pub(crate) fn delete_branch_all(branch: String) {
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

pub(crate) fn delete_branch_local(branch: &str) {
    Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg(branch)
        .status()
        .expect("failed to delete branch");
}


#[derive(Clone, PartialEq)]
pub(crate) enum CheckoutDir {
    Next,
    Prev,
    Start,
    Part(f32),
    Unknown,
}

pub(crate) fn get_branch_for_dir(dir: CheckoutDir) -> Option<String> {
    let parsed_branch = current_parsed_branch();
    let branches = get_sorted_matching_branches(&parsed_branch.base);
    let location = branches.iter().position(|x| x.partx100 == parsed_branch.partx100)?;

    if branches.is_empty() {
        return None
    }
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
        CheckoutDir::Start => {
            Some(branches[0].full())
        },
        CheckoutDir::Part(part) => {
            let partx100 = partfloat_to_partx100(part);
            branches.iter()
            .find(|x| x.partx100.is_some() && x.partx100.unwrap() == partx100)
            .map(|br| br.full())
        }
        CheckoutDir::Unknown => None,
    }
}

pub(crate) fn get_children_branches(branch: &ParsedBranch) -> Vec<ParsedBranch> {
    let branches = get_sorted_matching_branches(&branch.base);
    branches.into_iter().filter(|x| x.partx100 > branch.partx100).collect()
}

pub(crate) fn get_sorted_matching_branches(base: &str) -> Vec<ParsedBranch> {
    let mut v: Vec<ParsedBranch> = all_managed_branches().into_iter().map(|b| {
        parse_branch(b)
    }).filter(|b| {
        b.base == base
    }).collect();
    v.sort_by(|a, b| a.partx100.cmp(&b.partx100) );
    v
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedBranch {
    pub(crate) prefix: Option<String>,
    pub(crate) base: String,
    pub(crate) partx100: Option<u32>,
}

impl ParsedBranch {
    pub(crate) fn full(&self) -> String {
        let parts: Vec<String> = vec![
            self.prefix.clone(),
            Some(self.base.clone()),
            self.partx100.map(|p| format!("part-{:.1}", (p as f32)/100.0)),
        ].into_iter()
            .flatten()
            .collect();
        parts.join(get_saved_config().branch_split.as_str())
    }

    pub(crate) fn start(&self) -> String {
        let parts: Vec<String> = vec![
            self.prefix.clone(),
            Some("starts".to_string()),
            Some(self.base.clone()),
            self.partx100.map(|p| format!("part-{:.1}", (p as f32)/100.0)),
        ].into_iter()
            .flatten()
            .collect();
        parts.join(get_saved_config().branch_split.as_str())
    }

    pub(crate) fn remote_full(&self) -> String {
        let parts: Vec<String> = vec![
            "origin".to_string(),
            self.full(),
        ].into_iter().collect();
        parts.join("/")
    }

    pub(crate) fn remote_start(&self) -> String {
        let parts: Vec<String> = vec![
            "origin".to_string(),
            self.start(),
        ].into_iter().collect();
        parts.join("/")
    }
}

pub(crate) fn is_start_branch(branch: &String) -> bool {
    let cfg = get_saved_config();
    branch.starts_with(format!("{}{}{}", cfg.branch_prefix, cfg.branch_split, "starts").as_str())
}

pub(crate) fn current_parsed_branch() -> ParsedBranch {
    parse_branch(current_branch())
}

pub(crate) fn parse_branch(orig_branch: String) -> ParsedBranch {
    let cfg = get_saved_config();
    let prefix = cfg.branch_prefix;

    let mut found_prefix = None;
    let branch = match orig_branch.split_once(format!("{}{}", prefix, cfg.branch_split).as_str()) {
        Some(res) => {
            found_prefix = Some(prefix);
            res.1.to_string()
        }
        None => orig_branch.clone(),
    };

    match branch.split_once(format!("{}part-", cfg.branch_split).as_str()) {
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
    match part.parse::<f32>() {
        Ok(p) => Some(partfloat_to_partx100(p)),
        Err(_) => None,
    }
}

fn partfloat_to_partx100(part: f32) -> u32 {
    (part * 100.0) as u32
}

pub(crate) fn cherry_pick(start_ref: String, end_ref: String, strategy: Option<String>) {
    let mut c = Command::new("git");

    c.arg("cherry-pick")
        .arg("-v")
        .arg(format!("{}..{}", start_ref, end_ref));

    if let Some(strategy) = strategy {
        c.arg("--strategy-option").arg(strategy);
    }

    assert!(c.status().expect("Failed to cherry-pick").success());

}

pub(crate) fn cherry_abort() {
    assert!(
        Command::new("git")
            .arg("cherry-pick")
            .arg("--abort")
            .status()
            .expect("Failed to cherry-pick")
            .success(),
    );
}

pub(crate) fn cherry_continue() {
    assert!(
        Command::new("git")
            .arg("cherry-pick")
            .arg("--continue")
            .status()
            .expect("Failed to cherry-pick")
            .success()
    );
}

pub(crate) fn get_commit_hash(branch: String) -> String {
    let out = Command::new("git")
            .arg("rev-parse")
            .arg(branch.clone())
            .output()
            .expect("error getting hash");
    if !out.status.success() {
        panic!("error getting hash {}", branch);
    }
    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    result.to_string()
}

pub(crate) fn assert_branch_exists(branch: String) {
    let _ = get_commit_hash(branch);
}

pub(crate) fn diff(start_ref: String, end_ref: Option<String>) {
    let mut c = Command::new("git");
    c.arg("diff");
    c.arg(match end_ref {
        Some(end_ref) => format!("{}..{}", start_ref, end_ref),
        None => start_ref,
    });
    c.status().expect("failed to fixup main branch");
}


pub(crate) fn sync(force: bool) {
    let br = current_parsed_branch();
    fetch_branch(br.full().as_str());
    fetch_branch(br.start().as_str());

    if get_create_timestamp(br.full().as_str()) >= get_create_timestamp(br.remote_full().as_str()) && !force {
        println!("{} is newer than {}", br.full(), br.remote_full());
        println!("use --force to override");
        return
    }

    // Force the start & end to be remote branches
    reset(br.remote_full(), true);
    force_branch_to_be(br.start().as_str(), br.remote_start().as_str());
}

fn get_create_timestamp(br: &str) -> i64 {
    let out = Command::new("git")
            .arg("show")
            .arg("--no-patch")
            .arg("--no-notes")
            .arg("--pretty=format:%ct")
            .arg(br)
            .output()
            .expect("error getting hash");
    if !out.status.success() {
        panic!("error getting timestamp {}", br);
    }
    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    println!("{}", result);
    result.parse::<i64>().expect("failed to parse timestamp")
}