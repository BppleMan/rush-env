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

insta:
    # 1) 生成/审阅快照（避免测试抖动影响覆盖率）
    -cargo insta test -p rush-var --all-features
    # 2) 审阅快照
    cargo insta review

llvm-cov-prepare:
    #!/usr/bin/env zsh
    # 1) 安装 Xcode 命令行工具
    xcode-select --install

    # 2) 安装 llvm-tools-preview 和 cargo-llvm-cov
    rustup component add llvm-tools-preview
    cargo install cargo-llvm-cov

    # 3) 让 cargo-llvm-cov 用 Xcode 的 llvm
    export LLVM_COV="$(xcrun -f llvm-cov)"
    export LLVM_PROFDATA="$(xcrun -f llvm-profdata)"

    # 可选：看看路径是否正确
    echo $LLVM_COV
    echo $LLVM_PROFDATA
    "$LLVM_COV" --version
    "$LLVM_PROFDATA" --version

    # 3) 先清干净
    cargo llvm-cov clean --workspace

llvm-cov-rush-var:
    # 1) 生成/审阅快照（避免测试抖动影响覆盖率）
    cargo insta review

    # 2) 跑覆盖率（终端摘要）
    cargo llvm-cov -p rush-var \
      --ignore-filename-regex '/(tests|examples|benches)/'

llvm-cov-rush-var-html:
    cargo llvm-cov -p rush-var --html --open \
      --ignore-filename-regex '/(tests|examples|benches)/'
    cargo llvm-cov -p rush-var --json \
        --output-path target/llvm-cov \
        --ignore-filename-regex '/(tests|examples|benches)/' \
        --no-run
