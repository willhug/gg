use crate::config;
use crate::test_helpers::with_test_repo;

#[test]
fn test_read_default_config() {
    with_test_repo(|_repo| {
        let cfg = config::get_saved_config();
        assert_eq!(cfg.repo_main_branch, "main");
        assert_eq!(cfg.branch_prefix, "test");
        assert_eq!(cfg.repo_org, "testorg");
        assert_eq!(cfg.branch_split, "/");
        assert_eq!(cfg.linked_issue, None);
        assert_eq!(cfg.merge_command, None);
        assert_eq!(cfg.merge_method, None);
    });
}

#[test]
fn test_update_selected_issue() {
    with_test_repo(|_repo| {
        config::update_selected_issue(42);
        let cfg = config::get_saved_config();
        assert_eq!(cfg.linked_issue, Some(42));
    });
}

#[test]
fn test_clear_selected_issue() {
    with_test_repo(|_repo| {
        config::update_selected_issue(42);
        config::clear_selected_issue();
        let cfg = config::get_saved_config();
        assert_eq!(cfg.linked_issue, None);
    });
}

#[test]
fn test_update_merge_config() {
    with_test_repo(|_repo| {
        config::update_merge_config(Some("api".to_string()), Some("/merge".to_string()));
        let cfg = config::get_saved_config();
        assert_eq!(cfg.merge_method, Some("api".to_string()));
        assert_eq!(cfg.merge_command, Some("/merge".to_string()));
    });
}

#[test]
fn test_update_prefix_and_split() {
    with_test_repo(|_repo| {
        config::update_prefix_and_split("newprefix", "-");
        let cfg = config::get_saved_config();
        assert_eq!(cfg.branch_prefix, "newprefix");
        assert_eq!(cfg.branch_split, "-");
    });
}

#[test]
fn test_config_default_split() {
    with_test_repo(|repo| {
        // Write a config without branch_split field to test default
        let config_no_split = r#"{"repo_main_branch":"main","linked_issue":null,"branch_prefix":"test","repo_org":"testorg"}"#;
        std::fs::write(repo.path().join(".git").join("GG_CONFIG"), config_no_split)
            .expect("failed to write config");
        let cfg = config::get_saved_config();
        assert_eq!(cfg.branch_split, "/");
    });
}
