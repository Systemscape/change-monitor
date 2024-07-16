# Short alias for install
alias i := install

# list available recipes
default:
    @echo "Available just recipes:"
    @just --list --unsorted --list-heading '' --list-prefix "   • " --justfile {{justfile()}}

# Alias for cargo install --path .
@install:
    cargo install --path .

