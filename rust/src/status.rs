use std::{fs::{self, File, OpenOptions}, io::{Read, Write}};

use chrono::prelude::*;

use crate::config;



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
    repo: String,
}

#[derive(Debug, Clone)]
enum DayEventType {
    Issue,
    Todo,
}

pub fn list_statuses() {
    let cfg = config::get_full_config();
    let status = parse_status_file(cfg.status_file).unwrap();
    print!("{}", status_to_string(status));
}

pub fn write_status(body: String, todo: bool) {
    let cfg = config::get_full_config();
    let mut status = parse_status_file(cfg.status_file.clone()).unwrap();
    let today = today();
    if status.days.is_empty() || status.days[0].day != today {
        let day = Day {
            day: today.clone(),
            events: vec![],
        };
        let mut new_days = vec![day];
        new_days.append(&mut status.days);
        status.days = new_days;
    }
    let event = create_event(body, todo, cfg.repo_name);
    status.days[0].events.push(event);

    fs::create_dir_all(cfg.status_file_backup_dir.clone()).unwrap();

    fs::copy(
        cfg.status_file.clone(),
        format!("{}/{}_{}", cfg.status_file_backup_dir, today, Utc::now().timestamp()),
    ).unwrap();

    write_status_file(cfg.status_file, status).unwrap();
}

fn create_event(body: String, todo: bool, repo: String) -> DayEvent {
    let typ = match todo {
        true => DayEventType::Todo,
        false => DayEventType::Issue,
    };

    DayEvent{
        typ,
        info: body,
        repo,
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
            let repo_suffix = match event.repo.as_str() {
                "" => "".to_string(),
                _ => format!(" ({})", event.repo)
            };
            match event.typ {
                DayEventType::Issue => {
                    content.push_str(&format!("- {}{}\n", event.info, repo_suffix));
                },
                DayEventType::Todo => {
                    content.push_str(&format!("- TODO: {}{}\n", event.info, repo_suffix));
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
        } else if line.starts_with('[') && line.len() > 11 {
            let day_str = &line[1..11];
            if current_day.day.is_empty() {
                current_day.day = day_str.to_string();
            } else {
                status_file.days.push(current_day.clone());
                current_day.day = day_str.to_string();
                current_day.events = vec![];
            }
            continue;
        } else if let Some(stripped) = line.strip_prefix("- TODO: ") {
            let event = DayEvent{
                typ: DayEventType::Todo,
                info: stripped.to_string(),
                repo: "".to_string(),
            };
            current_day.events.push(event);
        } else if let Some(stripped) = line.strip_prefix("- ") {
            let event = DayEvent{
                typ: DayEventType::Issue,
                info: stripped.to_string(),
                repo: "".to_string(),
            };
            current_day.events.push(event);
        }
    }
    status_file.days.push(current_day);
    Ok(status_file)
}
