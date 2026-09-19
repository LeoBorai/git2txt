## Context

See `proposal.md` - Why. Current implementation (`src/bin/main.rs`, `src/git.rs`,
`src/scanner.rs`): `Cli { url: String }` is required; `main` always shells out to
`git clone --depth=1 <url> <tempdir>`, then `Scanner::scan` recursively walks the temp dir
(async, boxed recursion via `futures::future::BoxFuture`), skipping only directories named
`node_modules` or `.git` (`SKIP_DIRECTORIES`), appending every other file's raw bytes into one
`aggregate.txt`, which is then copied to `./output.txt`.

A fresh shallow clone's working tree already equals "tracked files, minus `.git`" — there is no
separate gitignore-handling today because a clone never contains untracked/ignored content. A
local working directory does not have that property: it can contain `target/`, build output,
`.env`, editor files, and a stale `output.txt`, none of which today's skip-list filters out.

## Goals / Non-Goals

**Goals:**
- Scan a local directory (default: CWD) with no clone and no network access.
- Make local-mode file selection behave the way a clean clone would: tracked files, plus new
  files not yet committed, minus anything `.gitignore`d.
- Keep remote-mode's externally observable output byte-for-byte equivalent for repositories that
  don't rely on the old walk's peculiarities (see Risks).

**Non-Goals:**
- Supporting non-Git local directories (plain folders with no `.git`). Enumeration depends on
  `git ls-files`; the tool's name and existing clone-based design already assume Git.
- Any new CLI flags beyond renaming/relaxing the existing positional argument.
- Changing where output is written (still `./output.txt` in the invoking process's CWD).

## Decisions

**1. Single positional `source` arg, auto-detected as URL vs. local path.**
Considered an explicit `--local` flag or `remote`/`local` subcommands instead (see the earlier
exploration). Auto-detect keeps the existing one-positional-arg CLI shape intact for remote use
(`git2txt <url>` keeps working) and makes local use as low-friction as possible (`git2txt` /
`git2txt <path>`). Detection rule: `source` is "remote" if it starts with `https://`, `http://`,
`ssh://`, or matches the SCP-like form `<user>@<host>:<path>`; otherwise it's treated as a local
path (default `.` when omitted).

**2. Unify enumeration on `git ls-files --cached --others --exclude-standard`, for both modes.**
Alternative was to keep `Scanner`'s recursive walk untouched for the remote/clone path and add a
separate, local-only enumeration mechanism. Rejected: it leaves two different file-selection
mechanisms to maintain, and there's no behavioral reason to keep the walk for remote mode once
`git` is already a required runtime dependency (it's used for cloning today). Since a temp clone
is itself a Git repository, the exact same function enumerates files for either scan target.
This also deletes `Scanner`'s `BoxFuture`-based recursion and `SKIP_DIRECTORIES` entirely —
directory skipping is now whatever Git already knows via `.gitignore`.

**3. `--cached --others --exclude-standard`, not plain `git ls-files`.**
Plain `git ls-files` only lists committed/staged (tracked) files, matching a fresh clone exactly
but excluding a user's in-progress uncommitted local files — confirmed with the user as
insufficient for the local use case ("all files in the Git repository excluding those in the
.gitignore"). `--cached --others --exclude-standard` adds untracked-but-not-ignored files on top
of tracked ones.

**4. `output.txt` is always excluded from enumeration, unconditionally.**
Without this, re-running `git2txt` in the same local directory would fold the previous run's
aggregate file into the new one (compounding on every run) whenever `output.txt` is untracked and
not `.gitignore`'d — which is the common case for a first-time user. Filtering it out by name at
the scan-directory root avoids that footgun without requiring users to edit `.gitignore`. Only
the temp-clone path is exempt in practice, since a clone can't contain a stray `output.txt`
unless the repo itself commits one.

**5. Missing/failing Git enumeration is a hard error, not a fallback to walking the filesystem.**
Silently falling back to a raw walk when `git ls-files` fails (e.g., not a repo) would reintroduce
the two-mechanisms problem this change removes, and would surprise users with unfiltered output.
An explicit error matches the existing `anyhow::Result` error-propagation style already used
throughout `main.rs`/`git.rs`/`scanner.rs`.

## Risks / Trade-offs

- **[Risk]** Repos that currently rely on `SKIP_DIRECTORIES`' hardcoded `node_modules` skip but
  which commit `node_modules` anyway (unusual, but possible) will see behavior change: previously
  skipped, now included (since `git ls-files` reports whatever is tracked). → **Mitigation**:
  this matches user expectations better (git-tracked reality over a hardcoded guess), and is
  called out as a **BREAKING**-adjacent behavior note in the proposal's Impact section.
- **[Risk]** `git ls-files` requires the `git` binary on `PATH`, same as today's `git clone` step
  — no new external dependency introduced, but local mode now also has this requirement even
  though it previously needed no `git` binary at all for a purely local, non-git-clone flow (this
  flow didn't exist before). → **Mitigation**: acceptable per Non-Goals (Git-repo-only local
  support); error message states plainly that the path is not a Git repository.
- **[Risk]** Removing `Scanner`'s recursive walk changes its public shape (used only internally
  today, but it's a `pub` struct in a `pub` lib crate, so any external consumer of the `git2txt`
  library crate depending on `Scanner::scan`'s walk semantics would break). → **Mitigation**:
  crate is pre-1.0 (`0.1.2`) and the binary is the primary consumer; acceptable per proposal's
  **BREAKING** marker.

## Migration Plan

No data migration. Deploys as a normal version bump via the existing `release.yml` workflow.
Rollback is a normal revert, since there is no persisted state or schema involved.
