# change-monitor

## Introduction

Imagine you have a git repository that contains several `.typ`, `.tex` or `.md` files or whatever you use to generate Portable Document Files. Once the _.pdf_ leaves your repository, they _will_ get printed out, sent via Mail and uploaded to whoknowswhere. 

How to identify the documents, find out who reviewed and approved them and prevent use of obsolete documents?

We are developers first and foremost and don't want to deal with any badly designed UIs for document management, we want to use the tools we are familiar with: `git`, _GitHub_ and the command line.

This tool solves just that problem. It is a thin wrapper around git that gives you the last commit which changed a file (or one of its dependencies) you are interested in. The dependencies are defined in a simple _.toml_ file.

We use it to automatically insert the git commit hash into our documents when building them and before using them internally or sending them to customers. We open a PR for changes and use GitHub for the review and approval process.

### Rationale

Let's say we have a file called `manual.typ`, which includes several images and is used to build `manual.pdf`. If the file itself or one of the images is changed, we want to update the commit hash in `manual.pdf`. We do not want to update the commit hash if it's not needed, since we might track changes to `manual.pdf` itself in the repository and don't want our repo size to explode.

If there are uncommited changes, a ` DIRTY` flag is appended to the commit hash in the document, marking this file as unfit for usage, since it might depend on uncommited changes.

## Usage

```
change-monitor <filename> [--date]
```

The `--date` flag gives you the date of the latest commit instead of the hash, so you know the date the file was last changed.

The commit hash or date, respectively, are written to `stdout`, everything else (loggingm, errors) goes to `stderr`.

The tool looks for a file called `.deps.toml` located at the basedirectory of your `<filename>`.

### Examples
```bash
$ change-monitor example.typ
INFO  [change_monitor] Monitor changes for file: "/Users/bob/docs/example-folder/example.typ"
5d6256345067a82563106c868f2ad1b384286dce
```

or to get the date:

```bash
$ change-monitor example.typ --date
INFO  [change_monitor] Monitor changes for file: "/Users/bob/docs/example-folder/example.typ"
2024-07-26
```

## .deps.toml template
List dependencies for each versioned file

```toml
["file1.typ"]
dependencies = ["dep1.typ", "dep2.typ"]

["file2.txt"]
dependencies = [] # no dependencies, except for the file itself

["file2.tex"]
dependencies = ["*.png", ":!subfolder"]

# file4.txt dependencies is not defined, so the whole basedirectory is taken as a dependency
```

We pass the entries to git directly, so you can use [git pathspecs](https://git-scm.com/docs/gitglossary#Documentation/gitglossary.txt-aiddefpathspecapathspec) to exclude files or to use wildcards.

## Installation

### Cloning
```bash
git clone https://github.com/Systemscape/change-monitor
```

## Dependencies

### Software
- [git](https://github.com/git/git)
- [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [just](https://github.com/casey/just) (optional)

# Installation
Just type `just` to get a list of all available commands and some usage instructions, output similar to:

```bash
Available just recipes:
   • default       # list available recipes
   • install       # Alias for cargo install --path 
   • i             # alias for `install`
```

### Install change-monitor

```bash
just install
```

or using the alias

```bash
just i
```

or if you don't have `just` installed

```bash
cargo install --path <path/to/repo>
```

### Alternative using cargo without cloning
```bash
cargo install --git https://github.com/Systemscape/change-monitor --locked
```
