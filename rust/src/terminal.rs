use std::{error::Error, io, sync::mpsc, thread, time::Duration};
use octocrab::models::issues::Issue;
use tui::{Terminal, style::{Color, Style}, text::{Span, Spans, Text}, widgets::{ListItem, Paragraph}};
use tui::backend::TermionBackend;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use termion::input::TermRead;
use tui::widgets::{Block, Borders, List};
use tui::layout::{Layout, Constraint, Direction};

use crate::{config, issues::{self, get_issues}};

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
                for evt in stdin.keys().into_iter().flatten() {
                    if let Err(err) = tx.send(Event::Input(evt)) {
                        eprintln!("{}", err);
                        return;
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
                thread::sleep(Duration::from_millis(5000));
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputState {
    Normal,
    Create,
}

struct App {
    issues: Vec<octocrab::models::issues::Issue>,
    selection: usize,
    input_state: InputState,
    buffered_issue_title: String,
    selected_issue: i64,
}

impl App {
    async fn new() -> App {
        App {
            issues: get_issues().await.unwrap(),
            selection: 0,
            input_state: InputState::Normal,
            buffered_issue_title: String::new(),
            selected_issue: 0,
        }
    }

    fn down(&mut self) {
        if self.selection < self.issues.len() - 1 {
            self.selection+=1;
        }
    }

    fn up(&mut self) {
        if self.selection > 0 {
            self.selection-=1;
        }
    }

    async fn update(&mut self) {
        let issues = get_issues().await.unwrap();
        self.issues = issues;
        if self.selection >= self.issues.len() {
            self.selection = self.issues.len() - 1
        }
        self.selected_issue = config::get_selected_issue_number();
    }

    fn get_selected(&mut self) -> &Issue {
        self.issues.get(self.selection).unwrap()
    }

    fn set_input_state(&mut self, state: InputState) {
        self.input_state = state;
        if state == InputState::Normal {
            self.buffered_issue_title.clear()
        }
    }
}

pub async fn start_terminal() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new();
    let mut app = App::new().await;


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
                if idx == app.selection {
                    style = Style::default().bg(Color::LightGreen);
                }
                let mut prefix = " ";
                if i.number == app.selected_issue {
                    prefix = "*";
                }
                ListItem::new(Spans::from(vec![
                    Span::styled(prefix, style.fg(Color::Red)),
                    Span::styled(i.html_url.as_str(), style.fg(Color::Blue)),
                    Span::styled(" ", style),
                    Span::styled(i.title.as_str(), style),
                ]))
            }).collect();
            let block = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("ISSUES"));
            f.render_widget(block, chunks[0]);
            if app.input_state == InputState::Create {
                let input = Paragraph::new(Text::from(Spans::from(app.buffered_issue_title.clone())))
                    .block(Block::default().borders(Borders::ALL).title("Create New Issue"));
                f.render_widget(input, chunks[1]);
            }
        })?;

        match events.next()? {
            Event::Input(input) => {
                match app.input_state {
                    InputState::Normal => match input {
                        Key::Esc | Key::Ctrl('c') | Key::Char('q') => {
                            break;
                        },
                        Key::Char('j') | Key::Down => {
                            app.down();
                        },
                        Key::Char('k') | Key::Up => {
                            app.up();
                        },
                        Key::Char('s') => {
                            let i = app.get_selected();
                            config::update_selected_issue(i.number);
                            app.selected_issue = i.number;
                        },
                        Key::Char('x') => {
                            config::update_selected_issue(0);
                        },
                        Key::Char('d') => {
                            let i = app.get_selected();
                            issues::close_issue(i.number).await?;
                            app.update().await;
                        },
                        Key::Char('c') => {
                            app.set_input_state(InputState::Create);
                        },
                        Key::Char('\n') => {
                            let i = app.get_selected();
                            open::that(&i.html_url.as_str()).unwrap();
                        },
                        _ => {
                            println!("Unknown input!");
                        }
                    },
                    InputState::Create => match input {
                        Key::Esc | Key::Ctrl('c') => {
                            app.set_input_state(InputState::Normal);
                        },
                        Key::Backspace => {
                            app.buffered_issue_title.pop();
                        },
                        Key::Char('\n') => {
                            issues::create_issue(app.buffered_issue_title.clone().as_str(), "").await?;
                            app.set_input_state(InputState::Normal);
                            app.update().await;
                        },
                        Key::Char(c) => {
                            app.buffered_issue_title.push(c);
                        }
                        _ => {},
                    },
                }
            }
            Event::Tick => {
                app.update().await;
            }
        }
    }

    Ok(())
}