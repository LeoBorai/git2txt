use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use futures::{future::BoxFuture, FutureExt};
use tokio::fs::{read, read_dir, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::debug;

const SKIP_DIRECTORIES: &[&str] = &["node_modules", ".git"];

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

    pub fn scan(&self) -> BoxFuture<'_, Result<()>> {
        debug!(path=?self.input_path.display(), "Scanning...");

        async move {
            let mut entries = read_dir(&self.input_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let entry_name = entry.file_name();

                if entry_path.is_dir() {
                    if SKIP_DIRECTORIES.contains(
                        &entry_name
                            .to_str()
                            .context("Failed to convert entry name to string")?,
                    ) {
                        continue;
                    }

                    let scanner = Scanner::new(entry_path, self.output_file.clone());
                    scanner.scan().await?;
                } else {
                    let contents = read(entry_path).await?;
                    let mut file = self.output_file.lock().await;

                    file.write_all(&contents).await?;
                }
            }

            Ok(())
        }
        .boxed()
    }
}
