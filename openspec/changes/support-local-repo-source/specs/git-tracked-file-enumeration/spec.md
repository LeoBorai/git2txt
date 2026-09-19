## Purpose

Defines which files from a resolved scan directory (a fresh clone or a local repository) are
included in the aggregate output, using Git's own tracked/ignored-file knowledge instead of an
independent directory walk.

## ADDED Requirements

### Requirement: File list comes from Git, not a directory walk
For a given scan directory, the system SHALL enumerate files to include by running
`git ls-files --cached --others --exclude-standard` inside that directory, rather than performing
an independent recursive filesystem walk with a hardcoded directory skip-list.

#### Scenario: Enumeration used for a cloned repository
- **WHEN** the scan directory is a freshly cloned repository
- **THEN** the system lists files to include via `git ls-files --cached --others
  --exclude-standard` run in that directory

#### Scenario: Enumeration used for a local repository
- **WHEN** the scan directory is a local path scanned in place
- **THEN** the system lists files to include via `git ls-files --cached --others
  --exclude-standard` run in that directory

### Requirement: Gitignored files are excluded
The system SHALL exclude any file matched by the repository's `.gitignore` (or other standard
Git exclude rules) from the aggregate output.

#### Scenario: Ignored file present
- **WHEN** the scan directory contains a file matched by `.gitignore`
- **THEN** that file's contents are not included in the aggregate output

### Requirement: Tracked and untracked-non-ignored files are included
The system SHALL include both files already tracked by Git and untracked files that are not
excluded by `.gitignore`, in the aggregate output.

#### Scenario: Committed file present
- **WHEN** the scan directory contains a file already committed to Git
- **THEN** that file's contents are included in the aggregate output

#### Scenario: New untracked, non-ignored file present
- **WHEN** the scan directory contains a file that has not been committed or staged, and is not
  matched by `.gitignore`
- **THEN** that file's contents are included in the aggregate output

### Requirement: Tool's own output file is always excluded
The system SHALL exclude a file named `output.txt` at the root of the scan directory from the
aggregate output, regardless of whether it is tracked, untracked, or `.gitignore`d, to prevent a
prior run's output from being scanned into a subsequent run.

#### Scenario: Untracked, non-ignored output.txt from a prior run
- **WHEN** the scan directory contains an untracked, non-`.gitignore`d file named `output.txt` at
  its root
- **THEN** that file's contents are not included in the new aggregate output

### Requirement: Enumeration failure aborts the run
WHEN `git ls-files` fails for the resolved scan directory (for example, because it is not a Git
repository), the system SHALL abort without writing a partial or empty output file.

#### Scenario: Enumeration command fails
- **WHEN** `git ls-files --cached --others --exclude-standard` fails for the scan directory
- **THEN** the system exits with an error
- **AND** no output file is written
