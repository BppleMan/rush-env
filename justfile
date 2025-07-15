#!/usr/bin/env just --justfile

release:
    cargo build --release

lint:
    cargo clippy

bin:
    cargo run --bin bin -- arg1

example:
    cargo run --example exname -- arg1

download repo:
    just {{ repo }}

antidote:
    curl -o ./repository/antidote.zip https://codeload.github.com/mattmc3/antidote/zip/refs/heads/main

vim-plug:
    curl -o ./repository/vim-plug.zip https://codeload.github.com/junegunn/vim-plug/zip/refs/heads/master

starship:
    curl -o ./repository/starship/starship-aarch64-apple-darwin.tar.gz https://github.com/starship/starship/releases/latest/download/starship-aarch64-apple-darwin.tar.gz
    curl -o ./repository/starship/starship-x86_64-apple-darwin.tar.gz https://github.com/starship/starship/releases/latest/download/starship-x86_64-apple-darwin.tar.gz
    curl -o ./repository/starship/starship-aarch64-unknown-linux-musl.tar.gz https://github.com/starship/starship/releases/latest/download/starship-aarch64-unknown-linux-musl.tar.gz
    curl -o ./repository/starship/starship-x86_64-unknown-linux-musl.tar.gz https://github.com/starship/starship/releases/latest/download/starship-x86_64-unknown-linux-musl.tar.gz

ohmyzsh:
    curl -o ./repository/ohmyzsh.zip https://codeload.github.com/ohmyzsh/ohmyzsh/zip/refs/heads/master
