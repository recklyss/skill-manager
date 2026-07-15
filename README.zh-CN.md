# skill-manager

[English](README.md)

<p align="center">
  <img src="assets/icon.png" alt="Skill Manager" width="128" />
</p>

<p align="center">
  <strong>面向 AI 扩展的本地优先控制中心。</strong><br />
  在不同 agent harness 中统一使用、确认、扫描和发现 Skill、MCP 服务器、slash command 与 CLI 工具。
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-111827?style=flat-square" /></a>
  <a href="https://github.com/recklyss/skill-manager/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/recklyss/skill-manager?style=flat-square&color=EA580C" /></a>
  <a href="https://www.npmjs.com/package/@recklyss/skill-manager"><img alt="npm version" src="https://img.shields.io/npm/v/%40recklyss%2Fskill-manager?style=flat-square&logo=npm&logoColor=white" /></a>
  <a href="#安装"><img alt="Install with Homebrew" src="https://img.shields.io/badge/install-homebrew-FBBF24?style=flat-square&logo=homebrew&logoColor=111827" /></a>
  <a href="#安装"><img alt="macOS ARM64/x64 and Linux x64/ARM64" src="https://img.shields.io/badge/platform-macOS%20ARM64%2Fx64%20%2B%20Linux%20x64%2FARM64-111827?style=flat-square&logo=linux&logoColor=white" /></a>
  <a href="#本地优先安全模型"><img alt="Local-first" src="https://img.shields.io/badge/data-local--first-0F766E?style=flat-square" /></a>
</p>

## 为什么需要它

AI 扩展通常分散在各个 harness 自己的文件夹、MCP 配置文件、slash command 位置和商城来源中。Skill Manager 提供一个本地控制界面来管理这些内容：

| 产品概念 | 含义 |
|---|---|
| **使用中** | Skill Manager 正在控制此项目，并可在不同 harness 中启用或停用。 |
| **待确认** | Skill Manager 发现了本地状态、配置差异或库存问题，需要你先做决定。 |
| **扫描** | 在信任某个 Skill 之前，使用 harness CLI 进行安全检查。 |
| **发现** | 浏览商城，并预览外部工具。 |

## 你可以做什么

- 查看哪些扩展正在使用、哪些需要确认，以及它们在哪些 harness 中启用。
- 将本地 Skill 采用到共享库存，再按 harness 启用或停用。
- 使用已启用的 harness CLI 扫描 Skill，并在使用前查看发现项。
- 安装或采用 MCP 服务器配置，解决配置差异，并写入支持的 harness。
- 统一管理可复用的 slash command，并同步到支持的 harness。
- 从商城来源发现 Skill、MCP 服务器，以及仅预览的 CLI 工具。
- 在浅色、深色模式之间切换，并从内置 Color Hunt 主题中选择。

## 产品导览

### 总览与主题

从整个扩展组合开始查看：使用中、待确认、可发现内容，以及各 harness 的覆盖情况。可在侧边栏切换主题——浅色、深色，以及 Earthy Sage、Ocean Depths、Berry Sunset 等 Color Hunt 配色。

<p align="center">
  <img src="assets/change-theme.png" alt="总览仪表盘与主题选择器" width="920" />
</p>

### 使用中的 Skill

以网格、看板或矩阵视图浏览已采用的 Skill。按名称、标签或描述搜索，然后按 harness 或全局启用/停用。深色模式适合长时间审阅。

<p align="center">
  <img src="assets/dark-theme-and-different-view-of-skills.png" alt="深色主题下的使用中 Skill 网格视图" width="920" />
</p>

### 采用 Skill

当 Skill Manager 在 harness 中发现尚未管理的 Skill 时，打开详情抽屉查看描述、所在 harness 和磁盘路径，一键采用到共享库存。

典型流程：

