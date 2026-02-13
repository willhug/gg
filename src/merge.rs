use std::collections::HashSet;

use crate::config;
use crate::git;
use crate::git_rebase;
use crate::github::pr::Pr;
use crate::github::GithubRepo;

pub(crate) struct MergeStackOpts {
    pub dry_run: bool,
    pub max_part: Option<f32>,
    pub parts: Vec<f32>,
    pub skip_approval: bool,
    pub skip_tests: bool,
}

/// Run prep-merge: rebase onto main, force push, change PR base to main, optionally trigger merge.
pub(crate) async fn prep_merge(github: &GithubRepo) -> anyhow::Result<()> {
    let cfg = config::get_saved_config();
    let main_branch = cfg.repo_main_branch.clone();
    let branch = git::current_parsed_branch();

    println!("Preparing {} for merge to {}", branch.full(), main_branch);

    // 1. Fetch main and rebase onto origin/main
    git::fetch_main();
    let onto = format!("origin/{}", main_branch);
    git_rebase::start_rebase(Some(onto), None);

    // 2. Force push both branches together
    git::push(vec![branch.full(), branch.start()], true);

    // 3. Change the PR base to main via GitHub API
    github
        .change_base(branch.full(), main_branch.clone())
        .await?;
    println!("Changed PR base to {}", main_branch);

    // 4. Optionally trigger merge based on merge_method config
    let merge_method = cfg.merge_method.as_deref().unwrap_or("none");
    let merge_command = cfg.merge_command.clone();

    match merge_method {
        "api" => {
            println!("Triggering rebase merge via API...");
            github.land_pr(branch.full()).await?;
        }
        "comment" => {
            let comment = merge_command.unwrap_or_else(|| "/merge".to_string());
            println!("Posting merge comment: {}", comment);
            github.post_comment(branch.full(), &comment).await?;
        }
        "auto" => {
            println!("Enabling auto-merge...");
            github.enable_auto_merge(branch.full()).await?;
        }
        _ => {
            println!("No merge method configured. PR is ready for manual merge.");
        }
    }

    Ok(())
}

/// Single-pass merge-stack: process the stack in order.
pub(crate) async fn merge_stack(
    github: &GithubRepo,
    opts: &MergeStackOpts,
) -> anyhow::Result<bool> {
    let branch = git::current_parsed_branch();
    let all_branches = git::get_sorted_matching_branches(&branch.base);

    if all_branches.is_empty() {
        println!("No branches found in current stack.");
        return Ok(true);
    }

    // Convert parts list to partx100 values for filtering
    let parts_filter: HashSet<u32> = opts
        .parts
        .iter()
        .map(|p| (*p * 100.0) as u32)
        .collect();
    let max_partx100 = opts.max_part.map(|p| (p * 100.0) as u32);

    // Filter branches based on --max-part and --parts
    let filtered_branches: Vec<_> = all_branches
        .iter()
        .filter(|br| {
            let partx100 = br.partx100.unwrap_or(0);
            if let Some(max) = max_partx100 {
                if partx100 > max {
                    return false;
                }
            }
            if !parts_filter.is_empty() {
                return parts_filter.contains(&partx100);
            }
            true
        })
        .collect();

    // Fetch PR status for all branches
    let branch_names: HashSet<String> = filtered_branches.iter().map(|b| b.full()).collect();
    let prs = github.prs_for_branches(&branch_names).await?;

    // Build a map from branch name to PR
    let pr_map: std::collections::HashMap<String, &Pr> =
        prs.iter().map(|pr| (pr.branch.clone(), pr)).collect();

    let mut all_merged = true;
    let mut first_actionable = true;

    println!("\n--- Stack Status ---");
    for br in &filtered_branches {
        let full = br.full();
        match pr_map.get(&full) {
            Some(pr) => {
                let is_merged = pr.state == "MERGED";
                let is_approved = opts.skip_approval
                    || pr
                        .review_decision
                        .as_deref()
                        .map(|d| d == "APPROVED")
                        .unwrap_or(false);
                let tests_pass = opts.skip_tests
                    || matches!(pr.test_status.as_str(), "SUCCESS" | "EXPECTED");
                let is_closed = pr.closed;

                let status_str = if is_merged {
                    "MERGED"
                } else if is_closed {
                    "CLOSED"
                } else if is_approved && tests_pass {
                    "READY"
                } else {
                    "PENDING"
                };

                println!(
                    "  {} - {} (review: {}, tests: {})",
                    full,
                    status_str,
                    pr.review_decision.as_deref().unwrap_or("none"),
                    pr.test_status,
                );

                if is_merged {
                    if !opts.dry_run {
                        println!("  -> Cleaning up merged branch: {}", full);
                        git::delete_branch_all(br.full());
                        git::delete_branch_all(br.start());
                    } else {
                        println!("  -> [dry-run] Would clean up merged branch: {}", full);
                    }
                } else if first_actionable && !is_closed {
                    all_merged = false;
                    if is_approved && tests_pass {
                        if !opts.dry_run {
                            println!("  -> Running prep-merge on: {}", full);
                            git::checkout(&full);
                            if let Err(e) = prep_merge(github).await {
                                eprintln!("  -> prep-merge failed: {}", e);
                            }
                        } else {
                            println!("  -> [dry-run] Would run prep-merge on: {}", full);
                        }
                    } else {
                        println!(
                            "  -> Waiting: {} needs {} {}",
                            full,
                            if !is_approved { "approval" } else { "" },
                            if !tests_pass { "passing tests" } else { "" },
                        );
                    }
                    first_actionable = false;
                } else if !is_closed {
                    all_merged = false;
                }
            }
            None => {
                println!("  {} - NO PR", full);
                all_merged = false;
            }
        }
    }
    println!("---\n");

    Ok(all_merged)
}

/// Watch mode: keep polling until all PRs are merged.
pub(crate) async fn merge_stack_watch(
    github: &GithubRepo,
    interval_secs: u64,
    opts: &MergeStackOpts,
) -> anyhow::Result<()> {
    loop {
        let all_done = merge_stack(github, opts).await?;
        if all_done {
            println!("All PRs in stack are merged!");
            return Ok(());
        }
        println!("Waiting {}s before next check...", interval_secs);
        tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
    }
}
