<p align="center"><img src="../assets/logo.svg" alt="holstered 标志" width="160"></p>

# holstered

**你的智能体挎着 500 多个技能，却一个都不拔。holstered 替它拔出对的那一个。**

终端里最快的拔枪：大约一秒，给出一个技能，或者一个都不给。

<sub><a href="../README.md">English</a> &middot; 简体中文</sub>

装了几百个技能的智能体只能看到技能名称，真正对口的那个往往用不上。holstered
是一个提示词钩子（prompt hook）：每收到一条用户提示词，它先用关键词（BM25）和语义（内置于二进制的小型向量模型）筛出候选技能，
再让决策模型从中选出一个（或者一个都不选），然后把该技能的 `SKILL.md`
注入到模型本轮的上下文中。设置[候选阈值](../CONFIGURATION.md)后，涉及多个任务的
提示词还会一并注入概率超过阈值的候选技能。

https://github.com/user-attachments/assets/b6de382b-8ca3-4fe0-9baa-0039342dd3b0

决策模型由你选择：托管在 OpenRouter 上的 **Jev**，或在本地运行的开源模型
**Kev**（数据不离开你的机器）。参见[选择决策模型](#选择决策模型)。

一个 Rust 二进制文件通过 [polyhook](https://github.com/polyhook/polyhook)
服务所有智能体，由 polyhook 负责转换各智能体的钩子输入与响应格式。

## 工作原理

<p align="center"><img src="../assets/flow.svg" alt="用户提示词 → 智能体提示词钩子 → holstered：polyhook 读取提示词，BM25 与向量检索筛出前 20 个技能，Jev 或 Kev 选出一个或不选，设置候选阈值时外加最多 2 个超过该阈值的候选技能，polyhook 注入它们的 SKILL.md → 模型在本轮看到这些技能。若模型回答 none、给出未提供的技能、出错或超时，则不注入任何内容，提示词原样通过。" width="560"></p>

以下情况不会注入任何内容，提示词原样通过：决策模型回答 `none`、给出了不在候选列表中的技能、
调用失败或超时（默认 8 秒），或者没有配置任何决策模型。holstered 永远不会阻断提示词。

注入的 `SKILL.md` 上限为 8,000 字节，因为部分智能体会截断超过约 10KB 的钩子输出。
更长的技能会在此处截断，并在末尾附上指向原文件的提示，但模型未必会去读取。
请将 `SKILL.md` 控制在 8KB 以内（约 100 行正文，可用 `wc -c` 检查），把细节移到它引用的文件中。

### 为什么是 BM25 + 决策模型

在一个 581 个技能的技能库上，使用 38 条带标注的提示词测试（其中 32 条有正确技能，6 条没有）：

<p align="center"><img src="../assets/benchmark.svg" alt="正确选择（共 32 条）/ 保持沉默（共 6 条）：关键词匹配 14/5，BM25 top-1 17/2，BM25 top-20 → Cohere rerank-v3.5 25/2，BM25 top-20 → Jev（Python 原型）28/6，holstered 二进制（bm25 crate → Jev）29/6，holstered 二进制（bm25 crate → 本地 Kev-4B）26/6。"></p>

holstered 这几行是发布版二进制端到端运行的结果：对接线上 Jev（每条提示词中位数 915 ms），
以及在 Apple Silicon Mac 上运行的本地 Kev-4B（中位数 1.7 秒）。三次失误中有一次是：
对一条制作幻灯片的提示词选了 `pptx` 技能，而标注只认可另一个技能。样本少、仅一人标注：
请把它看作方向参考，而非保证。

仅靠关键词，会漏掉与技能没有共同词语的提示词：“what can I cook with eggs, rice and spinach?”
从未匹配到“Plan meals and recipes from ingredients on hand”。因此候选列表还会按语义排序：
使用内置的 [potion-base-4M](https://huggingface.co/minishlab/potion-base-4M) 静态向量（离线运行，
每条提示词约 0.1 秒），再将两种排序融合。在仓库的[评测](../evals/README.md)中，Jev 和 Kev
的所有标注技能现在都能进入候选列表。

## 支持的智能体

智能体支持来自 polyhook；完整列表以及哪些智能体支持上下文注入，参见其
[Supported Tools](https://github.com/polyhook/polyhook#supported-tools)。

| 智能体 | 钩子 | 注入 |
|---|---|---|
| Claude Code | `UserPromptSubmit` | ✅ |
| Codex | `UserPromptSubmit` | ✅ |
| Gemini CLI | `BeforeAgent` | ✅ |
| Hermes Agent | `pre_llm_call` | ✅ |
| Cline | `UserPromptSubmit` | ✅ |
| Cursor、Windsurf、Amp | — | ❌ 它们的提示词钩子只能放行或阻断，因此无需注册 |

## 安装

任选一种：

```bash
# npm（预编译二进制，无需 Rust 工具链）
npm install -g holstered

# Go（首次运行时下载预编译二进制）
go install github.com/tupe12334/holstered/go/cmd/holstered@latest
# 或在 Go 项目中固定版本（Go 1.24+）：go get -tool github.com/tupe12334/holstered/go/cmd/holstered

# crates.io
cargo install holstered

# Homebrew（从源码构建）
brew install tupe12334/tap/holstered

# GitHub 上最新的 main
cargo install --git https://github.com/tupe12334/holstered
```

然后[选择决策模型](#选择决策模型)，并把 holstered 添加到你使用的每个智能体：

**Claude Code** — 安装插件（两条命令需作为两条独立的提示词发送）：
```
/plugin marketplace add tupe12334/holstered
```
```
/plugin install holstered@holstered
```

**Codex** — 安装插件，然后在 `codex` 中打开 `/hooks` 并信任它的钩子：
```bash
codex plugin marketplace add tupe12334/holstered
codex plugin add holstered@holstered
```

**Gemini CLI** — 安装扩展：
```bash
gemini extensions install https://github.com/tupe12334/holstered
```

**Hermes Agent** — 安装插件，然后重启 Hermes：
```bash
hermes plugins install tupe12334/holstered#plugins/hermes --enable
```

**Cline** — 钩子是以事件命名的可执行文件
```bash
mkdir -p ~/Documents/Cline/Hooks
ln -s "$(command -v holstered)" ~/Documents/Cline/Hooks/UserPromptSubmit
```

## 选择决策模型

在启动智能体的环境中设置以下其中一个。如果两个都设置了，以 Kev 为准。

| | Jev（托管） | Kev（本地） |
|---|---|---|
| 设置 | `OPENROUTER_API_KEY` | `HOLSTERED_KEV_URL` |
| 运行方式 | OpenRouter Decisions API | 你本机上的 [Kev](https://github.com/jaredpalmer/kev) 服务 |
| 隐私 | 提示词（前 500 个和后 1,500 个字符）和候选技能描述会发送到 OpenRouter | 数据不离开本机 |
| 每条提示词延迟 | 约 1 秒 | 在 Apple Silicon Mac 上约 1.7–3.5 秒 |
| 在上述测试集上的准确率 | 29/32 | 26/32 |

Kev 的配置与首次请求超时：参见 [CONFIGURATION.md](../CONFIGURATION.md#local-model-kev)。

## 配置

环境变量、发送到 OpenRouter 的数据，以及如何用本地 Kev 模型完全离线运行：参见
[CONFIGURATION.md](../CONFIGURATION.md)。

## 参与贡献

参见 [CONTRIBUTING.md](../CONTRIBUTING.md)。

## 许可证

[MIT](../LICENSE)
