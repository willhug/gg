use std::error::Error;
use termion::event::Key;
use tui::{Frame, backend::Backend, layout::{Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block, Borders, Cell, Row, Table}};
use async_trait::async_trait;

use crate::{git, github::GithubRepo};

use super::{InputResult, app::App};


struct BranchWithInfo {
    branch: String,
    current: bool,
    pr: Option<octocrab::models::pulls::PullRequest>,
}

impl BranchWithInfo {
    fn to_row(&self, selected: bool) -> Row {
        let mut style = Style::default();
        if selected {
            style = Style::default().bg(Color::LightGreen);
        }
        Row::new(vec![
            match self.current {
                true => Cell::from("*").style(style.fg(Color::Red)),
                false => Cell::from(" "),
            },
            Cell::from(self.branch.as_str()).style(style.fg(Color::Blue)),
            match &self.pr {
                Some(pull) => Cell::from(pull.html_url.as_str()).style(style.fg(Color::LightBlue)),
                None => Cell::from("N/A").style(style.fg(Color::Red)),
            },
        ]).style(style)
    }
}

pub(super) struct PullApp {
    // TODO Not only pulls, include branches
    pulls: Vec<BranchWithInfo>,
    selection: usize,
    github: GithubRepo
}

impl PullApp {
    pub(super) async fn new(github: GithubRepo) -> PullApp {
        let mut p = PullApp {
            pulls: vec![],
            selection: 0,
            github,
        };
        p.load_branch_infos().await;
        p
    }

    async fn load_branch_infos(&mut self) {
        let branches = git::all_branches();
        let current_branch = git::current_branch();
        let mut branch_infos = vec![];
        for branch in branches {
            branch_infos.push(
                BranchWithInfo {
                    current: branch == current_branch,
                    pr: match self.github.pr_for_branch(&branch).await{
                        Ok(res) => res,
                        Err(_) => None,
                    },
                    branch,
                },
            );
        }
        self.pulls = branch_infos;
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
        self.load_branch_infos().await;
    }

    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(100),
            ].as_ref())
            .split(f.size());
        if self.pulls.is_empty() {
            f.render_widget(Block::default().borders(Borders::ALL).title("No Branches"), chunks[0]);
            return
        }
        let items: Vec<Row> = self.pulls.iter().enumerate().map(|(idx, i)| {
            i.to_row(idx == self.selection)
        }).collect();
        let block = Table::new(items)
            .widths(&[Constraint::Length(1), Constraint::Length(50), Constraint::Length(50)])
            .column_spacing(1)
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
            Key::Char('c') => {
                let selected_branch = &self.pulls[self.selection];
                git::checkout(&selected_branch.branch);
                self.update().await;
            },
            _ => {
                println!("Unknown input!");
            }
        }
        Ok(InputResult::Continue)
    }
}