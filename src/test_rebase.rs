use crate::git;
use crate::git_rebase;
use crate::test_helpers::with_test_repo;

/// Helper: create a 2-part stack where part-1.0 and part-2.0 each have distinct commits.
/// Returns with checkout on part-2.0.
fn setup_two_part_stack(repo: &crate::test_helpers::TestRepo) {
    // Create start and full for part-1.0
    git::new("test/starts/rb/part-1.0");
    git::new("test/rb/part-1.0");
    repo.commit_file("part1.txt", "part1 content");

    // Create start and full for part-2.0 (branched from part-1.0)
    git::new("test/starts/rb/part-2.0");
    git::new("test/rb/part-2.0");
    repo.commit_file("part2.txt", "part2 content");
}

#[test]
fn test_fast_forward_rebase() {
    with_test_repo(|_repo| {
        // Create start and full for part-1.0 at same commit (no unique commits on part)
        git::new("test/starts/ff/part-1.0");
        git::new("test/ff/part-1.0");
        // start and full point to same commit - no cherry-picks needed

        // Create an "onto" branch with a commit ahead
        git::checkout("main");
        git::new("test/onto-branch");
        std::fs::write("onto.txt", "onto content").unwrap();
        std::process::Command::new("git")
            .args(["add", "onto.txt"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "onto commit"])
            .output()
            .unwrap();
        let onto_hash = git::get_commit_hash("test/onto-branch".to_string());

        // Now rebase part-1.0 onto the onto-branch (fast-forward case)
        git::checkout("test/ff/part-1.0");
        git_rebase::start_rebase(Some("test/onto-branch".to_string()), None);

        // Both start and full should now point to onto_hash
        let full_hash = git::get_commit_hash("test/ff/part-1.0".to_string());
        let start_hash = git::get_commit_hash("test/starts/ff/part-1.0".to_string());
        assert_eq!(full_hash, onto_hash);
        assert_eq!(start_hash, onto_hash);
    });
}

#[test]
fn test_cherry_pick_rebase_success() {
    with_test_repo(|repo| {
        setup_two_part_stack(repo);

        // Add a new commit to part-1.0
        git::checkout("test/rb/part-1.0");
        repo.commit_file("part1_extra.txt", "extra on part1");

        // Rebase part-2.0 onto updated part-1.0
        git::checkout("test/rb/part-2.0");
        git_rebase::start_rebase(Some("test/rb/part-1.0".to_string()), None);

        // Should end up on part-2.0
        assert_eq!(git::current_branch(), "test/rb/part-2.0");

        // part2.txt should still exist
        assert!(
            std::path::Path::new("part2.txt").exists(),
            "part2.txt should exist after rebase"
        );
        // part1_extra.txt should also be visible (since we rebased onto part-1.0 which has it)
        assert!(
            std::path::Path::new("part1_extra.txt").exists(),
            "part1_extra.txt should exist after rebase"
        );
    });
}

#[test]
fn test_rebase_abort() {
    with_test_repo(|repo| {
        // Create a conflict scenario
        git::new("test/starts/conflict/part-1.0");
        git::new("test/conflict/part-1.0");
        repo.commit_file("conflict.txt", "original from part1");

        git::new("test/starts/conflict/part-2.0");
        git::new("test/conflict/part-2.0");
        // Modify the same file differently
        std::fs::write("conflict.txt", "modified by part2").unwrap();
        std::process::Command::new("git")
            .args(["add", "conflict.txt"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "part2 conflict change"])
            .output()
            .unwrap();

        // Go back to part-1.0 and make a conflicting change
        git::checkout("test/conflict/part-1.0");
        std::fs::write("conflict.txt", "conflicting change on part1").unwrap();
        std::process::Command::new("git")
            .args(["add", "conflict.txt"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "part1 conflicting change"])
            .output()
            .unwrap();

        // Try to rebase part-2.0 onto updated part-1.0 (should conflict)
        git::checkout("test/conflict/part-2.0");
        git_rebase::start_rebase(Some("test/conflict/part-1.0".to_string()), None);

        // We should be on a tmp branch due to conflict
        let cur = git::current_branch();
        assert!(
            cur.starts_with("_tmp_-"),
            "Should be on tmp branch during conflict, got: {}",
            cur
        );

        // Abort the rebase
        git_rebase::abort_rebase();

        // Should be back on the original branch
        assert_eq!(git::current_branch(), "test/conflict/part-2.0");

        // Content should be the original part2 content
        let content = std::fs::read_to_string("conflict.txt").unwrap();
        assert_eq!(content, "modified by part2");
    });
}

#[test]
fn test_rebase_all_children() {
    with_test_repo(|repo| {
        // Create a 3-part stack
        git::new("test/starts/all/part-1.0");
        git::new("test/all/part-1.0");
        repo.commit_file("p1.txt", "p1");

        git::new("test/starts/all/part-2.0");
        git::new("test/all/part-2.0");
        repo.commit_file("p2.txt", "p2");

        git::new("test/starts/all/part-3.0");
        git::new("test/all/part-3.0");
        repo.commit_file("p3.txt", "p3");

        // Go back to part-1.0 and add a commit
        git::checkout("test/all/part-1.0");
        repo.commit_file("p1_new.txt", "new on p1");

        // Rebase all children
        git_rebase::rebase_all_children(None);

        // Check that part-2.0 has the new file
        git::checkout("test/all/part-2.0");
        assert!(
            std::path::Path::new("p1_new.txt").exists(),
            "p1_new.txt should exist on part-2.0 after rebase"
        );
        assert!(
            std::path::Path::new("p2.txt").exists(),
            "p2.txt should still exist on part-2.0"
        );

        // Check that part-3.0 has the new file
        git::checkout("test/all/part-3.0");
        assert!(
            std::path::Path::new("p1_new.txt").exists(),
            "p1_new.txt should exist on part-3.0 after rebase"
        );
        assert!(
            std::path::Path::new("p3.txt").exists(),
            "p3.txt should still exist on part-3.0"
        );
    });
}

#[test]
fn test_rebase_preserves_file_content() {
    with_test_repo(|repo| {
        git::new("test/starts/preserve/part-1.0");
        git::new("test/preserve/part-1.0");
        repo.commit_file("base.txt", "base content");

        git::new("test/starts/preserve/part-2.0");
        git::new("test/preserve/part-2.0");
        repo.commit_file("feature.txt", "feature content line 1\nfeature content line 2\n");

        // Add something to part-1.0
        git::checkout("test/preserve/part-1.0");
        repo.commit_file("base2.txt", "another base file");

        // Rebase part-2.0
        git::checkout("test/preserve/part-2.0");
        git_rebase::start_rebase(Some("test/preserve/part-1.0".to_string()), None);

        // Verify file contents
        let content = std::fs::read_to_string("feature.txt").unwrap();
        assert_eq!(
            content, "feature content line 1\nfeature content line 2\n",
            "File content should be preserved after rebase"
        );
        assert!(std::path::Path::new("base2.txt").exists());
    });
}
