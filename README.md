# change-monitor

# Installation

## Cloning
```bash
git clone <git repo address>
````

## Dependencies

### Software
- [git](https://github.com/git/git)
- [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [just](https://github.com/casey/just)

# Installation
Just type `just` to get a list of all available commands and some usage instructions, output similar to:

```bash
Available just recipes:
   • default       # list available recipes
   • install       # Alias for cargo install --path 
   • i             # alias for `install`
```

## Install change-monitor

```bash
just install
```

or using the alias

```bash
just i
```

# Usage

```
change-monitor <filename>
```

# .deps.toml template

```toml
["file1.txt"]
dependencies = ["dep1.txt", "dep2.txt"]

["file2.txt"]
dependencies = [] # no dependencies, except from the file itself
```