1. 确认 harness 中发现的 Skill，或从商城安装一个 Skill。
2. 将它采用到 Skill Manager 库存。
3. 只在需要的 harness 中启用。
4. 从一个地方更新、移除或删除。

<p align="center">
  <img src="assets/add-to-skill-manager.png" alt="将 harness 中的 Skill 添加到 Skill Manager" width="920" />
</p>

### Skill 扫描

在依赖某个 Skill 之前，使用已启用的 agent harness CLI 扫描它。无需单独配置 LLM API。

**支持的扫描 harness**（须在设置中启用，且 CLI 在 `PATH` 中）：

| Harness | CLI 二进制 | 调用方式 |
|---------|------------|----------|
| Claude | `claude` | `claude -p`（非交互） |
| Codex | `codex` | `codex exec` |
| GitHub Copilot | `copilot` | `copilot -p --allow-all` |
| Cursor | `cursor-agent` | `cursor-agent -p -f` |

典型流程：

1. 在设置中启用至少一个支持的 harness，并安装其 CLI。
2. 将使用中的 Skill 切换到扫描视图。
3. 选择用于扫描的 harness。
4. 对单个 Skill、已选 Skill 或当前可见列表运行扫描。
5. 查看严重程度、发现项、代码片段和修复建议。

<p align="center">
  <img src="assets/scan-skill-risks.png" alt="Skill 扫描结果与风险发现项" width="920" />
</p>

静态启发式规则始终在本地运行。所选 harness CLI 执行语义分析，并须返回严格 JSON（`verdict`、`riskLevel`、`summary`、`findings`）。

### MCP 服务器

MCP 服务器会被规范化为 Skill Manager 记录，再转换为各 harness 期望的配置形状。在卡片或矩阵视图中浏览已采用的服务器、查看传输细节，并跨 harness 启用或停用。

典型流程：

1. 确认 harness 中发现的 MCP 服务器，或从商城安装一个。
2. 将它采用到 Skill Manager 库存。
3. 在需要的 harness 中启用。
4. 解决配置差异、停用 harness 绑定，或从一个地方卸载。

<p align="center">
  <img src="assets/mcp-in-use.png" alt="使用中的 MCP 服务器" width="920" />
</p>

### Slash command

Slash command 作为共享 prompt 库保存，而不是在每个 harness 专用格式中重复维护。

典型流程：

1. 创建包含名称、描述和 prompt 的 slash command。
2. 用 `$ARGUMENTS` 表示运行时输入插入位置。
3. 同步到支持的 harness。
4. 确认已有的 harness command 文件，并在需要时采用到共享库。

### 商城

商城是发现界面：

- **Skill 商城**：从 skills.sh 浏览并安装 Skill。
- **MCP 商城**：从 MCP Registry 浏览并安装 MCP 服务器。
- **CLI 商城**：从 CLIs.dev 预览外部 CLI 工具。此区域仅展示，Skill Manager 不安装或管理 CLI。

<p align="center">
  <img src="assets/marketplace.png" alt="Skill 商城" width="920" />
</p>

### 设置

启用或停用 harness 支持，确认各 harness 的 Skill 存储路径，并选择主题。Skill Manager 会检测已安装的 harness 并显示其 skill 根目录，便于在采用扩展前核对路径。

<p align="center">
  <img src="assets/settings.png" alt="设置页：harness 根目录与主题" width="920" />
</p>

## 安装

### Homebrew（macOS 推荐）

```bash
brew tap recklyss/tap
brew install skill-manager
skill-manager start
```

### npm（macOS ARM64/x64 和 Linux x64/ARM64）

```bash
npm install -g @recklyss/skill-manager
skill-manager start
```

npm wrapper 会为当前平台和 CPU 架构下载对应的原生 release artifact。
GitHub Releases 会发布 macOS ARM64/x64 和 Linux x64/ARM64 的原生 release artifact。

## 支持的 harness

