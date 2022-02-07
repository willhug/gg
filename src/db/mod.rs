use std::{fs, path::{Path, PathBuf}};

use dirs::home_dir;
use rusqlite::Connection;
use tui::widgets::LineGauge;


pub fn init_connection() -> Result<Connection> {
    fs::create_dir_all(get_gg_dir_path())?;
    let conn = Connection::open(get_db_file_path())?;
    Workspace::init(conn);
    Repo::init(conn);
    LinkedIssue::init(conn);
    Ok(conn)
}

fn get_db_file_path() -> PathBuf {
    get_gg_dir_path().join("db.sqlite")
}

fn get_gg_dir_path() -> PathBuf {
    Path::new(&dirs::home_dir().unwrap()).join(".gg")
}

struct Workspace {
    github_user: String,
    github_token: String,
}

impl Workspace {
    fn init(conn: Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS workspace (
                github_user  TEXT NOT NULL,
                github_token TEXT NOT NULL
            )",
            [],
        )?;
    }
}

struct Repo {
    local_path: String,
    repo_org: String,
    repo_name: String,
    main_branch: String,
    branch_prefix: String,
}

impl Repo {
    fn init(conn: Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS repo (
                local_path TEXT PRIMARY KEY,
                org TEXT NOT NULL,
                name TEXT NOT NULL,
                main_branch TEXT NOT NULL,
                branch_prefix TEXT NOT NULL,
            )",
            [],
        )?;
    }
}

struct LinkedIssue {
    repo_name: String,
    issue: u32,
    branch: String,
}

impl LinkedIssue {
    fn init(conn: Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS linked_issue (
                repo_name TEXT NOT NULL,
                issue INTEGER,
                branch TEXT NOT NULL,
            )",
            [],
        )?;
    }
}