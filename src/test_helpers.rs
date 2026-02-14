use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use tempfile::TempDir;

/// Global mutex to serialize tests that change CWD.
pub(crate) static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// RAII guard that saves CWD on creation and restores on Drop.
pub(crate) struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    pub(crate) fn new(new_dir: &Path) -> Self {
        let original = env::current_dir().expect("failed to get current dir");
        env::set_current_dir(new_dir).expect("failed to set current dir");
        CwdGuard { original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

/// A real git repo in a temp directory for testing.
pub(crate) struct TestRepo {
    pub(crate) dir: TempDir,
}

impl TestRepo {
    pub(crate) fn new() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path();

        // git init
        run_git_in(path, &["init"]);
        run_git_in(path, &["config", "user.name", "Test"]);
        run_git_in(path, &["config", "user.email", "test@test.com"]);

        // Write GG_CONFIG
        let git_dir = path.join(".git");
        let config_content = r#"{"repo_main_branch":"main","linked_issue":null,"branch_prefix":"test","repo_org":"testorg","branch_split":"/","merge_command":null,"merge_method":null}"#;
        fs::write(git_dir.join("GG_CONFIG"), config_content)
            .expect("failed to write GG_CONFIG");

        // Initial commit
        fs::write(path.join("README.md"), "# Test Repo\n")
            .expect("failed to write README");
        run_git_in(path, &["add", "README.md"]);
        run_git_in(path, &["commit", "-m", "Initial commit"]);

        // Rename branch to main
        run_git_in(path, &["branch", "-M", "main"]);

        TestRepo { dir }
    }

    pub(crate) fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Create a file, stage it, and commit.
    pub(crate) fn commit_file(&self, name: &str, content: &str) {
        fs::write(self.path().join(name), content).expect("failed to write file");
        run_git_in(self.path(), &["add", name]);
        run_git_in(
            self.path(),
            &["commit", "-m", &format!("Add {}", name)],
        );
    }

    /// Set up a bare clone as local origin.
    pub(crate) fn setup_local_origin(&self) {
        let origin_path = self.path().join(".test-origin.git");
        Command::new("git")
            .arg("clone")
            .arg("--bare")
            .arg(".")
            .arg(origin_path.to_str().unwrap())
            .current_dir(self.path())
            .output()
            .expect("failed to clone bare");
        run_git_in(
            self.path(),
            &["remote", "add", "origin", origin_path.to_str().unwrap()],
        );
    }

    /// Run a git command in the repo dir.
    pub(crate) fn run_git(&self, args: &[&str]) -> std::process::Output {
        run_git_in(self.path(), args)
    }
}

fn run_git_in(path: &Path, args: &[&str]) -> std::process::Output {
    let out = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .expect(&format!("failed to run git {:?}", args));
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        eprintln!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args, stdout, stderr
        );
    }
    out
}

/// Run a test closure with a TestRepo, holding the TEST_MUTEX and CwdGuard.
pub(crate) fn with_test_repo<F: FnOnce(&TestRepo)>(f: F) {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let repo = TestRepo::new();
    let _guard = CwdGuard::new(repo.path());
    f(&repo);
}

/// Set up a test repo for async tests. Returns RAII guards that must be kept alive.
/// Usage: `let (_lock, repo, _guard) = setup_test_repo();`
#[allow(dead_code)]
pub(crate) fn setup_test_repo() -> (
    std::sync::MutexGuard<'static, ()>,
    TestRepo,
    CwdGuard,
) {
    let lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let repo = TestRepo::new();
    let guard = CwdGuard::new(repo.path());
    (lock, repo, guard)
}

/// Mock `gh` CLI for testing the non-octo code paths.
pub(crate) struct MockGh {
    _dir: TempDir,
    original_path: String,
}

impl MockGh {
    fn new(dir: TempDir, original_path: String) -> Self {
        MockGh {
            _dir: dir,
            original_path,
        }
    }
}

impl Drop for MockGh {
    fn drop(&mut self) {
        env::set_var("PATH", &self.original_path);
    }
}

/// Fluent builder for MockGh dispatch entries.
pub(crate) struct GhMockBuilder {
    entries: Vec<(String, i32, String)>, // (args_pattern, exit_code, stdout)
}

impl GhMockBuilder {
    pub(crate) fn new() -> Self {
        GhMockBuilder {
            entries: Vec::new(),
        }
    }

    /// Add a mock: when `gh` args contain `pattern`, respond with `stdout` and `exit_code`.
    pub(crate) fn on_args_containing(
        mut self,
        pattern: &str,
        stdout: &str,
        exit_code: i32,
    ) -> Self {
        self.entries.push((
            pattern.to_string(),
            exit_code,
            stdout.to_string(),
        ));
        self
    }

    /// Build the MockGh, creating the fake `gh` script and prepending to PATH.
    pub(crate) fn build(self) -> MockGh {
        let dir = TempDir::new().expect("failed to create mock gh dir");
        let dispatch_path = dir.path().join("gh_dispatch");
        let gh_path = dir.path().join("gh");

        // Write dispatch file
        let mut dispatch_content = String::new();
        for (pattern, exit_code, stdout) in &self.entries {
            use std::io::Write;
            let encoded = base64_encode(stdout);
            dispatch_content.push_str(&format!("{}|{}|{}\n", pattern, exit_code, encoded));
        }
        fs::write(&dispatch_path, &dispatch_content).expect("failed to write dispatch");

        // Write the gh script
        let script = format!(
            r#"#!/bin/bash
DISPATCH="{}"
ALL_ARGS="$*"
while IFS='|' read -r pattern exit_code encoded_stdout; do
    if echo "$ALL_ARGS" | grep -q "$pattern"; then
        echo "$encoded_stdout" | base64 -d
        exit "$exit_code"
    fi
done < "$DISPATCH"
exit 0
"#,
            dispatch_path.to_str().unwrap()
        );
        fs::write(&gh_path, &script).expect("failed to write gh script");

        // Make executable
        Command::new("chmod")
            .arg("+x")
            .arg(gh_path.to_str().unwrap())
            .output()
            .expect("failed to chmod gh");

        // Prepend to PATH
        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", dir.path().to_str().unwrap(), original_path);
        env::set_var("PATH", &new_path);

        MockGh::new(dir, original_path)
    }
}

fn base64_encode(s: &str) -> String {
    use std::process::Stdio;
    let mut child = Command::new("base64")
        .arg("-w0") // no line wrapping
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn base64");
    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(s.as_bytes()).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .to_string()
}