<table align="center">
  <tr>
    <td align="center" valign="middle">
      <img src="assets/harness-logos/codex-logo.svg" alt="Codex CLI" height="56" /><br />
      <strong>Codex CLI</strong><br />
      <a href="https://developers.openai.com/codex/cli">文档</a>
    </td>
    <td align="center" valign="middle">
      <img src="assets/harness-logos/claude-code-logo.svg" alt="Claude Code" height="56" /><br />
      <strong>Claude Code</strong><br />
      <a href="https://code.claude.com/docs/en/overview">文档</a>
    </td>
    <td align="center" valign="middle">
      <img src="assets/harness-logos/cursor-logo.svg" alt="Cursor" height="56" /><br />
      <strong>Cursor</strong><br />
      <a href="https://cursor.com/docs">文档</a>
    </td>
    <td align="center" valign="middle">
      <img src="assets/harness-logos/opencode-logo.svg" alt="OpenCode" height="56" /><br />
      <strong>OpenCode</strong><br />
      <a href="https://opencode.ai/docs">文档</a>
    </td>
    <td align="center" valign="middle">
      <img src="assets/harness-logos/hermes-logo.png" alt="Hermes Agent" height="56" /><br />
      <strong>Hermes Agent</strong><br />
      <a href="https://hermes-agent.nousresearch.com/docs">文档</a>
    </td>
    <td align="center" valign="middle">
      <img src="assets/harness-logos/openclaw-logo.svg" alt="OpenClaw" height="56" /><br />
      <strong>OpenClaw</strong><br />
      <a href="https://docs.openclaw.ai/start/getting-started">文档</a>
    </td>
    <td align="center" valign="middle">
      <img src="assets/harness-logos/copilot-logo.svg" alt="GitHub Copilot" height="56" /><br />
      <strong>GitHub Copilot</strong><br />
      <a href="https://docs.github.com/en/copilot/how-tos/copilot-cli">文档</a>
    </td>
  </tr>
</table>

| Harness | Skill | MCP 服务器 | Slash command |
|---|---:|---:|---:|
| Codex CLI | 支持 | 支持 | 支持 |
| Claude Code | 支持 | 支持 | 支持 |
| Cursor | 支持 | 支持 | 支持 |
| OpenCode | 支持 | 支持 | 支持 |
| Hermes Agent | 支持 | 支持 | 暂不支持 |
| OpenClaw | 支持 | 暂不支持 | 暂不支持 |
| GitHub Copilot | 支持 | 支持 | 暂不支持 |

## 本地优先安全模型

Skill Manager 是本地配置管理工具。它在你的机器上运行，并读取或写入本地 harness 扩展状态。

可能修改本地状态的操作包括：

- 采用本地 Skill 文件夹
- 为某个 harness 启用或停用 Skill
- 更新带来源信息的 Skill
- 移除或删除 Skill
- 运行 Skill 扫描，这会将受限 Skill 上下文发送给所选 harness CLI 进行分析
- 将 MCP 服务器安装到选定 harness 配置
- 采用已有 MCP 配置
- 启用、停用、解决差异或卸载 MCP 服务器
- 创建、更新、同步、导入或删除 slash command
- 修改 harness 支持设置

在 macOS 上，应用拥有的文件位于 `~/Library/Application Support/skill-manager`；在 Linux 上使用 XDG base directories。

## 工作方式

### Skill

采用之前，各 harness 指向各自的本地 Skill 文件夹。采用之后，Skill Manager 会在共享本地存储中保留一个规范包，并通过本地链接暴露给选定 harness。停用某个 harness 会移除该 harness 绑定，但不会删除包本身。

Skill Manager 默认把已管理 Skill 视为可迁移：Skill 一旦进入 shared store，就可以启用到任何受支持 harness。`originHarness` 只保留作来源记录。

