FROM ubuntu:22.04

# 安装 zsh 和其他必要工具
RUN apt-get update && \
    apt-get install -y zsh git curl

# 设置 zsh 为默认 shell
RUN chsh -s /usr/bin/zsh

# 设置工作目录
WORKDIR /rush-dir

# 使用 zsh 作为默认 shell
ENV SHELL=/usr/bin/zsh

# 启动 zsh
CMD ["/usr/bin/zsh"]
