use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use clap::Parser;

use git2txt::{git::Git, scanner::Scanner};
use tempdir::TempDir;
use tokio::fs::{copy, File, OpenOptions};
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

const AGGREGATE_FILENAME: &str = "aggregate.txt";

#[derive(Debug, Parser)]
#[command(
    name = "git2txt",
    about = "Download Git Repos into TXT Files",
    author = "Esteban Borai <estebanborai@gmail.com> (https://github.com/EstebanBorai/git2txt)",
    next_line_help = true
)]
pub struct Cli {
    /// Remote Git URL to clone, or a local path to an existing Git repository.
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    pub source: String,
}

#[derive(Debug, PartialEq, Eq)]
enum SourceKind {
    Remote(String),
    Local(PathBuf),
}

fn classify_source(source: &str) -> Result<SourceKind> {
    if looks_like_remote_url(source) {
        return Ok(SourceKind::Remote(source.to_string()));
    }

    let path = PathBuf::from(source);

    if path.exists() {
        return Ok(SourceKind::Local(path));
    }

    bail!("`{source}` is not a recognized Git URL or an existing local path")
}

fn looks_like_remote_url(source: &str) -> bool {
    source.starts_with("https://")
        || source.starts_with("http://")
        || source.starts_with("ssh://")
        || is_scp_like_url(source)
}

/// Matches SCP-like SSH URLs such as `git@github.com:owner/repo.git`.
fn is_scp_like_url(source: &str) -> bool {
    let Some((user_host, path)) = source.split_once(':') else {
        return false;
    };

    let Some((_user, host)) = user_host.split_once('@') else {
        return false;
    };

    !host.is_empty() && !host.contains('/') && !path.is_empty()
}

fn is_git_repository(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .with_context(|| format!("Failed to run git in {}", path.display()))?;

    Ok(output.status.success())
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter_layer)
        .init();

    let args = Cli::parse();
    let source_kind = classify_source(&args.source)?;

    // Keeps the temporary clone alive for a remote source until scanning is done.
    let mut _clone_dir: Option<TempDir> = None;

    let scan_dir = match source_kind {
        SourceKind::Remote(url) => {
            let outdir = TempDir::new("git2txt-temp")?;
            let git = Git::new(url, outdir.path());

            git.download().await?;

            let path = outdir.path().to_path_buf();
            _clone_dir = Some(outdir);
            path
        }
        SourceKind::Local(path) => {
            if !is_git_repository(&path)? {
                bail!("{} is not a Git repository", path.display());
            }

            path
        }
    };

    let aggregate_dir = TempDir::new("git2txt-output")?;
    let aggregate_file_path = aggregate_dir.path().join(AGGREGATE_FILENAME);

    File::create(&aggregate_file_path).await?;
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&aggregate_file_path)
        .await?;
    let file = Arc::new(Mutex::new(file));

    let scanner = Scanner::new(&scan_dir, Arc::clone(&file));
    scanner.scan().await?;

    copy(&aggregate_file_path, current_dir()?.join("output.txt")).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_https_url_as_remote() {
        assert_eq!(
            classify_source("https://github.com/EstebanBorai/git2txt.git").unwrap(),
            SourceKind::Remote("https://github.com/EstebanBorai/git2txt.git".to_string())
        );
    }

    #[test]
    fn detects_scp_like_url_as_remote() {
        assert_eq!(
            classify_source("git@github.com:EstebanBorai/git2txt.git").unwrap(),
            SourceKind::Remote("git@github.com:EstebanBorai/git2txt.git".to_string())
        );
    }

    #[test]
    fn detects_existing_relative_path_as_local() {
        assert_eq!(
            classify_source(".").unwrap(),
            SourceKind::Local(PathBuf::from("."))
        );
    }

    #[test]
    fn detects_existing_absolute_path_as_local() {
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(
            classify_source(cwd.to_str().unwrap()).unwrap(),
            SourceKind::Local(cwd)
        );
    }

    #[test]
    fn rejects_source_that_is_neither_url_nor_existing_path() {
        assert!(classify_source("not-a-real-path-or-url").is_err());
    }
}
