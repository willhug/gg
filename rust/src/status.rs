use std::{fs::File, io::Read};



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

pub fn runstatus() {
    let status = parse_status_file("/home/will/status.txt".to_string()).unwrap();
    println!("{:?}", status);
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