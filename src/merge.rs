use crate::config;
use crate::git;
use crate::git_rebase;
use crate::github::GithubRepo;

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
