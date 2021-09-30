
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::process::Command;
use std::str::from_utf8;
use std::fs::File;
use std::path::{Path};


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub repo_main_branch: String,
} 

pub fn get_config() -> Config {
    let path = Path::new(get_repo_root_path().as_str()).join(".git").join("GG_CONFIG");
    let mut file = File::open(path).expect("WANT IT!!!");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("no error reading!");
    let c: Config = serde_json::from_str(buf.as_str()).expect("res!");
    return c;
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