<div>
  <h1 align="center">git2txt</h1>
  <h4 align="center">
    Converts a Git repository to a single TXT file
  </h4>
</div>

<div align="center">

  [![Crates.io](https://img.shields.io/crates/v/git2txt.svg)](https://crates.io/crates/git2txt)
  [![Documentation](https://docs.rs/git2txt/badge.svg)](https://docs.rs/git2txt)
  ![Build](https://github.com/EstebanBorai/git2txt/workflows/build/badge.svg)
  ![Clippy](https://github.com/EstebanBorai/git2txt/workflows/clippy/badge.svg)
  ![Formatter](https://github.com/EstebanBorai/git2txt/workflows/fmt/badge.svg)

</div>

## Development

This project uses the library/binary crate architecture, which means that you can
install it as a dependency in your project or use it as a standalone binary.

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)

### Running

`git2txt` accepts a `source`, which may be a remote Git URL or a local path to an existing
Git repository. `source` is optional and defaults to the current directory.

```bash
# scan the current directory (must be inside a Git repository)
cargo r --

# scan a local repository, tracked + untracked-but-not-gitignored files
cargo r -- ./path/to/repo

# clone and scan a remote repository
cargo r -- <GIT REPO URL>
```

### Debugging

```bash
RUST_LOG=debug cargo r -- git@github.com:EstebanBorai/git2txt.git
```

### License

This project is dual licensed in both under the MIT License and the Apache License (Version 2.0).
