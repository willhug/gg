use std::error::Error;
use termion::event::Key;
use tui::{Frame, backend::Backend, layout::{Constraint, Direction, Layout}, style::{Color, Style}, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem}};
use async_trait::async_trait;

use crate::{git, github::GithubRepo};

use super::{InputResult, app::App};


struct BranchWithInfo {
    branch: String,
    current: bool,
    pr: Option<octocrab::models::pulls::PullRequest>,
}

pub(super) struct PullApp {
    // TODO Not only pulls, include branches
    pulls: Vec<BranchWithInfo>,
    selection: usize,
    github: GithubRepo
}

impl PullApp {
    pub(super) async fn new(github: GithubRepo) -> PullApp {
        let branches = git::all_branches();
        let current_branch = git::current_branch();
        let mut branch_infos = vec![];
        for branch in branches {
            branch_infos.push(
                BranchWithInfo {
                    current: branch == current_branch,
                    pr: match github.pr_for_branch(&branch).await{
                        Ok(res) => res,
                        Err(_) => None,
                    },
                    branch,
                },
            );
        }
        PullApp {
            pulls: branch_infos,
            selection: 0,
            github,
        }
    }

    fn down(&mut self) {
        if self.selection < self.pulls.len() - 1 {
            self.selection+=1;
        }
    }

    fn up(&mut self) {
        if self.selection > 0 {
            self.selection-=1;
        }
    }
}

#[async_trait]
impl App for PullApp {
    async fn update(&mut self) {
        // let issues = self.github.get_issues().await.unwrap();
        // self.issues = issues;
        if self.selection >= self.pulls.len() {
            self.selection = self.pulls.len() - 1
        }
    }

    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(100),
            ].as_ref())
            .split(f.size());
        let items: Vec<ListItem> = self.pulls.iter().enumerate().map(|(idx, i)| {
            let mut style = Style::default();
            if idx == self.selection {
                style = Style::default().bg(Color::LightGreen);
            }
            ListItem::new(Spans::from(vec![
                Span::styled(i.branch.as_str(), style.fg(Color::Blue)),
            ]))
        }).collect();
        let block = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Branches"));
        f.render_widget(block, chunks[0]);
    }

    async fn handle_input(&mut self, input: Key) -> Result<InputResult, Box<dyn Error>>{
        match input {
            Key::Esc | Key::Ctrl('c') | Key::Char('q') => {
                return Ok(InputResult::Exit);
            },
            Key::Char('j') | Key::Down => {
                self.down();
            },
            Key::Char('k') | Key::Up => {
                self.up();
            },
            _ => {
                println!("Unknown input!");
            }
        }
        Ok(InputResult::Continue)
    }
}