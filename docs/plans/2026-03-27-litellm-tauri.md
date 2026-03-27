# LiteLLM Tauri Desktop Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 搭建一个 macOS 可用的 LiteLLM Tauri 桌面管理台，首次启动自动创建应用专属 `.venv` 并安装 `litellm[proxy]`，然后支持本地配置、代理进程控制、日志查看与测试请求。

**Architecture:** 使用 React + TypeScript + Vite 构建桌面前端，使用 Tauri/Rust 负责系统 Python 检查、runtime `.venv` 自举、LiteLLM 子进程与测试请求。前端只操作结构化配置与界面状态，Rust 负责把配置序列化为 LiteLLM YAML、安装运行时并执行代理生命周期管理。

**Tech Stack:** Tauri 2, React, TypeScript, Vite, Tailwind CSS, shadcn/ui, Rust, Tokio, Serde, Serde YAML, Reqwest

---

### Task 1: 初始化 Tauri 项目骨架

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/tauri.conf.json`

**Step 1: 初始化前端与 Tauri 依赖声明**

写入最小可运行依赖，包括 React、Vite 与 Tauri。

**Step 2: 运行依赖安装**

Run: `npm install`
Expected: 成功生成 `node_modules` 且无依赖解析错误

**Step 3: 初始化最小 Tauri 壳**

写出最小窗口配置与 Rust 入口，确保应用能编译。

**Step 4: 验证前端可启动**

Run: `npm run dev`
Expected: 本地开发服务器成功启动

**Step 5: 验证 Tauri 开发模式**

Run: `npm run tauri dev`
Expected: 桌面窗口成功拉起

**Step 6: Commit**

```bash
git add .
git commit -m "feat: scaffold tauri desktop app"
```

### Task 2: 建立共享类型与应用配置模型

**Files:**
- Create: `src/types/config.ts`
- Create: `src/lib/default-config.ts`
- Modify: `src/App.tsx`
- Test: `src/lib/default-config.test.ts`

**Step 1: 写失败测试**

为默认配置结构和模型项最小合法性写测试。

**Step 2: 运行测试确认失败**

Run: `npm test -- src/lib/default-config.test.ts`
Expected: 因类型或实现缺失而失败

**Step 3: 写最小实现**

定义 `AppConfig`、`AppSettings`、`ProviderPreset`、`ModelEntry` 以及默认配置工厂。

**Step 4: 运行测试确认通过**

Run: `npm test -- src/lib/default-config.test.ts`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src/types/config.ts src/lib/default-config.ts src/lib/default-config.test.ts src/App.tsx
git commit -m "feat: add desktop config model"
```

### Task 3: 实现 Rust 侧本地配置持久化与 runtime 路径定义

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/src/config.rs`

**Step 1: 写失败测试**

为配置保存、加载、app data 路径解析、runtime 路径解析写 Rust 单测。

**Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test config -- --nocapture`
Expected: 因模块不存在或函数缺失而失败

**Step 3: 写最小实现**

实现：
- `load_app_config`
- `save_app_config`
- `write_litellm_yaml`
- `runtime_dir`
- `runtime_python_path`

并定义 app data 目录内文件路径与 runtime 目录。

**Step 4: 运行测试确认通过**

Run: `cd src-tauri && cargo test config -- --nocapture`
Expected: 相关测试通过

**Step 5: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/main.rs
git commit -m "feat: persist app config and litellm yaml"
```

### Task 4: 实现 runtime 自举与环境检查命令

**Files:**
- Create: `src-tauri/src/runtime.rs`
- Create: `src-tauri/src/environment.rs`
- Modify: `src-tauri/src/main.rs`
- Create: `src/lib/api.ts`
- Test: `src-tauri/src/runtime.rs`
- Test: `src-tauri/src/environment.rs`

**Step 1: 写失败测试**

为这些逻辑写测试：
- 提取系统 Python 检测结果
- runtime 状态转换
- bootstrap 命令构造
- 安装日志脱敏摘要

**Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test runtime -- --nocapture`
Expected: 因 runtime 模块缺失而失败

Run: `cd src-tauri && cargo test environment -- --nocapture`
Expected: 因检测逻辑缺失而失败

**Step 3: 写最小实现**

实现：
- `bootstrap_runtime`
- `repair_runtime`
- `get_runtime_status`
- `check_python`
- `check_environment`

