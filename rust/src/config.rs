
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;
use std::str::from_utf8;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};


#[derive(Debug)]
pub struct FullConfig {
    pub saved: SavedConfig,
    pub repo_name: String,
    pub repo_org: String,
    pub current_github_user: String,
    pub github_token: String,
    pub status_file: String,
    pub status_file_backup_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedConfig {
    pub repo_main_branch: String,
    pub linked_issue: Option<i64>,
    pub branch_prefix: String,
}

pub fn get_full_config() -> FullConfig {
    FullConfig {
        saved: get_saved_config(),
        repo_name: get_repo_name(),
        repo_org: "willhug".to_string(),
        current_github_user: "willhug".to_string(),
        github_token: std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required"),
        status_file: "/home/will/status.txt".to_string(),
        status_file_backup_dir: "/home/will/status_bu".to_string(),
    }
}

fn get_repo_name() -> String {
    let out = match Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output() {
                Ok(output) => output,
                Err(_e) => panic!("error!")
    };
    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    result.split('/').last().unwrap().to_string()
}

pub fn get_saved_config() -> SavedConfig {
    let path = get_saved_config_file_path();
    let file = File::open(path);

    let mut file = match file {
        Ok(file) => file,
        Err(_error) => create_saved_config(),
    };

    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("no error reading!");
    let c: SavedConfig = serde_json::from_str(buf.as_str()).expect("res!");
    c
}

fn get_saved_config_file_path() -> PathBuf {
    Path::new(get_repo_root_path().as_str()).join(".git").join("GG_CONFIG")
}

fn get_repo_root_path() -> String {
    let out = match Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output() {
                Ok(output) => output,
                Err(_e) => panic!("error!")
    };
    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let result = from_utf8(&out.stdout)
        .expect("msg")
        .trim_end_matches(x);
    result.to_string()
}

pub fn clear_selected_issue() {
    let mut cfg = get_saved_config();

    cfg.linked_issue = None;

    write_saved_config(cfg);
}

pub fn update_selected_issue(issue: i64) {
    let mut cfg = get_saved_config();

    cfg.linked_issue = Some(issue);

    write_saved_config(cfg);
}

pub fn get_selected_issue_number() -> i64 {
    let cfg = get_saved_config();
    cfg.linked_issue.unwrap_or(0)
}

fn create_saved_config() -> File {
    // Query for main/master info
    println!("Config file not found, creating one at {}", get_saved_config_file_path().to_str().expect("should have been str"));
    println!("Please input what the main branch is [main|master]: ");

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();

    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let line = line.trim_end_matches(x);
 
    println!("What will the branch prefix be: ");
    let mut prefix = String::new();
    std::io::stdin().read_line(&mut prefix).unwrap();
    let prefix = line.trim_end_matches(x);

    let config = SavedConfig {
        repo_main_branch: line.to_string(),
        linked_issue: None,
        branch_prefix: prefix.to_string(), // replace
    };

    write_saved_config(config)
}

fn write_saved_config(cfg: SavedConfig) -> File {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(get_saved_config_file_path())
        .unwrap();

    let cfg_json = serde_json::to_string(&cfg).expect("error serializing");

    file.write_all(cfg_json.as_bytes()).expect("could not write to cfg file");
    file.sync_all().expect("should be able to write to disk");
    file.seek(SeekFrom::Start(0)).unwrap();

    file
}