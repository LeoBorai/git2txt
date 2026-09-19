use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use tokio::fs::{read, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::debug;

/// The tool's own default output filename. Always excluded from enumeration so that a prior
/// run's output is never folded into a subsequent one.
const OUTPUT_FILENAME: &str = "output.txt";

pub struct Scanner {
    pub input_path: PathBuf,
    pub output_file: Arc<Mutex<File>>,
}

impl Scanner {
    pub fn new<P: AsRef<Path>>(input_path: P, output_file: Arc<Mutex<File>>) -> Self {
        Self {
            input_path: input_path.as_ref().to_path_buf(),
            output_file,
        }
    }

    pub async fn scan(&self) -> Result<()> {
        debug!(path=?self.input_path.display(), "Scanning...");

        let files = self.enumerate_files()?;

        for relative_path in files {
            let full_path = self.input_path.join(&relative_path);
            let contents = read(&full_path)
                .await
                .with_context(|| format!("Failed to read {}", full_path.display()))?;
            let mut file = self.output_file.lock().await;

            file.write_all(&contents).await?;
        }

        // tokio::fs::File writes are dispatched to a background blocking task and are not
        // guaranteed to be visible to a separate file handle (e.g. a later `copy`) until
        // flushed.
        self.output_file.lock().await.flush().await?;

        Ok(())
    }

    /// Lists files to include via `git ls-files --cached --others --exclude-standard`, which
    /// covers tracked files and untracked files not excluded by `.gitignore`, and excludes the
    /// tool's own output filename.
    fn enumerate_files(&self) -> Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.input_path)
            .arg("ls-files")
            .arg("--cached")
            .arg("--others")
            .arg("--exclude-standard")
            .output()
            .with_context(|| {
                format!(
                    "Failed to run git ls-files in {}",
                    self.input_path.display()
                )
            })?;

        if !output.status.success() {
            return Err(anyhow!(
                "git ls-files failed in {}: {}",
                self.input_path.display(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout =
            String::from_utf8(output.stdout).context("git ls-files output was not valid UTF-8")?;

        Ok(stdout
            .lines()
            .map(PathBuf::from)
            .filter(|path| path.as_os_str() != OUTPUT_FILENAME)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;
    use tokio::fs::OpenOptions;

    fn init_repo_with_files(files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new("git2txt-scanner-test").unwrap();

        let run = |args: &[&str]| {
            let status = Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(args)
                .status()
                .unwrap();
            assert!(status.success());
        };

        run(&["init", "--quiet"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "Test"]);

        for (name, contents) in files {
            fs::write(dir.path().join(name), contents).unwrap();
        }

        dir
    }

    async fn new_output_file() -> (TempDir, Arc<Mutex<File>>, PathBuf) {
        let out_dir = TempDir::new("git2txt-scanner-out").unwrap();
        let out_path = out_dir.path().join("aggregate.txt");

        File::create(&out_path).await.unwrap();
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&out_path)
            .await
            .unwrap();

        (out_dir, Arc::new(Mutex::new(file)), out_path)
    }

    #[tokio::test]
    async fn concatenates_tracked_and_untracked_non_ignored_files() {
        let repo = init_repo_with_files(&[
            (".gitignore", "ignored.txt\n"),
            ("a.txt", "AAA"),
            ("ignored.txt", "should not appear"),
        ]);

        Command::new("git")
            .arg("-C")
            .arg(repo.path())
            .args(["add", "a.txt", ".gitignore"])
            .status()
            .unwrap();
        Command::new("git")
            .arg("-C")
            .arg(repo.path())
            .args(["commit", "--quiet", "-m", "init"])
            .status()
            .unwrap();

        fs::write(repo.path().join("b.txt"), "BBB").unwrap();

        let (_out_dir, output_file, out_path) = new_output_file().await;
        let scanner = Scanner::new(repo.path(), output_file);
        scanner.scan().await.unwrap();

        let result = fs::read_to_string(out_path).unwrap();
        assert!(result.contains("AAA"));
        assert!(result.contains("BBB"));
        assert!(!result.contains("should not appear"));
    }

    #[tokio::test]
    async fn excludes_own_output_filename() {
        let repo = init_repo_with_files(&[(OUTPUT_FILENAME, "stale previous run")]);

        let (_out_dir, output_file, out_path) = new_output_file().await;
        let scanner = Scanner::new(repo.path(), output_file);
        scanner.scan().await.unwrap();

        let result = fs::read_to_string(out_path).unwrap();
        assert!(!result.contains("stale previous run"));
    }

    #[tokio::test]
    async fn fails_when_not_a_git_repository() {
        let dir = TempDir::new("git2txt-not-a-repo").unwrap();
        fs::write(dir.path().join("a.txt"), "AAA").unwrap();

        let (_out_dir, output_file, _out_path) = new_output_file().await;
        let scanner = Scanner::new(dir.path(), output_file);

        assert!(scanner.scan().await.is_err());
    }
}
