## Why

`git2txt` always requires a remote Git URL and clones it before scanning. Users who already
have a repository checked out locally (e.g. the CWD) have no way to flatten it into a TXT file
without a network round trip through a fresh clone of their own working copy.

## What Changes

- **BREAKING**: rename the CLI's positional argument from `url` to `source`. `source` accepts
  either a remote Git URL (`https://`, `git@`, `ssh://` schemes) or a local filesystem path.
  It becomes optional, defaulting to `.` (the current working directory).
- When `source` looks like a remote URL: behavior is unchanged — clone with `git clone --depth=1`
  into a temp dir, then scan that temp dir.
- When `source` resolves to an existing local path: skip cloning entirely and scan that path
  directly in place.
- Replace the recursive directory walk in `Scanner` (and its hardcoded `SKIP_DIRECTORIES` /
  `.git`+`node_modules` skip list) with file enumeration via
  `git ls-files --cached --others --exclude-standard`, run inside whichever directory is being
  scanned (temp clone or local path). This makes both modes behave identically: only tracked
  files plus untracked-but-not-`.gitignore`d files are included, in both cases.
- Enumeration requires the scanned directory to be inside a Git repository (both a fresh clone
  and an arbitrary local path must be one); a local path that is not a Git repo, or a `source`
  that is neither an existing path nor a recognizable remote URL, fails with a clear error
  instead of silently producing an empty or wrong output.

## Capabilities

### New Capabilities
- `repo-source-resolution`: determining whether `source` is a remote URL or a local path, and
  producing the directory to scan (cloning when remote, using the path directly when local).
- `git-tracked-file-enumeration`: listing the files to include in the aggregate output for a
  given directory using `git ls-files --cached --others --exclude-standard`, replacing the
  previous recursive-walk + hardcoded skip-list approach.

### Modified Capabilities
(none — no existing specs exist yet for this project)

## Impact

- `src/bin/main.rs`: `Cli` struct (`url` → `source`, optional with default `.`), source-kind
  detection, branching between clone-then-scan and scan-in-place.
- `src/scanner.rs`: recursive `Scanner::scan` walk and `SKIP_DIRECTORIES` replaced by
  git-ls-files-based enumeration; drops the `futures`/`BoxFuture` recursion.
- `src/git.rs`: unchanged.
- CLI compatibility: existing invocations `git2txt <url>` keep working unchanged; bare
  `git2txt` (no `url`) previously errored (required arg) and now scans the CWD instead.
