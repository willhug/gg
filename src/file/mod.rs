use tempfile::NamedTempFile;
use std::{io::{Read, Write}, env};
use std::process::Command;

pub fn open_vim(input: String) -> String {
    let mut file = match env::consts::OS {
        "macos" => {
            NamedTempFile::new_in("/tmp").expect("could not create file")
        }
        _ => {
            NamedTempFile::new().expect("could not create file")
        }
    };
    file.write_all(input.as_bytes()).expect("could not initialize file");
    let path = file.path().to_str().expect("msg");
    println!("file created for vim! {}", path);
    Command::new("vim")
            .arg(path)
            .status()
            .expect("failed to open file");

    println!("finished running vim grabbing contents from file!");

    let mut file2 = file.reopen().expect("could not open");
    let mut buf = String::new();
    file2.read_to_string(&mut buf).expect("error reading buffer");

    buf
}