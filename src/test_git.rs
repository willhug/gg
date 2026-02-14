use crate::git;
use crate::test_helpers::with_test_repo;

#[test]
fn test_new_creates_branch() {
    with_test_repo(|_repo| {
        git::new("test/feat/part-1.0");
        let branches = git::all_branches();
        assert!(
            branches.contains(&"test/feat/part-1.0".to_string()),
            "Branch test/feat/part-1.0 should exist, got: {:?}",
            branches
        );
    });
}

#[test]
fn test_current_branch() {
    with_test_repo(|_repo| {
        git::new("test/my-feature/part-1.0");
        let cur = git::current_branch();
        assert_eq!(cur, "test/my-feature/part-1.0");
    });
}

#[test]
fn test_all_branches() {
    with_test_repo(|_repo| {
        git::new("test/feat/part-1.0");
        git::checkout("main");
        git::new("test/feat/part-2.0");
        let branches = git::all_branches();
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"test/feat/part-1.0".to_string()));
        assert!(branches.contains(&"test/feat/part-2.0".to_string()));
    });
}

#[test]
fn test_all_managed_branches() {
    with_test_repo(|_repo| {
        // Create managed branches
        git::new("test/feat/part-1.0");
        git::checkout("main");
        git::new("test/starts/feat/part-1.0");
        git::checkout("main");
        git::new("test/feat/part-2.0");
        git::checkout("main");
        git::new("other-branch");

        let managed = git::all_managed_branches();
        assert!(managed.contains(&"test/feat/part-1.0".to_string()));
        assert!(managed.contains(&"test/feat/part-2.0".to_string()));
        // start branches should be excluded
        assert!(!managed.contains(&"test/starts/feat/part-1.0".to_string()));
        // non-prefixed branches should be excluded
        assert!(!managed.contains(&"other-branch".to_string()));
        assert!(!managed.contains(&"main".to_string()));
    });
}

#[test]
fn test_branch_exists() {
    with_test_repo(|_repo| {
        assert!(git::branch_exists("main"));
        assert!(!git::branch_exists("nonexistent-branch"));
        git::new("test/exists/part-1.0");
        assert!(git::branch_exists("test/exists/part-1.0"));
    });
}

#[test]
fn test_delete_branch_local() {
    with_test_repo(|_repo| {
        git::new("test/to-delete/part-1.0");
        git::checkout("main");
        assert!(git::branch_exists("test/to-delete/part-1.0"));
        git::delete_branch_local("test/to-delete/part-1.0");
        assert!(!git::branch_exists("test/to-delete/part-1.0"));
    });
}

#[test]
fn test_parse_branch_with_prefix_and_part() {
    with_test_repo(|_repo| {
        let parsed = git::parse_branch("test/my-feature/part-1.0".to_string());
        assert_eq!(parsed.prefix, Some("test".to_string()));
        // The base includes everything up to the split+part- marker.
        // For "my-feature/part-1.0" splitting on "/part-" gives "my-feature" as base.
        assert_eq!(parsed.base, "my-feature");
        assert_eq!(parsed.partx100, Some(100));
    });
}

#[test]
fn test_parse_branch_without_part() {
    with_test_repo(|_repo| {
        let parsed = git::parse_branch("test/my-feature".to_string());
        assert_eq!(parsed.prefix, Some("test".to_string()));
        assert_eq!(parsed.base, "my-feature");
        assert_eq!(parsed.partx100, None);
    });
}

#[test]
fn test_parsed_branch_full_and_start() {
    with_test_repo(|_repo| {
        let parsed = git::parse_branch("test/my-feature/part-1.0".to_string());
        assert_eq!(parsed.full(), "test/my-feature/part-1.0");
        assert_eq!(parsed.start(), "test/starts/my-feature/part-1.0");
    });
}

#[test]
fn test_parsed_branch_with_split() {
    with_test_repo(|_repo| {
        let parsed = git::parse_branch("test/my-feature/part-1.0".to_string());
        assert_eq!(parsed.full_with_split("-"), "test-my-feature-part-1.0");
        assert_eq!(
            parsed.start_with_split("-"),
            "test-starts-my-feature-part-1.0"
        );
    });
}

