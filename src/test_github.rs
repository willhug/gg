use crate::github::GithubRepo;
use crate::test_helpers::{setup_test_repo, GhMockBuilder};

fn make_github() -> GithubRepo {
    GithubRepo {
        org: "testorg".to_string(),
        repo: "testrepo".to_string(),
        current_user: "testuser".to_string(),
    }
}

#[tokio::test]
async fn test_land_pr() {
    let (_lock, _repo, _guard) = setup_test_repo();

    let _mock = GhMockBuilder::new()
        .on_args_containing("pr merge", "Merged", 0)
        .build();

    let github = make_github();
    let result = github.land_pr("test/feat/part-1.0".to_string()).await;
    assert!(result.is_ok(), "land_pr should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_change_base() {
    let (_lock, _repo, _guard) = setup_test_repo();

    let _mock = GhMockBuilder::new()
        .on_args_containing("pr edit", "", 0)
        .build();

    let github = make_github();
    let result = github
        .change_base("test/feat/part-1.0".to_string(), "main".to_string())
        .await;
    assert!(
        result.is_ok(),
        "change_base should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_post_comment() {
    let (_lock, _repo, _guard) = setup_test_repo();

    let _mock = GhMockBuilder::new()
        .on_args_containing("pr comment", "", 0)
        .build();

    let github = make_github();
    let result = github
        .post_comment("test/feat/part-1.0".to_string(), "/merge")
        .await;
    assert!(
        result.is_ok(),
        "post_comment should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_enable_auto_merge() {
    let (_lock, _repo, _guard) = setup_test_repo();

    let _mock = GhMockBuilder::new()
        .on_args_containing("pr merge", "", 0)
        .build();

    let github = make_github();
    let result = github
        .enable_auto_merge("test/feat/part-1.0".to_string())
        .await;
    assert!(
        result.is_ok(),
        "enable_auto_merge should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_prs_for_branches() {
    let (_lock, _repo, _guard) = setup_test_repo();

    // gh CLI returns camelCase for headRefName (has serde rename),
    // but review_decision has no rename attribute so serde expects snake_case.
    let json_response = r#"[
        {
            "closed": false,
            "title": "Add feature",
            "headRefName": "test/feat/part-1.0",
            "url": "https://github.com/testorg/testrepo/pull/1",
            "review_decision": "APPROVED",
            "mergeable": "MERGEABLE"
        },
        {
            "closed": true,
            "title": "Old PR",
            "headRefName": "test/feat/part-2.0",
            "url": "https://github.com/testorg/testrepo/pull/2",
            "review_decision": null,
            "mergeable": "UNKNOWN"
        }
    ]"#;

    let _mock = GhMockBuilder::new()
        .on_args_containing("pr list", json_response, 0)
        .build();

    let github = make_github();
    let mut branches = std::collections::HashSet::new();
    branches.insert("test/feat/part-1.0".to_string());
    branches.insert("test/feat/part-2.0".to_string());

    let prs = github
        .prs_for_branches(&branches)
        .await
        .expect("prs_for_branches should succeed");

    assert_eq!(prs.len(), 2);

    let pr1 = prs
        .iter()
        .find(|p| p.branch == "test/feat/part-1.0")
        .unwrap();
    assert!(!pr1.closed);
    assert_eq!(pr1.title, "Add feature");
    assert_eq!(pr1.review_decision, Some("APPROVED".to_string()));

    let pr2 = prs
        .iter()
        .find(|p| p.branch == "test/feat/part-2.0")
        .unwrap();
    assert!(pr2.closed);
    assert_eq!(pr2.title, "Old PR");
}
