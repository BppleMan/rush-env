# rush

> A modern, configuration-driven Zsh environment generator, built to unify your dotfiles and automate your shell configuration across platforms and workflows.

---

## 🌟 What is rush?

**rush** is a modular, programmable CLI tool designed to generate a complete Zsh configuration script via a single `eval` command. Instead of manually maintaining a scattered collection of `.zshrc` fragments, aliases, and environment variables, rush allows you to define your terminal environment declaratively using a structured configuration file.

From plugin managers like Antigen to paths, proxies, and platform-specific tweaks—rush orchestrates your entire shell startup logic into a clean, reproducible flow.

---

## 🚀 Why rush?

In an increasingly cross-platform development landscape, maintaining shell environments across macOS, Linux, WSL, and remote servers can be painful. rush is designed to be:

- **Portable**: render system-specific shell logic based on detected or configured OS
- **Composable**: enable or disable modules like `env`, `alias`, `antigen`, `proxy` via config
- **Declarative**: use a single TOML or YAML file or GUI? to describe your environment
- **Stateless**: never write to your disk—rush emits the full shell logic via stdout
- **Hackable**: every output is based on customizable templates

With rush, a single command like:

```zsh
eval "$(rush --os macos --profile work)"
```

can completely bootstrap your terminal experience, making it consistent, auditable, and fast to replicate on any machine.

---

## 🧪 Project Status

This project is **still in its infancy**. While the design, structure, and goals are established, actual features are **not yet implemented**.

We are currently working on:

- Configuration schema design
- Template engine integration
- Core architecture to support modular and extensible behavior

If you’re interested in joining early design discussions or want to contribute ideas, issues and PRs are welcome!

---

## 🌐 Other Languages

This README is also available in:

- [🇨🇳 中文版 README_CN.md](./README_CN.md)

---

## 📄 License

Apache License 2.0

---

Let your dotfiles evolve. One `eval` at a time.