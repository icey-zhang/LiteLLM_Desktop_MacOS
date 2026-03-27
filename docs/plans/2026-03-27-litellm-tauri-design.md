# LiteLLM Tauri Desktop Design

## Goal

构建一个仅面向 macOS 的 Tauri 桌面应用，用于在本机管理 LiteLLM Proxy。第一版聚焦“应用自管运行时”：首次启动时自动在应用数据目录创建专属 `.venv` 并安装 `litellm[proxy]`，然后提供图形化配置、一键启动/停止代理、实时日志与桌面内测试请求。

## Scope

### In Scope

- macOS 单平台
- 依赖本机已有 `python3`
- 首次启动自动创建应用专属 `.venv`
- 首次启动自动安装 `litellm[proxy]`
- 桌面管理台而不是聊天客户端
- 图形化配置 LiteLLM 基础参数与至少一个模型条目
- 生成并持久化 LiteLLM `config.yaml`
- 启动、停止、重启 LiteLLM Proxy
- 实时查看 stdout/stderr 日志
- 从应用内发起最小 `chat.completions` 测试请求
- 代理健康检查与基础错误提示

### Out of Scope

- Windows / Linux 支持
- Docker 模式
- 自动安装 Python
- 多 profile / 多工作区
- 内置聊天界面
- 完整复刻 LiteLLM 官方 Admin UI
- 高级路由策略编辑器，例如 fallback、budget、rate limits、load balancing

## Product Shape

应用保持单窗口，主界面分成 4 个区域：

1. 概览
2. 配置
3. 日志
4. 请求测试

这样的边界可以保证第一版聚焦“可稳定管理本机 LiteLLM Proxy”，而不是扩展成完整的 LLM 工作台。

## Architecture

### Frontend

- 技术：React + TypeScript + Vite
- 负责：
  - 运行时安装状态展示
  - 表单输入与校验
  - 运行状态展示
  - 日志流渲染
  - 测试请求结果展示

### Tauri / Rust Layer

- 负责：
  - 系统 Python 检查
  - 专属 runtime `.venv` 生命周期管理
  - 自动安装 `litellm[proxy]`
  - 本地配置读写
  - LiteLLM 子进程生命周期管理
  - stdout/stderr 日志采集并推送给前端
  - 本地健康检查
  - 测试请求代理调用

### LiteLLM Process

- 由应用通过 Rust `Command` 启动
- Python 解释器固定使用应用 runtime 目录中的 `.venv/bin/python`
- 使用应用生成的 `litellm-config.yaml`
- 监听本地端口，默认 `127.0.0.1:4000`

## UX Design

### 概览

- 显示系统 Python 检查结果
- 显示 runtime 安装状态：未知、检查中、安装中、修复中、已就绪、失败
- 显示 runtime Python 路径
- 显示代理状态：未启动、启动中、运行中、失败
- 显示监听地址与端口
- 提供启动、停止、重启按钮

### 配置

- 基础字段：
  - `port`
  - `masterKey`
  - `autoStartProxy`
- 只读字段：
  - `runtimePythonPath`
- Provider 基础字段：
  - `provider`
  - `apiKey`
  - `apiBase`
- 模型列表字段：
  - `alias`
  - `litellmModel`
  - `apiBase`
  - `extraParams`
- 支持新增、删除、编辑模型项
- 保存后生成应用内部的结构化配置，并同步输出 LiteLLM YAML

### 日志

- 显示实时 stdout/stderr
- 支持清空当前视图
- 支持文本过滤
- 高亮错误日志

### 请求测试

- 选择模型别名
- 输入 system / user 消息
- 发起最小 `chat.completions` 请求到本地代理
- 展示：
  - 请求 URL
  - HTTP 状态码
  - 耗时
  - 返回 JSON
  - 错误消息

## Data Model

前端维护 GUI 友好的结构化配置，Rust 层再把它映射为 LiteLLM YAML。

### AppSettings

