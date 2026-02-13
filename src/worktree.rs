use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorktreeEntry {
    pub name: String,
    pub path: String,
    pub branch: Option<String>,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorktreeState {
    pub worktrees: Vec<WorktreeEntry>,
    pub pool_size: u32,
}

impl WorktreeState {
    fn new() -> Self {
        WorktreeState {
            worktrees: vec![],
            pool_size: 0,
        }
    }
}

fn get_worktree_dir() -> PathBuf {
    let root = git::get_repo_root();
    let root_path = Path::new(&root);
    let repo_name = root_path
        .file_name()
        .expect("repo root should have a directory name")
        .to_string_lossy();
    let parent = root_path
        .parent()
        .expect("repo root should have a parent directory");
    parent.join(format!("{}_worktrees", repo_name))
}

fn get_state_file() -> PathBuf {
    let root = git::get_repo_root();
    Path::new(&root).join(".git").join("GG_WORKTREES")
}

fn load_state() -> WorktreeState {
    let path = get_state_file();
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| WorktreeState::new()),
        Err(_) => WorktreeState::new(),
    }
}

fn save_state(state: &WorktreeState) {
    let path = get_state_file();
    let json = serde_json::to_string_pretty(state).expect("failed to serialize worktree state");
    fs::write(&path, json).expect("failed to write worktree state file");
}

/// Create a new worktree with the given name.
pub(crate) fn create_worktree(name: &str) {
    let wt_dir = get_worktree_dir();
    let wt_path = wt_dir.join(name);

    if wt_path.exists() {
        eprintln!("Worktree '{}' already exists at {}", name, wt_path.display());
        return;
    }

    // Create the parent directory
    fs::create_dir_all(&wt_dir).expect("failed to create .gg-worktrees directory");

    // Create a detached worktree (HEAD)
    let status = Command::new("git")
        .arg("worktree")
        .arg("add")
        .arg("--detach")
        .arg(wt_path.to_str().unwrap())
        .status()
        .expect("failed to run git worktree add");

    if !status.success() {
        eprintln!("Failed to create worktree '{}'", name);
        return;
    }

    // Update state
    let mut state = load_state();
    state.worktrees.push(WorktreeEntry {
        name: name.to_string(),
        path: wt_path.to_string_lossy().to_string(),
        branch: None,
        status: "idle".to_string(),
    });
    save_state(&state);

    println!("Created worktree '{}' at {}", name, wt_path.display());
}

/// List all managed worktrees.
pub(crate) fn list_worktrees() {
    let state = load_state();

    if state.worktrees.is_empty() {
        println!("No worktrees configured. Use `gg worktree create <name>` or `gg worktree pool --size N`.");
        return;
    }

    println!("{:<15} {:<10} {:<40} {}", "NAME", "STATUS", "BRANCH", "PATH");
    println!("{}", "-".repeat(80));
    for wt in &state.worktrees {
        let exists = Path::new(&wt.path).exists();
        let status = if !exists {
            "missing"
        } else {
            wt.status.as_str()
        };
        println!(
            "{:<15} {:<10} {:<40} {}",
            wt.name,
            status,
            wt.branch.as_deref().unwrap_or("-"),
            wt.path,
        );
    }

    if state.pool_size > 0 {
        println!("\nPool size: {}", state.pool_size);
    }
}

/// Assign a branch to a worktree.
pub(crate) fn assign_worktree(worktree_name: &str, branch: &str) {
    let mut state = load_state();

    let entry = match state.worktrees.iter_mut().find(|w| w.name == worktree_name) {
        Some(e) => e,
        None => {
            eprintln!("Worktree '{}' not found", worktree_name);
            return;
        }
    };

    let wt_path = entry.path.clone();

    // Checkout the branch in the worktree
    let status = Command::new("git")
        .arg("-C")
        .arg(&wt_path)
        .arg("checkout")
        .arg(branch)
        .status()
        .expect("failed to run git checkout in worktree");

    if !status.success() {
        // Try creating the branch in the worktree
        let status2 = Command::new("git")
            .arg("-C")
            .arg(&wt_path)
            .arg("checkout")
            .arg("-b")
            .arg(branch)
            .status()
            .expect("failed to run git checkout -b in worktree");

        if !status2.success() {
            eprintln!("Failed to checkout/create branch '{}' in worktree '{}'", branch, worktree_name);
            return;
        }
    }

    entry.branch = Some(branch.to_string());
    entry.status = "active".to_string();
    save_state(&state);

    println!("Assigned branch '{}' to worktree '{}' at {}", branch, worktree_name, wt_path);
}

/// Remove a worktree by name.
pub(crate) fn remove_worktree(name: &str) {
    let mut state = load_state();

    let idx = match state.worktrees.iter().position(|w| w.name == name) {
        Some(i) => i,
        None => {
            eprintln!("Worktree '{}' not found", name);
            return;
        }
    };

    let entry = &state.worktrees[idx];
    let wt_path = entry.path.clone();

    // Remove via git worktree
    let status = Command::new("git")
        .arg("worktree")
        .arg("remove")
        .arg("--force")
        .arg(&wt_path)
        .status()
        .expect("failed to run git worktree remove");

    if !status.success() {
        eprintln!("Warning: git worktree remove failed for '{}', cleaning up state anyway", name);
        // Try to clean up the directory manually
        if Path::new(&wt_path).exists() {
            fs::remove_dir_all(&wt_path).ok();
        }
    }

    state.worktrees.remove(idx);
    save_state(&state);

    println!("Removed worktree '{}'", name);
}

/// Pre-create N worktrees for agent use.
pub(crate) fn create_pool(size: u32) {
    let mut state = load_state();
    let existing_count = state.worktrees.len() as u32;

    if size <= existing_count {
        println!("Already have {} worktrees (requested {})", existing_count, size);
        state.pool_size = size;
        save_state(&state);
        return;
    }

    let to_create = size - existing_count;
    for i in 0..to_create {
        let name = format!("agent-{}", existing_count + i + 1);
        create_worktree(&name);
    }

    // Reload state after creating worktrees
    let mut state = load_state();
    state.pool_size = size;
    save_state(&state);

    println!("Worktree pool set to {} (created {} new)", size, to_create);
}