#[test]
fn test_is_start_branch() {
    with_test_repo(|_repo| {
        assert!(git::is_start_branch(
            &"test/starts/my-feature/part-1.0".to_string()
        ));
        assert!(!git::is_start_branch(
            &"test/my-feature/part-1.0".to_string()
        ));
        assert!(!git::is_start_branch(&"main".to_string()));
    });
}

#[test]
fn test_get_sorted_matching_branches() {
    with_test_repo(|repo| {
        // Create a stack of 3 parts
        git::new("test/sorted/part-1.0");
        repo.commit_file("file1.txt", "content1");
        git::checkout("main");
        git::new("test/sorted/part-3.0");
        repo.commit_file("file3.txt", "content3");
        git::checkout("main");
        git::new("test/sorted/part-2.0");
        repo.commit_file("file2.txt", "content2");

        let sorted = git::get_sorted_matching_branches("sorted");
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].partx100, Some(100));
        assert_eq!(sorted[1].partx100, Some(200));
        assert_eq!(sorted[2].partx100, Some(300));
    });
}

#[test]
fn test_get_children_branches() {
    with_test_repo(|repo| {
        git::new("test/children/part-1.0");
        repo.commit_file("f1.txt", "c1");
        git::checkout("main");
        git::new("test/children/part-2.0");
        repo.commit_file("f2.txt", "c2");
        git::checkout("main");
        git::new("test/children/part-3.0");
        repo.commit_file("f3.txt", "c3");

        let parent = git::parse_branch("test/children/part-1.0".to_string());
        let children = git::get_children_branches(&parent);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].partx100, Some(200));
        assert_eq!(children[1].partx100, Some(300));
    });
}

#[test]
fn test_get_branch_for_dir() {
    with_test_repo(|repo| {
        // Create a 3-part stack
        git::new("test/nav/part-1.0");
        repo.commit_file("f1.txt", "c1");
        git::checkout("main");
        git::new("test/nav/part-2.0");
        repo.commit_file("f2.txt", "c2");
        git::checkout("main");
        git::new("test/nav/part-3.0");
        repo.commit_file("f3.txt", "c3");

        // From part-2.0
        git::checkout("test/nav/part-2.0");
        assert_eq!(
            git::get_branch_for_dir(git::CheckoutDir::Next),
            Some("test/nav/part-3.0".to_string())
        );
        assert_eq!(
            git::get_branch_for_dir(git::CheckoutDir::Prev),
            Some("test/nav/part-1.0".to_string())
        );
        assert_eq!(
            git::get_branch_for_dir(git::CheckoutDir::Start),
            Some("test/nav/part-1.0".to_string())
        );
        assert_eq!(
            git::get_branch_for_dir(git::CheckoutDir::Part(3.0)),
            Some("test/nav/part-3.0".to_string())
        );

        // From part-1.0: no prev
        git::checkout("test/nav/part-1.0");
        assert_eq!(git::get_branch_for_dir(git::CheckoutDir::Prev), None);

        // From part-3.0: no next
        git::checkout("test/nav/part-3.0");
        assert_eq!(git::get_branch_for_dir(git::CheckoutDir::Next), None);
    });
}

#[test]
fn test_force_branch_to_be_and_commit_hash() {
    with_test_repo(|repo| {
        git::new("test/force/part-1.0");
        repo.commit_file("extra.txt", "extra");
        let hash_with_extra = git::get_commit_hash("test/force/part-1.0".to_string());

        git::checkout("main");
        let hash_main = git::get_commit_hash("main".to_string());

        // They should be different
        assert_ne!(hash_with_extra, hash_main);

        // Force the branch to point at main
        git::force_branch_to_be("test/force/part-1.0", "main");
        let hash_after = git::get_commit_hash("test/force/part-1.0".to_string());
        assert_eq!(hash_after, hash_main);
    });
}

#[test]
fn test_diff_and_status_no_panic() {
    with_test_repo(|repo| {
        // Create a branch with a start branch, add some commits
        git::new("test/starts/difftest/part-1.0");
        git::new("test/difftest/part-1.0");
        repo.commit_file("changed.txt", "new content");

        // These should not panic
        git::diff("test/starts/difftest/part-1.0".to_string(), None);
        git::status("test/starts/difftest/part-1.0".to_string(), None);
    });
}