Hermes Agent Skill 使用 Hermes 分类目录：`~/.hermes/skills/<category>/<skill>/SKILL.md`。共享 Skill 启用到 Hermes 时，默认会链接到 `skill-manager` 分类下。Skill Manager 只导入 Hermes 自己从外部 hub provenance 安装的 Skill（`.hub/lock.json` 中非 official/builtin/optional 的条目）。Hermes 自学习/local Skill、`.bundled_manifest` 跟踪的内置打包 Skill，以及 Hermes hub provenance 中记录的官方 optional Skill，都会从 Skill Manager 库存和批量操作中排除；Skill Manager 不会修改、链接或删除这些文件夹，让 `hermes update` 和 Hermes 自有 Skill 同步继续保持原有所有权。

### Skill 扫描

Skill 扫描会从 `SKILL.md` 以及 Skill 包内选定的文本文件构建受限 prompt 上下文（最多 64 KB）。静态启发式规则在本地运行。语义分析调用你在扫描视图中选择的 harness CLI（Claude、Codex、Copilot 或 Cursor）。CLI 须返回包含 `verdict`、`riskLevel`、`summary`、`findings` 的严格 JSON。扫描超时时间为 120 秒。

旧版 LLM 扫描配置 API 仍保留以兼容，但主扫描流程已不再依赖它们。

扫描报告会展示 Skill 是否安全、最高严重程度、发现项、位置、片段和修复建议。前端会将已完成报告缓存在浏览器 localStorage 中，因此最近结果在页面切换后仍可查看。

### MCP 服务器

MCP 服务器以规范化 Skill Manager 记录保存，再转换为每个 harness 需要的配置形状：

- Codex 使用 `mcp_servers` 下的 TOML。
- Claude Code 和 Cursor 使用 `mcpServers` JSON 条目。
- OpenCode 使用类型化的本地或远程 MCP 条目。
- Hermes Agent 使用 `~/.hermes/config.yaml`（或 `$HERMES_HOME/config.yaml`）中 `mcp_servers` 下的 YAML 配置。
- OpenClaw 暂不支持 MCP 写入。

当 Skill Manager 发现同一个 MCP 服务器存在不同配置时，会先要求你选择事实来源。

### Slash command

Slash command 以 TOML 记录保存在 Skill Manager 应用存储中，再渲染到每个支持 harness 的格式：

- OpenCode 写入 `~/.config/opencode/commands` 下的 Markdown command 文件，并通过 `/` 调用。
- Claude Code 写入 `~/.claude/commands` 下的 Markdown command 文件，并通过 `/` 调用。
- Cursor 写入 `~/.cursor/commands` 下的纯文本 command 文件，并通过 `/` 调用。
- Codex 写入 `~/.codex/prompts` 下的 prompt 文件，并通过 `/prompts:` 调用。
- Hermes Agent 暂不支持 slash command 写入；Hermes 的可复用工作流优先通过 Skill 管理。
- OpenClaw 暂不支持 slash command 写入。

Skill Manager 使用同步状态和内容哈希跟踪目标所有权。它不会自动覆盖未跟踪的 command 文件；当目标不再匹配上次同步哈希时，会报告托管文件已变更或缺失。确认操作可用于采用未托管 command、恢复托管内容、将已变更的 harness command 采用为新来源，或移除损坏绑定且保留 harness 文件。

### CLI

CLI marketplace 条目仅用于预览。

## 配置

在 macOS 上，应用拥有的文件位于 `~/Library/Application Support/skill-manager`；在 Linux 上使用 XDG base directories。

常用 macOS 路径：

- 共享 Skill 存储：`~/Library/Application Support/skill-manager/shared`
- MCP manifest：`~/Library/Application Support/skill-manager/mcp/manifest.json`
- slash command 库：`~/Library/Application Support/skill-manager/slash-commands/commands`
- slash command 同步状态：`~/Library/Application Support/skill-manager/slash-commands/sync-state.json`
- 商城缓存：`~/Library/Application Support/skill-manager/marketplace`
- 应用数据库和 LLM 扫描配置：`~/Library/Application Support/skill-manager/skill-manager.db`
- 应用设置：`~/Library/Application Support/skill-manager/settings.json`

