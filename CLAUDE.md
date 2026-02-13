# GG - Git Workflow Tool for Stacked PRs

GG is a Rust CLI tool for managing stacked git branches and PRs. It handles branch creation, navigation, rebasing, PR management, and parallel worktree workflows.

## Build

```bash
cargo build                   # Without GitHub API (uses gh CLI)
cargo build --features octo   # With octocrab GitHub API
```

## Core Commands

### Branch Management
- `gg new --feature <name>` - Start a new feature branch (creates both full and start branches)
- `gg new` - Create the next part in the current stack (auto-increments part number)
- `gg new --part <N>` - Create a specific part number
- `gg new --main` - Create branch from latest main (fetches first)
- `gg co --next` / `gg co --prev` - Navigate the stack
- `gg co --start` - Go to the first branch in the stack
- `gg co --part <N>` - Go to a specific part
- `gg br` - List all branches with PR status, review state, and CI status
- `gg del` - Delete current branch (and start branch), checks out prev first
- `gg del --branch <name>` - Delete a specific branch
- `gg rename <new-name>` - Rename current branch

### Pushing & PRs
- `gg push -f` - Force push current branch
- `gg push -f -s` - Force push current branch AND its start branch
- `gg pr -s` - Create PR with start branch as base (for stacked PRs)
- `gg pr -d` - Create draft PR
- `gg land` - Squash merge current PR, clean up branches

### Rebasing
- `gg rebase` (or `gg rs`) - Rebase current branch onto previous in stack
- `gg rebase --all` - Rebase all children in the stack
- `gg rebase --onto <branch>` - Rebase onto a specific branch
- `gg rebase --continue` / `-c` - Continue after resolving conflicts
- `gg rebase --abort` / `-a` - Abort current rebase
- `gg rebase --cleanup` - Clean up after manual rebase fixes

### Stack Merging
- `gg prep-merge` - Prepare current branch for merge to main: rebases onto origin/main, force pushes, changes PR base to main
- `gg merge-stack` - Single-pass: process ready PRs in stack order
- `gg merge-stack --dry-run` - Show what would happen without acting
- `gg merge-stack --watch` - Poll every 30s until all PRs merged
- `gg merge-stack --watch --interval 60` - Custom poll interval

### Worktrees (for parallel AI agents)
- `gg worktree create <name>` - Create a new worktree
- `gg worktree list` - List all worktrees with status
- `gg worktree assign <name> <branch>` - Assign a branch to a worktree
- `gg worktree remove <name>` - Remove a worktree
- `gg worktree pool --size N` - Pre-create N worktrees for agent use

### Configuration
- `gg init` - Interactive setup (main branch, prefix, org, split char)
- `gg config set merge-method <api|comment|auto|none>` - Set how prep-merge triggers merging
- `gg config set merge-command "/merge"` - Set the comment text for comment-based merging
- `gg config show` - Display current configuration

### Other
- `gg fetch` - Fetch main branch
- `gg sync` - Sync current branch with remote (if remote is newer)
- `gg diff` / `gg d` - Show diff from start branch
- `gg status` / `gg s` - Show changed files from start branch
- `gg cleanup` - Delete branches for closed PRs
- `gg cleanup -f` - Force delete without confirmation
- `gg setbase` - Change PR base to main branch
- `gg fixup` - Interactive rebase on start branch (for squashing)

## Branch Naming Convention

Branches follow the pattern: `{prefix}{split}{base}{split}part-{N}`

Example with prefix `wh` and split `/`:
- `wh/my-feature/part-1.0` (full branch)
- `wh/starts/my-feature/part-1.0` (start/base branch)
- `wh/my-feature/part-2.0` (next in stack)

Start branches track where a part begins, enabling per-part diffs and stacked PR bases.

## Configuration

Stored in `.git/GG_CONFIG` (JSON):
- `repo_main_branch` - "main" or "master"
- `branch_prefix` - e.g., "wh"
- `branch_split` - e.g., "/"
- `repo_org` - GitHub organization
- `merge_method` - "api", "comment", "auto", or null
- `merge_command` - e.g., "/merge"

## Workflow for Stacked PRs

1. `gg new --feature my-feature --main` - Start from main
2. Make changes, commit normally with git
3. `gg push -f -s && gg pr -s` - Push and create PR with start base
4. `gg new` - Start next part (auto-creates part-2.0)
5. Make more changes, commit
6. `gg push -f -s && gg pr -s` - Push and create stacked PR
7. After changes to part 1: `gg co --prev`, edit, commit, `gg rebase --all`
8. `gg push -f -s` on each branch to update remotes

## AI Agent Workflow

1. Set up worktree pool: `gg worktree pool --size 3`
2. List available worktrees: `gg worktree list`
3. Assign work: `gg worktree assign agent-1 wh/my-feature/part-1.0`
4. Work in the worktree directory (shown in `gg worktree list`)
5. All gg commands work from within worktrees
6. To land a stack: `gg merge-stack --watch` (monitors and processes automatically)

## Architecture

- `src/main.rs` - CLI entry point, command dispatch
- `src/git/mod.rs` - Git operations (branch, checkout, push, cherry-pick)
- `src/git_rebase/mod.rs` - Cherry-pick based rebase with hook management
- `src/config/mod.rs` - Configuration (`.git/GG_CONFIG`)
- `src/github/pr.rs` - PR operations (create, merge, change base, comments, auto-merge)
- `src/github/issue.rs` - Issue operations
- `src/github/mod.rs` - GithubRepo struct and initialization
- `src/merge.rs` - prep-merge and merge-stack logic
- `src/worktree.rs` - Worktree management
- `src/terminal/` - TUI for branch/issue viewing
- `src/color/mod.rs` - Terminal color utilities
- `src/record/mod.rs` - Status/daily record tracking
- `src/file/mod.rs` - Vim editor integration
