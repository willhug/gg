
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;
use std::str::from_utf8;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub repo_main_branch: String,
    pub linked_issue: Option<i64>,
} 

pub fn get_config() -> Config {
    let path = get_config_file_path();
    let file = File::open(path.clone());

    let mut file = match file {
        Ok(file) => file,
        Err(_error) => create_config(),
    };

    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("no error reading!");
    let c: Config = serde_json::from_str(buf.as_str()).expect("res!");
    return c;
}

fn get_config_file_path() -> PathBuf {
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
    return result.to_string()
}

pub fn update_selected_issue(issue: i64) {
    let mut cfg = get_config();

    cfg.linked_issue = Some(issue);

    write_config(cfg);
}

pub fn get_selected_issue_number() -> i64 {
    let cfg = get_config();
    match cfg.linked_issue {
        Some(issue) => issue,
        None => 0,
    }
}

fn create_config() -> File {
    // Query for main/master info
    println!("Config file not found, creating one at {}", get_config_file_path().to_str().expect("should have been str"));
    println!("Please input what the main branch is [main|master]: ");

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();

    let x: &[_] = &[' ', '\t', '\n', '\r'];
    let line = line.trim_end_matches(x);

    let config = Config {
        repo_main_branch: line.to_string(),
        linked_issue: None,
    };

    write_config(config)
}

fn write_config(cfg: Config) -> File {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(get_config_file_path())
        .unwrap();

    let cfg_json = serde_json::to_string(&cfg).expect("error serializing");

    file.write_all(cfg_json.as_bytes()).expect("could not write to cfg file");
    file.sync_all().expect("should be able to write to disk");
    file.seek(SeekFrom::Start(0)).unwrap();

    return file;
}