常用 Linux 路径：

- 共享 Skill 存储：`${XDG_DATA_HOME:-~/.local/share}/skill-manager/shared`
- MCP manifest：`${XDG_DATA_HOME:-~/.local/share}/skill-manager/mcp/manifest.json`
- slash command 库：`${XDG_DATA_HOME:-~/.local/share}/skill-manager/slash-commands/commands`
- slash command 同步状态：`${XDG_DATA_HOME:-~/.local/share}/skill-manager/slash-commands/sync-state.json`
- 商城缓存：`${XDG_DATA_HOME:-~/.local/share}/skill-manager/marketplace`
- 应用数据库和 LLM 扫描配置：`${XDG_DATA_HOME:-~/.local/share}/skill-manager/skill-manager.db`
- 应用设置：`${XDG_CONFIG_HOME:-~/.config}/skill-manager/settings.json`

大多数用户不需要修改这些位置。如果你在自定义环境中管理 Skill，可以用环境变量覆盖单个 Skill 根目录。

| Harness | 环境变量 | 默认 Skill Manager Skill 根目录 |
|---|---|---|
| Codex | `SKILL_MANAGER_CODEX_ROOT` | `~/.agents/skills` |
| Claude | `SKILL_MANAGER_CLAUDE_ROOT` | `~/.claude/skills` |
| Cursor | `SKILL_MANAGER_CURSOR_ROOT` | `~/.cursor/skills` |
| OpenCode | `SKILL_MANAGER_OPENCODE_ROOT` | `~/.config/opencode/skills` |
| Hermes Agent | `SKILL_MANAGER_HERMES_ROOT` | `${HERMES_HOME:-~/.hermes}/skills` |
| OpenClaw | `n/a` | `~/.openclaw/skills` |
| GitHub Copilot | `SKILL_MANAGER_COPILOT_ROOT` | `~/.copilot/skills` |

MCP 配置位置由 harness 拥有。Skill Manager 只写入经过验证的配置路径，并跳过不支持的 harness 写入。Hermes Agent 配置发现会优先使用 `SKILL_MANAGER_HERMES_HOME`，然后是 `HERMES_HOME`，最后回退到 `~/.hermes`。

## 从源码运行

### Tauri 桌面应用（推荐）

```bash
# 要求：Rust 1.85+、Node.js 24+（见 `.nvmrc`）
npm install
npm run tauri:dev
```

应用会以原生桌面窗口打开——无需浏览器，也无需手动启动服务器。

构建原生安装包：

```bash
npm run tauri:build
```

### 验证

```bash
npm run typecheck
npm test
npm run test:rust                # Rust 集成测试
npm run build
cd src-tauri && cargo check      # Rust 编译检查
```

## 故障排查

- 如果商城请求失败并显示 `Marketplace is temporarily unavailable`，请确认网络连接后重试。
- 在 macOS 上，如果 `npm install -g @recklyss/skill-manager` 提示 Homebrew 已拥有 `skill-manager`，请先卸载 Homebrew formula。反过来也一样：切回 Homebrew 前请先卸载 npm 包。
- 如果某个 MCP harness 显示为不可用，说明 Skill Manager 检测到本地客户端缺失，或该客户端不支持所需配置界面。

## 后续计划

### 扩展类型

- [ ] Hook 支持
- [x] Slash command 支持
- [ ] Plugin 支持

### Harness 扩展

- [x] GitHub Copilot
- [ ] Gemini CLI
- [ ] Cline
- [ ] Windsurf
- [ ] Qwen Code
- [ ] Kimi Code
- [ ] Qoder

## 社区

- 查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解贡献指南。
- 查看 [SECURITY.md](SECURITY.md) 以私下报告安全漏洞。