- `pythonPath: string`
- `port: number`
- `masterKey: string`
- `autoStartProxy: boolean`

### ProviderPreset

- `provider: string`
- `apiKey: string`
- `apiBase?: string`

### ModelEntry

- `id: string`
- `alias: string`
- `litellmModel: string`
- `provider: string`
- `apiKey?: string`
- `apiBase?: string`
- `extraParams: Record<string, string | number | boolean>`

### AppConfig

- `settings: AppSettings`
- `providers: ProviderPreset[]`
- `models: ModelEntry[]`

## File Layout

应用数据放在 Tauri app data 目录，而不是工作目录。

- `app-config.json`
- `litellm-config.yaml`
- `runtime/.venv/`
- `logs/proxy.log`

这样便于备份、迁移与后续增加 profile。

## Runtime Bootstrap

应用启动时先执行 runtime bootstrap，而不是直接检测 LiteLLM：

1. 检查系统 `python3` 是否可用
2. 检查 `~/Library/Application Support/ai.litellm.desktop/runtime/.venv` 是否存在
3. 如果不存在：
   - `python3 -m venv <runtime/.venv>`
   - `<runtime/.venv/bin/python> -m pip install -U pip`
   - `<runtime/.venv/bin/python> -m pip install "litellm[proxy]"`
4. 如果存在：
   - 校验 `<runtime/.venv/bin/python>` 是否可执行
   - 校验 `python -m pip show litellm`
5. 如果校验失败：
   - 标记 runtime 损坏
   - 自动尝试重建一次
6. 成功后：
   - 把应用配置中的 `pythonPath` 固定为 runtime 里的解释器
   - 后续所有代理启动都使用这个解释器

## Runtime State Machine

前端需要单独展示 runtime 状态机：

- `unknown`
- `checking`
- `installing`
- `ready`
- `repairing`
- `error`

UI 文案要围绕这些状态展示安装进度、失败原因与自动修复结果，而不是简单报“litellm 未安装”。

## Process Lifecycle

Rust 层维护一个单例 `ProxyManager`：

1. 确认 runtime 已就绪
2. 写出最新 `litellm-config.yaml`
3. 使用 runtime Python 启动子进程
4. 读取 stdout/stderr 并发到前端
5. 定期做健康检查
6. 停止时优雅结束，必要时强制 kill

## Health Check

最小健康检查策略：

- 先检查子进程是否仍然存活
- 再请求本地代理一个轻量接口或尝试 TCP/HTTP 连通性
- 如果进程存在但代理不可访问，状态标记为“异常”

## Error Handling

第一版要清晰处理这些错误：

- 未安装 Python
- 创建 `.venv` 失败
- `pip install "litellm[proxy]"` 失败
- runtime 损坏或不完整
- 端口被占用
- 配置序列化失败
- 进程启动后秒退
- 测试请求超时
- 上游 401 / 429 / 500

界面错误提示要包含可执行建议，例如安装命令或修改端口建议。

## Security Posture

第一版不做钥匙串集成，但应避免把密钥打印到日志中。日志流展示前需做基本脱敏，至少屏蔽：

- `api_key`
- `master_key`
- `Authorization` header

## Testing Strategy

第一版以三层验证为主：

1. 前端组件和状态逻辑单测
2. Rust 配置转换、runtime bootstrap 与命令接口单测
3. 轻量集成验证：
   - 能创建 runtime `.venv`
   - 能生成 YAML
   - 能拉起代理
   - 能发出本地测试请求

## Recommended Tech Stack

- Frontend: React + TypeScript + Vite
- UI: Tailwind CSS + shadcn/ui
- Rust: tauri, tokio, serde, serde_yaml, reqwest

## Future Extensions

如果第一版稳定，优先扩展这些能力：

1. 多 profile
2. Docker runner
3. 钥匙串存储
4. 最近请求记录
5. 直接跳转或嵌入 LiteLLM `/ui`
