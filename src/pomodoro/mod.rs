use ansi_escapes::Beep;

use std::{io::{Write, stdout}, thread, time};

use crossterm::{QueueableCommand, cursor};


pub fn run_pomodoro(duration_min: u32) {
    println!("Running Pomodoro for {} mins", duration_min);

    let mut stdout = stdout();
    for i in 0..duration_min {
        stdout.queue(cursor::SavePosition).unwrap();
        stdout.write_all(format!("{} minutes left", duration_min - i).as_bytes()).unwrap();
        stdout.flush().unwrap();
        thread::sleep(time::Duration::from_secs(60));
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();
    }
    // Beep 10 times
    for _ in 1..10 {
        print!("{}", Beep);
        thread::sleep(time::Duration::from_secs(1));
    }
}