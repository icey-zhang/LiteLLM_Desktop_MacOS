# LiteLLM Desktop for macOS

一个基于 Tauri 2 + React 的 macOS 桌面应用，用图形界面管理本机 LiteLLM Proxy。

它的目标不是做聊天客户端，而是把原本需要在终端里手工完成的几件事收进一个窗口里：

- 检查系统 `python3`
- 自动创建 LiteLLM 专属虚拟环境
- 安装 `litellm[proxy]`
- 编辑 LiteLLM 配置
- 启动 / 停止 / 重启本地代理
- 查看实时日志
- 在应用内直接发起一次最小 `chat.completions` 测试请求

## 当前能力

当前版本已经实现：

- macOS 单平台桌面壳
- 首次启动自动创建应用专属 `.venv`
- 自动安装和修复 `litellm[proxy]`
- Provider 分组与模型映射配置
- 生成并持久化 LiteLLM 配置文件
- 本地 LiteLLM Proxy 进程控制
- stdout / stderr / system 日志面板
- 内置测试请求面板
- 基础健康检查与错误提示

## 技术栈

- Tauri 2
- React 19
- TypeScript
- Vite
- Rust

## 系统要求

- macOS 13.0 或更高版本
- 本机可执行的 `python3`
- Node.js 18+
- Rust toolchain
- Xcode Command Line Tools

说明：

- 应用不会自动安装 Python。
- LiteLLM 运行时会安装在应用自己的数据目录里，不依赖你手工维护项目级虚拟环境。

## 开发

安装依赖：

```bash
npm install
```

运行前端测试：

```bash
npm test
```

启动浏览器模式前端：

```bash
npm run dev
```

启动 Tauri 桌面开发模式：

```bash
npm run tauri -- dev
```

## 打包

构建前端：

```bash
npm run build
```

构建 macOS 桌面应用：

```bash
npm run tauri -- build
```

常见产物位于：

- `src-tauri/target/release/bundle/macos/`
- `src-tauri/target/release/bundle/dmg/`

Release 通常建议上传 `.dmg`，必要时也可以附带 `.app.tar.gz`。

## 应用如何工作

应用启动后会先检查本机 `python3`，然后在应用数据目录下准备自己的 LiteLLM runtime：

1. 创建 `runtime/.venv`
2. 升级 `pip`
3. 安装 `litellm[proxy]`
4. 用这个专属 Python 启动 LiteLLM Proxy

因此，真正负责运行代理的解释器不是你系统里任意一个 Python 路径，而是应用托管的 runtime Python。

## 应用数据目录

Tauri 标识符为 `ai.litellm.desktop`，macOS 下数据通常位于：

```text
~/Library/Application Support/ai.litellm.desktop/
```

目录内会包含类似文件：

- `app-config.json`
- `litellm-config.yaml`
- `runtime/.venv/`
- `logs/proxy.log`

## 项目结构

```text
src/                React 前端
src/components/     概览 / 配置 / 日志 / 测试 面板
src/lib/            Tauri API 封装、默认配置、声音反馈
src/types/          前后端共享配置类型
src-tauri/src/      Rust 后端命令、runtime、自举、进程管理
docs/plans/         设计与实现计划文档
```

## 使用流程

1. 启动应用
2. 等待运行环境检测 / 自动安装完成
3. 在“配置”页填写 Provider、API Key、模型映射和端口
4. 保存配置
5. 在“概览”页启动代理
6. 在“请求测试”页发起一条最小验证请求
7. 如需排障，在“日志”页查看实时输出

## 已知边界

当前版本有意保持收敛，暂不覆盖：

- Windows / Linux
- Docker 模式
- 自动安装 Python
- 多工作区 / 多 profile
- 内置聊天界面
- LiteLLM 高级路由策略编辑器

## 仓库说明

这个仓库用于保存源码；编译后的 macOS 安装包通过 GitHub Releases 分发。
