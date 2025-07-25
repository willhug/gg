use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};
use termion::event::Key;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::{
    git::{self, is_start_branch, parse_branch},
    github::{pr::Pr, GithubRepo},
};

use super::{app::App, InputResult};

pub(crate) struct BranchWithInfo {
    pub(crate) branch: String,
    pub(crate) date_created: i64,
    pub(crate) current: bool,
    pub(crate) has_start: bool,
    pub(crate) pr: Option<Pr>,
}

static WIDTHS: &[tui::layout::Constraint; 4] = &[
    Constraint::Length(1),
    Constraint::Length(1),
    Constraint::Length(50),
    Constraint::Length(50),
];

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
            match self.has_start {
                true => Cell::from("!").style(style.fg(Color::Green)),
                false => Cell::from("?").style(style.fg(Color::Red)),
            },
            Cell::from(self.branch.as_str()).style(style.fg(Color::Blue)),
            match &self.pr {
                Some(pull) => {
                    let mut col = Color::LightBlue;
                    if pull.closed {
                        col = Color::LightRed;
                    }
                    Cell::from(pull.url.as_str()).style(style.fg(col))
                }
                None => Cell::from("N/A").style(style.fg(Color::Red)),
            },
        ])
        .style(style)
    }

    pub(crate) fn set_pr(&mut self, pr: Option<Pr>) {
        self.pr = pr;
    }
}

// TODO Move
pub(crate) async fn load_branch_infos(github: &GithubRepo) -> Vec<BranchWithInfo> {
    let branches = git::all_branch_infos();
    let mut br_map = HashMap::new();
    let mut br_set = HashSet::new();
    let current_branch = git::current_branch();
    for branch in &branches {
        br_map.insert(branch.name.clone(), branch);
        br_set.insert(branch.name.clone());
    }
    let prs = github.prs_for_branches(&br_set).await.unwrap();
    let mut pr_map = HashMap::new();
    for pr in prs {
        pr_map.insert(pr.branch.clone(), pr);
    }
    let mut branch_infos: Vec<BranchWithInfo> = branches.clone()
        .into_iter()
        .filter(|x| !is_start_branch(&x.name))
        .map(|branch| {
            let parsed_br = parse_branch(branch.name.clone());
            BranchWithInfo {
                date_created: branch.date_created,
                current: branch.name == current_branch,
                pr: None,
                has_start: br_map.contains_key(&parsed_br.start()),
                branch: branch.name,
            }
        })
        .collect();
    for branch_info in branch_infos.iter_mut() {
        let pr = pr_map.remove(&branch_info.branch);
        branch_info.set_pr(pr);
    }
    branch_infos
}

pub(super) struct PullApp {
    // TODO Not only pulls, include branches
    pulls: Vec<BranchWithInfo>,
    selection: usize,
    github: GithubRepo,
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
        self.pulls = load_branch_infos(&self.github).await;
    }

    fn down(&mut self) {
        if self.selection < self.pulls.len() - 1 {
            self.selection += 1;
        }
    }

    fn up(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        }
    }
}

#[async_trait]
impl App for PullApp {
    async fn update(&mut self) {
        if self.selection >= self.pulls.len() {
            self.selection = self.pulls.len() - 1
        }
        self.load_branch_infos().await;
    }

    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());
        if self.pulls.is_empty() {
            f.render_widget(
                Block::default().borders(Borders::ALL).title("No Branches"),
                chunks[0],
            );
            return;
        }
        let items: Vec<Row> = self
            .pulls
            .iter()
            .enumerate()
            .map(|(idx, i)| i.to_row(idx == self.selection))
            .collect();
        let block = Table::new(items)
            .widths(WIDTHS)
            .column_spacing(1)
            .block(Block::default().borders(Borders::ALL).title("Branches"));
        f.render_widget(block, chunks[0]);
    }

    async fn handle_input(&mut self, input: Key) -> Result<InputResult, Box<dyn Error>> {
        match input {
            Key::Esc | Key::Ctrl('c') | Key::Char('q') => {
                return Ok(InputResult::Exit);
            }
            Key::Char('d') => {
                let selected_branch = &self.pulls[self.selection];
                git::delete_branch_all(selected_branch.branch.clone());
                if selected_branch.has_start {
                    git::delete_branch_all(
                        git::parse_branch(selected_branch.branch.clone()).start(),
                    );
                }
                self.update().await;
            }
            Key::Char('j') | Key::Down => {
                self.down();
            }
            Key::Char('k') | Key::Up => {
                self.up();
            }
            Key::Char('c') => {
                let selected_branch = &self.pulls[self.selection];
                git::checkout(&selected_branch.branch);
                self.update().await;
            }
            Key::Char('\n') => {
                let i = &self.pulls[self.selection];
                match &i.pr {
                    Some(pr) => {
                        open::that(&pr.url.as_str()).unwrap();
                    }
                    None => {}
                }
            }
            _ => {
                println!("Unknown input!");
            }
        }
        Ok(InputResult::Continue)
    }
}
