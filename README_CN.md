# rush

> 一个现代化、配置驱动的 Zsh 启动脚本生成器，旨在统一你的 dotfiles 管理方式，并在不同平台、场景中自动生成一致的终端配置。

---

## 🌟 什么是 rush？

**rush** 是一个模块化、可编程的 CLI 工具，用于根据结构化配置动态生成完整的 Zsh 启动脚本。你无需再手动维护 `.zshrc`、alias、环境变量和平台差异化逻辑，只需一份配置文件，就能定义你的终端环境。

无论是插件管理器（如 Antigen），还是路径、代理、系统差异化设置，rush 都能帮你以优雅、可维护的方式组织起来，并通过标准输出一键注入：

```zsh
eval "$(rush --os macos --profile work)"
```

即可完成环境初始化，轻松实现跨平台一致性。

---

## 🚀 rush 能做什么？

我们希望 rush 成为：

- **跨平台适配器**：支持 macOS、Linux、WSL 等多平台的环境差异处理
- **模块化引擎**：通过配置启用/关闭模块，如 `env`、`alias`、`antigen`、`proxy`
- **声明式驱动**：通过 TOML 或 YAML 文件或 GUI? 描述环境结构
- **零侵入设计**：不写入磁盘，仅通过标准输出生成完整 shell 脚本
- **模板可定制**：模块逻辑基于模板渲染，支持 override 与继承

将来我们甚至希望 rush 成为「dotfiles 领域的 Terraform」……

---

## 📢 当前状态

rush 正处于**早期设计阶段**，当前尚未提供稳定功能。我们正在设计核心架构、模块定义方案与模板引擎集成策略。

如果你对终端配置、模块化系统或 dotfiles 有热情，欢迎提出建议、提交 Issue 或参与设计！

---

## 🌍 语言支持

本项目的英文版说明文档请见：

- [🇺🇸 English README.md](./README.md)

---

## 📄 许可协议

本项目使用 **Apache License 2.0** 开源协议发布。

---

让你的 dotfiles 像代码一样演进。只需一行 `eval`。