# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`git2txt` — Rust CLI that clones a Git repository and flattens all its files into a single
aggregate `output.txt`, for feeding a repo into an LLM.

## Commands

```bash
cargo build                          # build
cargo run -- <GIT_REPO_URL>          # clone repo, produce ./output.txt in cwd
RUST_LOG=debug cargo run -- <URL>    # run with debug logging
cargo test                           # run tests
cargo clippy                         # lint (CI: clippy.yml, runs on ubuntu/windows/macOS)
cargo fmt --all -- --check           # format check (CI: fmt.yml)
just fmt                             # `cargo clippy --fix` + `cargo fmt` locally
```

There is no test suite in `src/` currently — CI's `test.yml` runs `cargo test` but there are no
`#[test]` functions yet.

## Architecture

Library/binary split (`src/lib.rs` exports `git` and `scanner`; the binary is `src/bin/main.rs`):

- **`git::Git`** (`src/git.rs`) — shells out to `git clone --depth=1 <url> <outdir>` via
  `std::process::Command`. Not a `git2`/libgit2 binding.
- **`scanner::Scanner`** (`src/scanner.rs`) — recursively walks a directory tree, appending every
  file's raw bytes into one shared, mutex-guarded output `File`. Recurses by constructing a new
  `Scanner` per subdirectory sharing the same `Arc<Mutex<File>>` sink. Skips directories listed in
  `SKIP_DIRECTORIES` (currently `node_modules`, `.git`) — add new ignore entries here.
- **`main.rs`** wires them together: clones the target repo into a `tempdir::TempDir`, scans the
  clone into `<tempdir>/output/aggregate.txt`, then copies that to `./output.txt` in the caller's
  current directory. The temp dir is cleaned up automatically when it drops.

Async runtime is Tokio; directory recursion returns a boxed future (`BoxFuture`, via the `futures`
crate) since async fns can't be directly recursive.

## OpenSpec workflow

This repo uses OpenSpec (see `openspec/` and the `openspec-*` / `opsx:*` skills) for proposing and
tracking spec-driven changes. Active change proposals live under `openspec/changes/`; completed
ones are archived under `openspec/changes/archive/`. Use the `openspec-propose`/`opsx:propose`
skill to start a new change rather than editing specs by hand.

## Release process

Releases are cut via the `release.yml` GitHub Actions workflow (manual `workflow_dispatch` with a
patch/minor/major choice): it dry-run publishes, bumps the version in `Cargo.toml` with
`cargo set-version`, tags, creates a GitHub release, publishes to crates.io, and attaches
cross-compiled binaries (linux-x64-musl, macos-x64, macos-aarch64). Don't bump the version in
`Cargo.toml` manually for a release — that workflow owns it.
