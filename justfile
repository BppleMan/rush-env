#!/usr/bin/env just --justfile

release:
    cargo build --release

lint:
    cargo clippy

# Run code coverage tests for a specific package
cov package:
    cargo tarpaulin -p {{ package }} --out Html --out Json --out Xml --out Lcov --output-dir target/{{ package }}-coverage

# Run code coverage tests for rush-var package specifically
cov-rush-var:
    cargo tarpaulin -p rush-var --exclude-files "rush-env/*" --exclude-files "rush-say/*" --out Html --out Json --out Xml --out Lcov --output-dir target/rush-var-coverage

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

# Run coverage using Apple's llvm-cov via xcrun (absolute paths; no PATH changes)
llvm-cov package:
    #!/usr/bin/env bash
    set -euo pipefail
    LLVM_COV="$(xcrun -f llvm-cov)"
    LLVM_PROFDATA="$(xcrun -f llvm-profdata)"
    rm -rf target/llvm-cov
    mkdir -p target/llvm-cov/profraws
    PROFILE_DIR="$(pwd)/target/llvm-cov/profraws"
    LLVM_PROFILE_FILE="${PROFILE_DIR}/%m-%p.profraw" RUSTFLAGS="-C instrument-coverage -C link-dead-code -C opt-level=0" cargo test -p {{package}} --tests --all-features
    find "${PROFILE_DIR}" -maxdepth 1 -name '*.profraw' -print0 | xargs -0 "${LLVM_PROFDATA}" merge -sparse -o target/llvm-cov/coverage.profdata
    CRATE_NAME="$(printf %s "{{package}}" | tr '-' '_')"
    OBJS=$(find target/debug/deps -type f -perm -111 \( -name "${CRATE_NAME}-*" -o -name "${CRATE_NAME}_*" \) | tr '\n' ' ')
    "${LLVM_COV}" report ${OBJS} -instr-profile=target/llvm-cov/coverage.profdata -use-color -ignore-filename-regex='/\.cargo/registry|rustc|target/'
    "${LLVM_COV}" show ${OBJS} -instr-profile=target/llvm-cov/coverage.profdata -format=html -output-dir=target/llvm-cov/html -show-line-counts-or-regions -ignore-filename-regex='/\.cargo/registry|rustc|target/'

# Shortcut for rush-var package
llvm-cov-rush-var:
    just llvm-cov rush-var
