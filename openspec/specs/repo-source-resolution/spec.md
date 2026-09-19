# repo-source-resolution Specification

## Purpose

Determines whether the CLI's `source` argument refers to a remote Git repository or an
existing local directory, and resolves it to a single directory on disk to be scanned.

## Requirements

### Requirement: `source` argument defaults to current working directory
The CLI SHALL accept an optional positional `source` argument. WHEN `source` is omitted, the
system SHALL treat it as `.` (the current working directory).

#### Scenario: Invoked with no arguments
- **WHEN** the user runs `git2txt` with no arguments
- **THEN** the system resolves `source` to the current working directory

### Requirement: Remote URLs are cloned
WHEN `source` matches a recognized remote Git URL scheme (`https://`, `http://`, `ssh://`, or an
SCP-like `git@host:path` form), the system SHALL clone it with `git clone --depth=1` into a
temporary directory and use that temporary directory as the scan target.

#### Scenario: HTTPS URL provided
- **WHEN** the user runs `git2txt https://github.com/example/repo.git`
- **THEN** the system clones the repository into a temporary directory
- **AND** scans the temporary directory instead of any local path

#### Scenario: SCP-like SSH URL provided
- **WHEN** the user runs `git2txt git@github.com:example/repo.git`
- **THEN** the system clones the repository into a temporary directory

### Requirement: Existing local paths are scanned in place
WHEN `source` is not a recognized remote URL and refers to an existing path on the local
filesystem, the system SHALL scan that path directly without performing any clone or network
access.

#### Scenario: Local path provided
- **WHEN** the user runs `git2txt ./some/dir` and `./some/dir` exists on disk
- **THEN** the system does not invoke `git clone`
- **AND** scans `./some/dir` directly

#### Scenario: Default current directory
- **WHEN** the user runs `git2txt` with no arguments in a directory that exists
- **THEN** the system scans the current working directory directly without cloning

### Requirement: Local scan target must be a Git repository
WHEN the resolved local scan target (explicit path or default current directory) is not inside a
Git repository, the system SHALL fail with an error that states the path is not a Git repository,
without producing an output file.

#### Scenario: Local path is not a Git repository
- **WHEN** the user runs `git2txt ./some/dir` and `./some/dir` exists but is not inside a Git
  repository
- **THEN** the system exits with an error indicating `./some/dir` is not a Git repository
- **AND** no output file is written

### Requirement: Unresolvable source fails clearly
WHEN `source` is neither a recognized remote Git URL nor an existing local path, the system SHALL
fail with an error stating that `source` could not be resolved, without attempting a clone or
scan.

#### Scenario: Source is neither a URL nor an existing path
- **WHEN** the user runs `git2txt not-a-real-path-or-url`
- **THEN** the system exits with an error stating the source could not be resolved as a URL or an
  existing path
- **AND** no output file is written