并在 Tauri 中暴露命令。runtime 流程需要：
- 创建 `.venv`
- 安装 `pip`
- 安装 `litellm[proxy]`
- 校验 `pip show litellm`

**Step 4: 前端接线**

创建 `src/lib/api.ts`，封装对 Tauri command 的调用。

**Step 5: 运行测试确认通过**

Run: `cd src-tauri && cargo test environment -- --nocapture`
Expected: 测试通过

Run: `cd src-tauri && cargo test runtime -- --nocapture`
Expected: 测试通过

**Step 6: Commit**

```bash
git add src-tauri/src/runtime.rs src-tauri/src/environment.rs src-tauri/src/main.rs src/lib/api.ts
git commit -m "feat: add managed runtime bootstrap"
```

### Task 5: 让 ProxyManager 依赖 runtime 并控制进程生命周期

**Files:**
- Create: `src-tauri/src/proxy_manager.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/src/proxy_manager.rs`

**Step 1: 写失败测试**

针对状态流转写测试：
- 初始为 stopped
- runtime 未就绪时拒绝启动
- start 后进入 starting/running
- stop 后回到 stopped

**Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test proxy_manager -- --nocapture`
Expected: 因管理器不存在而失败

**Step 3: 写最小实现**

实现：
- `start_proxy`
- `stop_proxy`
- `restart_proxy`
- `get_proxy_status`

以及子进程 PID、状态与日志句柄管理。启动前必须读取 runtime Python 路径，而不是接受用户输入解释器。

**Step 4: 运行测试确认通过**

Run: `cd src-tauri && cargo test proxy_manager -- --nocapture`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src-tauri/src/proxy_manager.rs src-tauri/src/main.rs
git commit -m "feat: add proxy process manager"
```

### Task 6: 实现日志流推送与脱敏

**Files:**
- Create: `src-tauri/src/logs.rs`
- Modify: `src-tauri/src/proxy_manager.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/src/logs.rs`

**Step 1: 写失败测试**

为日志脱敏规则写测试，验证 `api_key`、`master_key`、`Authorization` 被屏蔽。

**Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test logs -- --nocapture`
Expected: 因脱敏实现缺失而失败

**Step 3: 写最小实现**

实现：
- 日志行脱敏
- stdout/stderr 读取
- 通过 Tauri event 发给前端

**Step 4: 运行测试确认通过**

Run: `cd src-tauri && cargo test logs -- --nocapture`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src-tauri/src/logs.rs src-tauri/src/proxy_manager.rs src-tauri/src/main.rs
git commit -m "feat: stream sanitized proxy logs"
```

### Task 7: 构建桌面 UI 骨架与 runtime 状态展示

**Files:**
- Modify: `src/App.tsx`
- Create: `src/components/shell.tsx`
- Create: `src/components/overview-panel.tsx`
- Create: `src/components/config-panel.tsx`
- Create: `src/components/logs-panel.tsx`
- Create: `src/components/test-panel.tsx`
- Create: `src/styles.css`
- Test: `src/components/shell.test.tsx`

**Step 1: 写失败测试**

验证四个主面板可切换渲染，并能在概览区显示 runtime 状态块。

**Step 2: 运行测试确认失败**

Run: `npm test -- src/components/shell.test.tsx`
Expected: 因组件不存在而失败

**Step 3: 写最小实现**

实现单窗口四区域结构：
- 概览
- 配置
- 日志
- 请求测试

**Step 4: 运行测试确认通过**

Run: `npm test -- src/components/shell.test.tsx`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src/App.tsx src/components src/styles.css
git commit -m "feat: add desktop management UI shell"
```

### Task 8: 打通配置页与本地保存，移除用户自填 Python 路径

**Files:**
- Modify: `src/components/config-panel.tsx`
- Modify: `src/lib/api.ts`
- Create: `src/components/config-panel.test.tsx`

**Step 1: 写失败测试**

验证：
- Python 路径变为只读展示
- 能新增模型项
- 能触发保存

**Step 2: 运行测试确认失败**

Run: `npm test -- src/components/config-panel.test.tsx`
Expected: 因交互逻辑缺失而失败

**Step 3: 写最小实现**

通过 Tauri command：
- 加载配置
- 保存配置
- 保存时刷新 YAML
- 保存时强制回填 runtime Python 路径

**Step 4: 运行测试确认通过**

Run: `npm test -- src/components/config-panel.test.tsx`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src/components/config-panel.tsx src/components/config-panel.test.tsx src/lib/api.ts
git commit -m "feat: wire config editor to local persistence"
```

