use crate::git;
use crate::github::GithubRepo;
use crate::merge;
use crate::test_helpers::{setup_test_repo, GhMockBuilder};

fn make_github() -> GithubRepo {
    GithubRepo {
        org: "testorg".to_string(),
        repo: "testrepo".to_string(),
        current_user: "testuser".to_string(),
    }
}

#[tokio::test]
async fn test_merge_stack_dry_run() {
    let (_lock, repo, _guard) = setup_test_repo();

    // Create a 2-part stack
    git::new("test/starts/ms/part-1.0");
    git::new("test/ms/part-1.0");
    repo.commit_file("f1.txt", "c1");

    git::checkout("main");
    git::new("test/starts/ms/part-2.0");
    git::new("test/ms/part-2.0");
    repo.commit_file("f2.txt", "c2");

    // Mock PRs: part-1.0 PENDING, part-2.0 CLOSED
    let json_response = r#"[
        {
            "closed": false,
            "title": "Part 1",
            "headRefName": "test/ms/part-1.0",
            "url": "https://github.com/testorg/testrepo/pull/1",
            "state": "OPEN",
            "reviewDecision": null,
            "mergeable": "MERGEABLE",
            "test_status": "PENDING"
        },
        {
            "closed": true,
            "title": "Part 2",
            "headRefName": "test/ms/part-2.0",
            "url": "https://github.com/testorg/testrepo/pull/2",
            "state": "CLOSED",
            "reviewDecision": null,
            "mergeable": "UNKNOWN",
            "test_status": ""
        }
    ]"#;

    let _mock = GhMockBuilder::new()
        .on_args_containing("pr list", json_response, 0)
        .build();

    let github = make_github();

    // Checkout part-1.0 so merge_stack can find the stack
    git::checkout("test/ms/part-1.0");

    let opts = merge::MergeStackOpts {
        dry_run: true,
        max_part: None,
        parts: vec![],
        skip_approval: false,
        skip_tests: false,
    };
    let result = merge::merge_stack(&github, &opts).await;
    assert!(
        result.is_ok(),
        "merge_stack should succeed: {:?}",
        result.err()
    );
    let all_merged = result.unwrap();
    assert!(!all_merged, "Not all should be merged (part-1 is pending)");

    // In dry-run, branches should NOT be deleted
    assert!(git::branch_exists("test/ms/part-1.0"));
    assert!(git::branch_exists("test/ms/part-2.0"));
}

#[tokio::test]
async fn test_merge_stack_all_closed() {
    let (_lock, repo, _guard) = setup_test_repo();

    // Create a 2-part stack
    git::new("test/starts/mc/part-1.0");
    git::new("test/mc/part-1.0");
    repo.commit_file("f1.txt", "c1");

    git::checkout("main");
    git::new("test/starts/mc/part-2.0");
    git::new("test/mc/part-2.0");
    repo.commit_file("f2.txt", "c2");

    // Mock PRs: both MERGED
    let json_response = r#"[
        {
            "closed": true,
            "title": "Part 1",
            "headRefName": "test/mc/part-1.0",
            "url": "https://github.com/testorg/testrepo/pull/1",
            "state": "MERGED",
            "reviewDecision": "APPROVED",
            "mergeable": "MERGEABLE",
            "test_status": "SUCCESS"
        },
        {
            "closed": true,
            "title": "Part 2",
            "headRefName": "test/mc/part-2.0",
            "url": "https://github.com/testorg/testrepo/pull/2",
            "state": "MERGED",
            "reviewDecision": "APPROVED",
            "mergeable": "MERGEABLE",
            "test_status": "SUCCESS"
        }
    ]"#;

    let _mock = GhMockBuilder::new()
        .on_args_containing("pr list", json_response, 0)
        .build();

    let github = make_github();

    git::checkout("test/mc/part-1.0");

    let opts = merge::MergeStackOpts {
        dry_run: false,
        max_part: None,
        parts: vec![],
        skip_approval: false,
        skip_tests: false,
    };
    let result = merge::merge_stack(&github, &opts).await;
    assert!(result.is_ok());
    let all_merged = result.unwrap();
    assert!(all_merged, "All PRs are merged, should return true");
}
