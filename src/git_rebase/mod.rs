use std::{fs, path::Path, process::Command, str::from_utf8, thread, time::Duration};

use sha2::{Digest, Sha256};

use crate::git::{
    assert_branch_exists, checkout, cherry_abort, cherry_continue, cherry_pick, current_branch,
    current_parsed_branch, delete_branch_all, delete_branch_local, force_branch_to_be,
    get_branch_for_dir, get_children_branches, get_commit_hash, get_repo_root, new, parse_branch,
    reset, ParsedBranch,
};

pub(crate) fn rebase_all_children(strategy: Option<String>) {
    let cur = current_parsed_branch();
    let children = get_children_branches(&cur);

    for child in children {
        checkout(&child.full());
        start_rebase(None, strategy.clone());
    }
}

pub(crate) fn start_rebase(onto: Option<String>, strategy: Option<String>) {
    check_and_clean_lock();

    if is_rebase_in_progress() {
        eprintln!("A previous rebase appears to be in progress (leftover _tmp_- branches found).");
        eprintln!("Run `gg rebase --continue` to resume or `gg rebase --abort` to cancel.");
        std::process::exit(1);
    }

    setup_hooks_path();
    let cur = TmpBranchWrapper::new(current_parsed_branch());
    let onto = onto.unwrap_or_else(|| {
        get_branch_for_dir(crate::git::CheckoutDir::Prev)
            .expect("No previous branch to rebase onto")
    });
    assert_branch_exists(onto.clone());
    rebase_onto(cur, onto, strategy);
}

pub(crate) fn abort_rebase() {
    let br = TmpBranchWrapper::new_from_tmp_branch(current_branch());

    cherry_abort();

    checkout(&br.inner.full());

    delete_branch_all(br.tmp_branch_name());
    delete_branch_all(br.tmp_start_branch_name());
    finish_up_hooks_path();
}

pub(crate) fn continue_rebase() {
    let br = TmpBranchWrapper::new_from_tmp_branch(current_branch());

    if !cherry_continue() {
        eprintln!("Cherry-pick --continue failed. Resolve conflicts and run `gg rebase --continue` again.");
        std::process::exit(1);
    }

    finish_rebase(br);
}

fn rebase_onto(branch_to_rebase: TmpBranchWrapper, onto: String, strategy: Option<String>) {
    if branches_are_equivalent(branch_to_rebase.inner.start(), branch_to_rebase.inner.full()) {
        println!("There are no commits to rebase, fast forwarding the branches");
        force_branch_to_be(&branch_to_rebase.inner.full(), &onto);
        force_branch_to_be(&branch_to_rebase.inner.start(), &onto);
        finish_up_hooks_path();
        return;
    }
    // TODO CREATE WORKTREE TO ISOLATE BRANCH REBASE
    println!(
        "Rebasing {} onto {} via cherry-picks",
        branch_to_rebase.inner.full(),
        onto
    );
    checkout(&onto);
    new(branch_to_rebase.tmp_start_branch_name().as_str());
    new(branch_to_rebase.tmp_branch_name().as_str());
    println!("Cherry-picking changes onto branch {}", onto);
    let success = cherry_pick(
        branch_to_rebase.inner.start(),
        branch_to_rebase.inner.full(),
        strategy,
    );
    if !success {
        eprintln!("Cherry-pick failed (likely a conflict).");
        eprintln!(
            "Resolve conflicts, then run `gg rebase --continue` or `gg rebase --abort` to cancel."
        );
        return;
    }
    finish_rebase(branch_to_rebase);
}

pub(crate) fn fixup_rebase() {
    let br = TmpBranchWrapper::new_from_tmp_branch(current_branch());

    finish_rebase(br);
}

fn finish_rebase(br: TmpBranchWrapper) {
    checkout(&br.inner.start());
    reset(br.tmp_start_branch_name(), true);

    checkout(&br.inner.full());
    reset(br.tmp_branch_name(), true);

    // Use .ok() so we don't panic if branches are already gone
    delete_branch_local(&br.tmp_branch_name());
    delete_branch_local(&br.tmp_start_branch_name());
    finish_up_hooks_path();
}

