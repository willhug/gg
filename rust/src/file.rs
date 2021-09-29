use tempfile::NamedTempFile;
use std::io::{Read, Write};
use std::process::Command;

pub fn open_vim(input: String) -> String {
    let mut file = NamedTempFile::new().expect("could not create file");
    file.write(input.as_bytes()).expect("could not initialize file");
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