use std::{error::Error, io, sync::mpsc, thread, time::Duration};
use octocrab::models::issues::Issue;
use tui::{Terminal, style::{Color, Style}, text::{Span, Spans}, widgets::ListItem};
use tui::backend::TermionBackend;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use termion::input::TermRead;
use tui::widgets::{Block, Borders, List};
use tui::layout::{Layout, Constraint, Direction};

use crate::issues::get_issues;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    #[allow(dead_code)]
    input_handle: thread::JoinHandle<()>,
    #[allow(dead_code)]
    tick_handle: thread::JoinHandle<()>,
}


impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if let Err(err) = tx.send(Event::Tick) {
                    eprintln!("{}", err);
                    break;
                }
                thread::sleep(Duration::from_millis(250));
            })
        };
        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}

struct App {
    issues: Vec<octocrab::models::issues::Issue>,
    selection: u64,
    // TODO
}

impl App {
    fn new(issues: Vec<Issue>) -> App {
        App {
            issues: issues,
            selection: 0,
        }
    }
 
    fn down(&mut self) {
        self.selection+=1;
    }

    fn up(&mut self) {
        self.selection-=1;
    }

    fn update(&mut self) {
    }
}

pub async fn start_terminal() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new();
    let issues = get_issues().await.unwrap();
    let mut app = App::new(issues);


    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ].as_ref())
                .split(f.size());
            let items: Vec<ListItem> = app.issues.iter().enumerate().map(|(idx, i)| {
                let mut style = Style::default();
                if idx as u64 == app.selection {
                    style = Style::default().bg(Color::LightGreen);
                }
                ListItem::new(Spans::from(vec![
                    Span::styled(i.html_url.as_str(), style.clone().fg(Color::Blue)),
                    Span::styled(" ", style),
                    Span::styled(i.title.as_str(), style),
                ]))
            }).collect();
            let block = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("ISSUES"));
            f.render_widget(block, chunks[0]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                match input {
                    Key::Char('q') => {
                        break;
                    },
                    Key::Char('j') | Key::Down => {
                        app.down();
                    },
                    Key::Char('k') | Key::Up => {
                        app.up();
                    },
                    _ => {
                        println!("Unknown input!");
                    }
                }
            }
            Event::Tick => {
                app.update();
            }
        }
    }

    Ok(())
}