### Task 9: 打通概览页的 runtime 自举与进程控制

**Files:**
- Modify: `src/components/overview-panel.tsx`
- Modify: `src/lib/api.ts`
- Create: `src/components/overview-panel.test.tsx`

**Step 1: 写失败测试**

验证概览页能：
- 展示 runtime 状态
- 自动触发 bootstrap
- 展示启动中/安装中/失败文案
- 触发启动、停止、重启

**Step 2: 运行测试确认失败**

Run: `npm test -- src/components/overview-panel.test.tsx`
Expected: 因按钮和状态逻辑未实现而失败

**Step 3: 写最小实现**

接入：
- `bootstrap_runtime`
- `get_runtime_status`
- `check_environment`
- `get_proxy_status`
- `start_proxy`
- `stop_proxy`
- `restart_proxy`

**Step 4: 运行测试确认通过**

Run: `npm test -- src/components/overview-panel.test.tsx`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src/components/overview-panel.tsx src/components/overview-panel.test.tsx src/lib/api.ts
git commit -m "feat: add proxy controls to overview panel"
```

### Task 10: 打通日志面板订阅

**Files:**
- Modify: `src/components/logs-panel.tsx`
- Modify: `src/lib/api.ts`
- Create: `src/components/logs-panel.test.tsx`

**Step 1: 写失败测试**

验证日志流渲染、清空和关键字过滤。

**Step 2: 运行测试确认失败**

Run: `npm test -- src/components/logs-panel.test.tsx`
Expected: 因日志订阅逻辑不存在而失败

**Step 3: 写最小实现**

订阅 Tauri log event，并维护前端日志缓冲区。

**Step 4: 运行测试确认通过**

Run: `npm test -- src/components/logs-panel.test.tsx`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src/components/logs-panel.tsx src/components/logs-panel.test.tsx src/lib/api.ts
git commit -m "feat: add live proxy logs panel"
```

### Task 11: 实现测试请求命令与测试面板

**Files:**
- Create: `src-tauri/src/test_request.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src/components/test-panel.tsx`
- Create: `src/components/test-panel.test.tsx`
- Test: `src-tauri/src/test_request.rs`

**Step 1: 写失败测试**

Rust 侧测试：
- 正确构造本地 `chat/completions` 请求
- 正确返回统一结果结构

前端测试：
- 可提交消息
- 可展示成功或失败结果

**Step 2: 运行测试确认失败**

Run: `cd src-tauri && cargo test test_request -- --nocapture`
Expected: 因请求逻辑缺失而失败

Run: `npm test -- src/components/test-panel.test.tsx`
Expected: 因前端交互未实现而失败

**Step 3: 写最小实现**

Rust 暴露 `test_proxy_request` 命令，使用 `reqwest` 调本地代理；前端接入表单和结果展示。

**Step 4: 运行测试确认通过**

Run: `cd src-tauri && cargo test test_request -- --nocapture`
Expected: 测试通过

Run: `npm test -- src/components/test-panel.test.tsx`
Expected: 测试通过

**Step 5: Commit**

```bash
git add src-tauri/src/test_request.rs src-tauri/src/main.rs src/components/test-panel.tsx src/components/test-panel.test.tsx
git commit -m "feat: add local proxy request tester"
```

### Task 12: 补齐 runtime 修复、错误提示与收尾验证

**Files:**
- Modify: `src-tauri/src/proxy_manager.rs`
- Modify: `src/components/overview-panel.tsx`
- Modify: `src/components/test-panel.tsx`
- Modify: `README.md`

**Step 1: 写失败测试**

为常见错误路径写测试：
- Python 缺失
- runtime 创建失败
- `pip install` 失败
- runtime 损坏
- 端口冲突
- 本地请求超时

**Step 2: 运行测试确认失败**

Run: `npm test`
Expected: 至少新增错误路径测试失败

**Step 3: 写最小实现**

补齐用户可执行的错误提示文案、runtime 自动修复逻辑与概览页状态映射，补写本地运行说明。

**Step 4: 运行完整验证**

Run: `npm test`
Expected: 前端测试全部通过

Run: `cd src-tauri && cargo test -- --nocapture`
Expected: Rust 测试全部通过

Run: `npm run tauri build`
Expected: macOS 桌面应用构建成功

**Step 5: Commit**

```bash
git add .
git commit -m "feat: complete litellm desktop manager v1"
```
