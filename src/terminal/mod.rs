mod app;
pub mod branches;

use async_trait::async_trait;
#[cfg(feature = "octo")]
use octocrab::models::issues::Issue;
use std::{error::Error, io, sync::mpsc, thread, time::Duration};
use termion::input::TermRead;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List};
use tui::{
    backend::Backend,
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{ListItem, Paragraph},
    Frame, Terminal,
};

use crate::{
    config::{self, get_full_config},
    github::GithubRepo,
};

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
    #[cfg(feature = "octo")]
    issues: Vec<octocrab::models::issues::Issue>,
    #[cfg(not(feature = "octo"))]
    issues: Vec<serde_json::Value>,
    selection: usize,
    input_state: InputState,
    buffered_issue_title: String,
    selected_issue: u64,
    github: GithubRepo,
}

impl IssueViewApp {
    async fn new(github: GithubRepo) -> IssueViewApp {
        let mut app = IssueViewApp {
            issues: vec![],
            selection: 0,
            input_state: InputState::Normal,
            buffered_issue_title: "".to_string(),
            selected_issue: config::get_selected_issue_number(),
            github,
        };
        app.update().await;
        app
    }

    #[cfg(feature = "octo")]
    fn get_selected(&self) -> &octocrab::models::issues::Issue {
        &self.issues[self.selection]
    }

    #[cfg(not(feature = "octo"))]
    fn get_selected(&self) -> &serde_json::Value {
        &self.issues[self.selection]
    }

    fn set_input_state(&mut self, state: InputState) {
        self.input_state = state;
        if state == InputState::Create {
            self.buffered_issue_title = "".to_string();
        }
    }

    fn selected(&self) -> usize {
        if self.issues.is_empty() {
            return 0;
        }
        self.selection % self.issues.len()
    }

    fn up(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        if self.selection == 0 {
            self.selection = self.issues.len() - 1;
        } else {
            self.selection -= 1;
        }
    }

    fn down(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        self.selection = (self.selection + 1) % self.issues.len();
    }
}

#[async_trait]
impl<'a> App for IssueViewApp {
    async fn update(&mut self) {
        self.issues = match self.github.get_issues().await {
            Ok(issues) => issues,
            Err(_) => vec![],
        };
        if self.selection >= self.issues.len() {
            self.selection = if self.issues.is_empty() {
                0
            } else {
                self.issues.len() - 1
            };
        }
        self.selected_issue = config::get_selected_issue_number();
    }

    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());
        let items: Vec<ListItem> = self
            .issues
            .iter()
            .enumerate()
            .map(|(idx, i)| {
                let mut style = Style::default();
                if idx == self.selection {
                    style = Style::default().bg(Color::LightGreen);
                }
                let mut prefix = " ";

                #[cfg(feature = "octo")]
                let (url, title, number) = (i.html_url.as_str(), i.title.as_str(), i.number);
                #[cfg(not(feature = "octo"))]
                let (url, title, number) = (
                    i["html_url"].as_str().unwrap_or(""),
                    i["title"].as_str().unwrap_or("Untitled"),
                    i["number"].as_u64().unwrap_or(0),
                );

                if number == self.selected_issue {
                    prefix = "*";
                }
                ListItem::new(Spans::from(vec![
                    Span::styled(prefix, style.fg(Color::Red)),
                    Span::styled(url, style.fg(Color::Blue)),
                    Span::styled(" ", style),
                    Span::styled(title, style),
                ]))
            })
            .collect();
        let block = List::new(items).block(Block::default().borders(Borders::ALL).title("ISSUES"));
        f.render_widget(block, chunks[0]);
        if self.input_state == InputState::Create {
            let input = Paragraph::new(Text::from(Spans::from(self.buffered_issue_title.clone())))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Create New Issue"),
                );
            f.render_widget(input, chunks[1]);
        }
    }

    async fn handle_input(&mut self, input: Key) -> Result<InputResult, Box<dyn Error>> {
        match self.input_state {
            InputState::Normal => match input {
                Key::Esc | Key::Ctrl('c') | Key::Char('q') => {
                    return Ok(InputResult::Exit);
                }
                Key::Char('j') | Key::Down => {
                    self.down();
                }
                Key::Char('k') | Key::Up => {
                    self.up();
                }
                Key::Char('s') => {
                    let i = self.get_selected();
                    #[cfg(feature = "octo")]
                    let number = i.number;
                    #[cfg(not(feature = "octo"))]
                    let number = i["number"].as_u64().unwrap_or(0);

                    config::update_selected_issue(number);
                    self.selected_issue = number;
                }
                Key::Char('x') => {
                    config::update_selected_issue(0);
                }
                Key::Char('d') => {
                    let i = self.get_selected();
                    #[cfg(feature = "octo")]
                    let number = i.number;
                    #[cfg(not(feature = "octo"))]
                    let number = i["number"].as_u64().unwrap_or(0);

                    match self.github.close_issue(number).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Failed to close issue: {}", e);
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to close issue",
                            )));
                        }
                    }
                    self.update().await;
                }
                Key::Char('c') => {
                    self.set_input_state(InputState::Create);
                }
                Key::Char('\n') => {
                    let i = self.get_selected();
                    #[cfg(feature = "octo")]
                    let url = i.html_url.as_str();
                    #[cfg(not(feature = "octo"))]
                    let url = i["html_url"].as_str().unwrap_or("");

                    open::that(url).unwrap();
                }
                _ => {
                    println!("Unknown input!");
                }
            },
            InputState::Create => match input {
                Key::Esc | Key::Ctrl('c') => {
                    self.set_input_state(InputState::Normal);
                }
                Key::Backspace => {
                    self.buffered_issue_title.pop();
                }
                Key::Char('\n') => {
                    match self
                        .github
                        .create_issue(self.buffered_issue_title.clone().as_str(), "")
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Failed to create issue: {}", e);
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to create issue",
                            )));
                        }
                    }
                    self.set_input_state(InputState::Normal);
                    self.update().await;
                }
                Key::Char(c) => {
                    self.buffered_issue_title.push(c);
                }
                _ => {}
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
        }
        View::Pulls => {
            run_loop(PullApp::new(github).await).await?;
        }
    };

    Ok(())
}

async fn run_loop<T: App>(mut app: T) -> Result<(), Box<dyn Error>> {
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
                    InputResult::Continue => {}
                };
            }
            Event::Tick => {
                app.update().await;
            }
        }
    }
    Ok(())
}
