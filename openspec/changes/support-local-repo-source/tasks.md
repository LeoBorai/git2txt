## 1. Source resolution

- [x] 1.1 Change `Cli.url: String` to `Cli.source: String` with `default_value = "."` in
      `src/bin/main.rs`, and verify `cargo run --` (no args) no longer errors with "required
      argument missing"
- [x] 1.2 Implement a `source` classifier (remote URL vs. local path): remote if it starts with
      `https://`, `http://`, `ssh://`, or matches the SCP-like `user@host:path` form; local
      otherwise. Verify with unit tests covering an HTTPS URL, an SCP-like `git@...` URL, a
      relative path (`.`, `./foo`), and an absolute path
- [x] 1.3 Branch `main` on the classifier: remote → existing `Git::download` into a `TempDir`
      (unchanged); local → resolve `source` to a `PathBuf` directly, no clone. Verify by running
      `cargo run -- <a remote URL>` and confirming a clone still happens (temp dir created, no
      regression), and `cargo run --` in this repo's own checkout and confirming no `git clone`
      is invoked (no network access, no "Cloning..." message)
- [x] 1.4 Validate the local scan target exists and is inside a Git repository (e.g. via
      `git -C <path> rev-parse --is-inside-work-tree`) before enumeration; return a clear
      `anyhow` error naming the path if not. Verify by running `cargo run -- /tmp` (or another
      non-Git directory) and confirming a clear "not a Git repository" error, no output file
      written
- [x] 1.5 Handle the unresolvable-source case: `source` is not an existing local path and not a
      recognized URL → clear `anyhow` error naming `source`, no clone/scan attempted. Verify by
      running `cargo run -- not-a-real-path-or-url` and confirming the error message and that no
      output file is written

## 2. Git-based file enumeration

- [x] 2.1 Replace `Scanner`'s recursive `read_dir` walk in `src/scanner.rs` with a function that
      runs `git ls-files --cached --others --exclude-standard` (via `std::process::Command`,
      consistent with `git.rs`'s existing `git clone` invocation) inside the resolved scan
      directory and returns the resulting relative file paths. Remove `SKIP_DIRECTORIES`, the
      `futures`/`BoxFuture` recursion, and the now-unused `futures` dependency from `Cargo.toml`
      if nothing else in the crate uses it. Verify `cargo build` succeeds with no unused-import
      warnings
- [x] 2.2 Filter out a root-level file literally named `output.txt` from the enumerated list
      before reading any files. Verify with a test: create an untracked, non-ignored
      `output.txt` in a fixture repo, run enumeration, and confirm it is absent from the result
- [x] 2.3 For each enumerated path, read its bytes and append them into the aggregate output file
      (same mutex-guarded `File` write pattern as today), preserving today's byte-concatenation
      behavior (no separators added between files). Verify by running the tool against a small
      fixture repo with 2-3 known files and diffing the resulting `output.txt` against the
      expected concatenation
- [x] 2.4 Propagate `git ls-files` failure as an `anyhow::Error` that aborts before any output
      file is created or partially written. Verify by pointing enumeration at a non-Git directory
      and confirming no `output.txt` (or partial temp aggregate) is left behind

## 3. Integration and regression check

- [x] 3.1 Wire sections 1 and 2 together in `main.rs`: resolve `source` → scan directory →
      enumerate files → write `./output.txt` in the invoking process's CWD (unchanged output
      location for both modes). Verify end-to-end: `cargo run -- <a small public repo URL>`
      still produces `output.txt` as before, and `cargo run --` in this repo's own checkout
      produces an `output.txt` containing this repo's tracked files
- [x] 3.2 Run `cargo clippy` and `cargo fmt --all -- --check` and fix any new warnings introduced
      by the refactor, matching CI's `clippy.yml`/`fmt.yml` checks
- [x] 3.3 Update `README.md`'s "Running" section to document the new `source` argument
      (optional, defaults to CWD, accepts a local path or a remote URL) and verify the documented
      examples match actual CLI behavior
