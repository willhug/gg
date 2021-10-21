use std::{fs::{self, File, OpenOptions}, io::{Read, Write}};

use chrono::prelude::*;



#[derive(Debug)]
struct StatusFile {
    days: Vec<Day>
}

#[derive(Debug, Clone)]
struct Day {
    day: String,
    events: Vec<DayEvent>,
}

#[derive(Debug, Clone)]
struct DayEvent {
    typ: DayEventType,
    info: String,
}

#[derive(Debug, Clone)]
enum DayEventType {
    Issue,
    TODO,
}

pub fn list_statuses() {
    let status = parse_status_file("/home/will/status.txt".to_string()).unwrap();
    print!("{}", status_to_string(status));
}

pub fn write_status(body: String, todo: bool) {
    let mut status = parse_status_file("/home/will/status.txt".to_string()).unwrap();
    let today = today();
    if status.days.len() == 0 || status.days[0].day != today {
        let day = Day {
            day: today.clone(),
            events: vec![],
        };
        let mut new_days = vec![day];
        new_days.append(&mut status.days);
        status.days = new_days;
    }
    let event = create_event(body, todo);
    status.days[0].events.push(event);

    fs::create_dir_all("/home/will/status_bu").unwrap();

    fs::copy(
        "/home/will/status.txt",
        format!("/home/will/status_bu/{}_{}", today, Utc::now().timestamp()),
    ).unwrap();

    write_status_file("/home/will/status.txt".to_string(), status).unwrap();
}

fn create_event(body: String, todo: bool) -> DayEvent {
    let typ = match todo {
        true => DayEventType::TODO,
        false => DayEventType::Issue,
    };

    DayEvent{
        typ,
        info: body,
    }
}

fn today() -> String {
    let local = Local::now();
    format!("{}-{}-{}", local.year(), local.month(), local.day())
}

fn write_status_file(filepath: String, status_file: StatusFile) -> Result<(), anyhow::Error> {
    let status_str = status_to_string(status_file);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filepath)
        .map_err(anyhow::Error::msg)?;

    file.write_all(status_str.as_bytes()).map_err(anyhow::Error::msg)?;
    file.sync_all().map_err(anyhow::Error::msg)
}

fn status_to_string(status_file: StatusFile) -> String {
    let mut content = String::new();
    for day in status_file.days {
        content.push_str(&format!("[{}]\n", day.day));
        for event in day.events {
            match event.typ {
                DayEventType::Issue => {
                    content.push_str(&format!("- {}\n", event.info));
                },
                DayEventType::TODO => {
                    content.push_str(&format!("- TODO: {}\n", event.info));
                },

            }
        }
        content.push('\n');
    }
    content
}


fn parse_status_file(filename: String) -> Result<StatusFile, anyhow::Error> {
    let mut file = File::open(filename).map_err(anyhow::Error::msg)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(anyhow::Error::msg)?;

    let mut status_file = StatusFile{
        days: vec![],
    };
    let mut current_day = Day{
        day: "".to_string(),
        events: vec![],
    };
    for line in buf.lines() {
        if line.trim() == "" {
            continue;
        } else if line.starts_with("[") && line.len() > 11 {
            let day_str = &line[1..11];
            if current_day.day == "" {
                current_day.day = day_str.to_string();
            } else {
                status_file.days.push(current_day.clone());
                current_day.day = day_str.to_string();
                current_day.events = vec![];
            }
            continue;
        } else if line.starts_with("- TODO: ") {
            let event = DayEvent{
                typ: DayEventType::TODO,
                info: line[8..].to_string(),
            };
            current_day.events.push(event);
        } else if line.starts_with("- ") {
            let event = DayEvent{
                typ: DayEventType::Issue,
                info: line[2..].to_string(),
            };
            current_day.events.push(event);
        }
    }
    status_file.days.push(current_day);
    Ok(status_file)
}
