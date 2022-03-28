use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use chrono::prelude::*;

use crate::config;

#[derive(Debug)]
struct StatusFile {
    days: Vec<Day>,
}

#[derive(Debug, Clone)]
struct Day {
    day: NaiveDate,
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
    println!("opening status file: {:?}", cfg.status_file);
    let status = parse_status_file(cfg.status_file).unwrap();
    print!("{}", status_to_string(status));
}

pub fn write_status(body: String, todo: bool) {
    let cfg = config::get_full_config();
    let mut status = parse_status_file(cfg.status_file.clone()).unwrap();
    let today = today();
    if status.days.is_empty() || status.days[0].day != today {
        let day = Day {
            day: today,
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
        cfg.status_file_backup_dir.join(format!(
            "{}_{}",
            today.format("%Y-%m-%d"),
            Utc::now().timestamp()
        )),
    )
    .unwrap();

    write_status_file(cfg.status_file, status).unwrap();
}

fn create_event(body: String, todo: bool, repo: String) -> DayEvent {
    let typ = match todo {
        true => DayEventType::Todo,
        false => DayEventType::Issue,
    };

    DayEvent {
        typ,
        info: body,
        repo,
    }
}

fn today() -> NaiveDate {
    let local = Local::now();
    local.naive_local().date()
}

fn write_status_file(filepath: PathBuf, status_file: StatusFile) -> Result<(), anyhow::Error> {
    let status_str = status_to_string(status_file);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filepath)
        .map_err(anyhow::Error::msg)?;

    file.write_all(status_str.as_bytes())
        .map_err(anyhow::Error::msg)?;
    file.sync_all().map_err(anyhow::Error::msg)
}

fn status_to_string(status_file: StatusFile) -> String {
    let mut content = String::new();
    for day in status_file.days {
        content.push_str(&format!("[{}]\n", day.day.format("%Y-%m-%d")));
        for event in day.events {
            let repo_suffix = match event.repo.as_str() {
                "" => "".to_string(),
                _ => format!(" ({})", event.repo),
            };
            match event.typ {
                DayEventType::Issue => {
                    content.push_str(&format!("- {}{}\n", event.info, repo_suffix));
                }
                DayEventType::Todo => {
                    content.push_str(&format!("- TODO: {}{}\n", event.info, repo_suffix));
                }
            }
        }
        content.push('\n');
    }
    content
}

fn parse_status_file(filename: PathBuf) -> Result<StatusFile, anyhow::Error> {
    let mut file = File::open(filename).map_err(anyhow::Error::msg)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(anyhow::Error::msg)?;

    let mut status_file = StatusFile { days: vec![] };
    let mut initialized = false;
    let mut current_day = Day {
        day: NaiveDate::from_ymd(0, 1, 1),
        events: vec![],
    };
    for line in buf.lines() {
        if line.trim() == "" {
            continue;
        }
        if let Some(day) = parse_day_from_line(line) {
            if !initialized {
                initialized = true;
            } else {
                status_file.days.push(current_day.clone());
            }
            current_day = Day {
                day,
                events: vec![],
            };
            continue;
        }
        if let Some(event) = parse_event_from_line(line) {
            current_day.events.push(event);
        }
    }
    status_file.days.push(current_day);
    Ok(status_file)
}

fn parse_day_from_line(line: &str) -> Option<NaiveDate> {
    if !line.starts_with('[') {
        return None;
    }
    let t_line = line.trim();
    let date = NaiveDate::parse_from_str(t_line, "[%Y-%m-%d]").unwrap();
    Some(date)
}

fn parse_event_from_line(line: &str) -> Option<DayEvent> {
    if let Some(stripped) = line.strip_prefix("- TODO: ") {
        return Some(DayEvent {
            typ: DayEventType::Todo,
            info: stripped.to_string(),
            repo: "".to_string(),
        });
    }
    if let Some(stripped) = line.strip_prefix("- ") {
        return Some(DayEvent {
            typ: DayEventType::Issue,
            info: stripped.to_string(),
            repo: "".to_string(),
        });
    }
    None
}