/// Check for stale .git/index.lock files and clean them up if no git process is running.
fn check_and_clean_lock() {
    let root = get_repo_root();
    let lock_path = Path::new(&root).join(".git").join("index.lock");

    if !lock_path.exists() {
        return;
    }

    // Check if any git process is running
    if is_git_process_running() {
        eprintln!(
            "Warning: {} exists and a git process is running. Waiting briefly...",
            lock_path.display()
        );
        thread::sleep(Duration::from_millis(500));
        // Retry once
        if lock_path.exists() && is_git_process_running() {
            eprintln!("Lock file still present with active git process. Please wait for it to finish.");
            std::process::exit(1);
        }
    }

    if lock_path.exists() {
        eprintln!(
            "Warning: Removing stale lock file at {}",
            lock_path.display()
        );
        if let Err(e) = fs::remove_file(&lock_path) {
            eprintln!("Failed to remove lock file: {}", e);
            std::process::exit(1);
        }
    }
}

/// Check if any git process is currently running.
fn is_git_process_running() -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg("git")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a previous rebase left behind _tmp_- branches.
fn is_rebase_in_progress() -> bool {
    let out = Command::new("git")
        .arg("branch")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    out.lines().any(|line| line.trim().starts_with(TMP_PREFIX))
}

// Hooks break the rebase, so we need to delete before the rebase and set it up again after
fn setup_hooks_path() {
    store_hooks_path();
    set_hooks_path(DEVNULL);
}

fn finish_up_hooks_path() {
    let stored = get_stored_hooks_path();
    set_hooks_path(stored.as_str());
    let hooks_file = get_tmp_hooks_file();
    if Path::new(&hooks_file).exists() {
        fs::remove_file(&hooks_file).ok();
    }
}

fn store_hooks_path() {
    let path = get_hooks_path();
    let hooks_file = get_tmp_hooks_file();
    fs::write(&hooks_file, path).expect("failed to write hooks path to temp file");
}

fn get_stored_hooks_path() -> String {
    let hooks_file = get_tmp_hooks_file();
    fs::read_to_string(&hooks_file).unwrap_or_default()
}

fn get_hooks_path() -> String {
    let out = Command::new("git")
        .arg("config")
        .arg("core.hooksPath")
        .output()
        .expect("failed to get hooks path");

    let x: &[_] = &[' ', '\t', '\n', '\r', '*'];
    from_utf8(&out.stdout)
        .expect("failed to parse hooks path as UTF-8")
        .trim_end_matches(x)
        .to_string()
}

fn set_hooks_path(path: &str) {
    if path.is_empty() {
        Command::new("git")
            .arg("config")
            .arg("--unset")
            .arg("core.hooksPath")
            .output()
            .expect("failed to unset hooks path");
    } else {
        Command::new("git")
            .arg("config")
            .arg("core.hooksPath")
            .arg(path)
            .output()
            .expect("failed to set hooks path");
    }
}

/// Get a repo-specific temp file path for storing hooks configuration.
/// Uses a hash of the repo root to avoid collisions between repos.
fn get_tmp_hooks_file() -> String {
    let root = get_repo_root();
    let mut hasher = Sha256::new();
    hasher.update(root.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    format!("/tmp/gg_hooks_{}", &hash[..12])
}

const DEVNULL: &str = "/dev/null";

const TMP_PREFIX: &str = "_tmp_-";

struct TmpBranchWrapper {
    inner: ParsedBranch,
}

impl TmpBranchWrapper {
    fn new_from_tmp_branch(branch: String) -> TmpBranchWrapper {
        let orig_br = branch_from_tmp(branch);
        TmpBranchWrapper::new(parse_branch(orig_br))
    }

    fn new(p: ParsedBranch) -> TmpBranchWrapper {
        TmpBranchWrapper { inner: p }
    }

    fn tmp_branch_name(&self) -> String {
        tmp_for_branch_name(self.inner.full())
    }

    fn tmp_start_branch_name(&self) -> String {
        tmp_for_branch_name(self.inner.start())
    }
}

fn tmp_for_branch_name(branch: String) -> String {
    format!("{}{}", TMP_PREFIX, branch)
}

fn branch_from_tmp(branch: String) -> String {
    if let Some(suffix) = branch.strip_prefix(TMP_PREFIX) {
        return suffix.to_string();
    }
    panic!(
        "Expected to be on a tmp rebase branch (starting with '{}'), but current branch is '{}'",
        TMP_PREFIX, branch
    );
}

fn branches_are_equivalent(br1: String, br2: String) -> bool {
    get_commit_hash(br1) == get_commit_hash(br2)
}
