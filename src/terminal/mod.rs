pub mod branches;
mod app;

use async_trait::async_trait;
use std::{error::Error, io, sync::mpsc, thread, time::Duration};
use octocrab::models::issues::Issue;
use tui::{Frame, Terminal, backend::Backend, style::{Color, Style}, text::{Span, Spans, Text}, widgets::{ListItem, Paragraph}};
use tui::backend::TermionBackend;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use termion::input::TermRead;
use tui::widgets::{Block, Borders, List};
use tui::layout::{Layout, Constraint, Direction};

use crate::{config::{self, get_full_config}, github::GithubRepo};

use self::{app::App, branches::PullApp};

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
enum View {
    Issues,
    Pulls,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputState {
    Normal,
    Create,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputResult {
    Continue,
    Exit,
}


struct IssueViewApp {
    issues: Vec<octocrab::models::issues::Issue>,
    selection: usize,
    input_state: InputState,
    buffered_issue_title: String,
    selected_issue: i64,
    github: GithubRepo
}

impl IssueViewApp {
    async fn new(github: GithubRepo) -> IssueViewApp {
        IssueViewApp {
            issues: github.get_issues().await.unwrap(),
            selection: 0,
            input_state: InputState::Normal,
            buffered_issue_title: String::new(),
            selected_issue: 0,
            github,
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

    fn get_selected(&self) -> &Issue {
        self.issues.get(self.selection).unwrap()
    }

    fn set_input_state(&mut self, state: InputState) {
        self.input_state = state;
        if state == InputState::Normal {
            self.buffered_issue_title.clear()
        }
    }
}

#[async_trait]
impl App for IssueViewApp {
    async fn update(&mut self) {
        let issues = self.github.get_issues().await.unwrap();
        self.issues = issues;
        if self.selection >= self.issues.len() {
            self.selection = self.issues.len() - 1
        }
        self.selected_issue = config::get_selected_issue_number();
    }


    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(f.size());
        let items: Vec<ListItem> = self.issues.iter().enumerate().map(|(idx, i)| {
            let mut style = Style::default();
            if idx == self.selection {
                style = Style::default().bg(Color::LightGreen);
            }
            let mut prefix = " ";
            if i.number == self.selected_issue {
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
        if self.input_state == InputState::Create {
            let input = Paragraph::new(Text::from(Spans::from(self.buffered_issue_title.clone())))
                .block(Block::default().borders(Borders::ALL).title("Create New Issue"));
            f.render_widget(input, chunks[1]);
        }
    }

    async fn handle_input(&mut self, input: Key) -> Result<InputResult, Box<dyn Error>>{
        match self.input_state {
            InputState::Normal => match input {
                Key::Esc | Key::Ctrl('c') | Key::Char('q') => {
                    return Ok(InputResult::Exit);
                },
                Key::Char('j') | Key::Down => {
                    self.down();
                },
                Key::Char('k') | Key::Up => {
                    self.up();
                },
                Key::Char('s') => {
                    let i = self.get_selected();
                    config::update_selected_issue(i.number);
                    self.selected_issue = i.number;
                },
                Key::Char('x') => {
                    config::update_selected_issue(0);
                },
                Key::Char('d') => {
                    let i = self.get_selected();
                    self.github.close_issue(i.number).await?;
                    self.update().await;
                },
                Key::Char('c') => {
                    self.set_input_state(InputState::Create);
                },
                Key::Char('\n') => {
                    let i = self.get_selected();
                    open::that(&i.html_url.as_str()).unwrap();
                },
                _ => {
                    println!("Unknown input!");
                }
            },
            InputState::Create => match input {
                Key::Esc | Key::Ctrl('c') => {
                    self.set_input_state(InputState::Normal);
                },
                Key::Backspace => {
                    self.buffered_issue_title.pop();
                },
                Key::Char('\n') => {
                    self.github.create_issue(self.buffered_issue_title.clone().as_str(), "").await?;
                    self.set_input_state(InputState::Normal);
                    self.update().await;
                },
                Key::Char(c) => {
                    self.buffered_issue_title.push(c);
                }
                _ => {},
            },
        }
        Ok(InputResult::Continue) // TODO ENUM
    }
}

pub async fn start_terminal() -> Result<(), Box<dyn Error>> {
    start_terminal_with_opts(View::Issues).await
}

pub async fn start_pr_terminal() -> Result<(), Box<dyn Error>> {
    start_terminal_with_opts(View::Pulls).await
}

async fn start_terminal_with_opts(view: View) -> Result<(), Box<dyn Error>> {
    let github = GithubRepo::new(get_full_config()).await;
    match view {
        View::Issues => {
            run_loop(IssueViewApp::new(github).await).await?;
        },
        View::Pulls => {
            run_loop(PullApp::new(github).await).await?;
        },
    };

    Ok(())
}

async fn run_loop<T: App>(mut app: T) -> Result<(), Box<dyn Error>>{
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new();


    loop {
        terminal.draw(|f| {
            app.draw(f);
        })?;

        match events.next()? {
            Event::Input(input) => {
                match app.handle_input(input).await? {
                    InputResult::Exit => break,
                    InputResult::Continue => {},
                };
            }
            Event::Tick => {
                app.update().await;
            }
        }
    }
    Ok(())
}
