# Contributing to WinSweep

感谢你愿意为 WinSweep 做出贡献！本文档说明参与协作的基本约定。

## 环境准备

- **Windows 10/11 x64**
- **Rust**（stable，MSVC 工具链，`x86_64-pc-windows-msvc`）
- **Node.js ≥ 18**（仅用于 `@tauri-apps/cli`）
- **Python 3**（仅用于生成图标源图，可选）

```bash
npm install
# 若图标缺失（icon.ico 不存在会导致 Windows 资源编译失败）：
python scripts/gen_icon.py && cp scripts/app-icon.png src-tauri/app-icon.png
npx tauri icon src-tauri/app-icon.png --output src-tauri/icons
# 构建
cd src-tauri && cargo build --release
```

## 提交规范

- 提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/)：
  `feat:` / `fix:` / `docs:` / `refactor:` / `chore:` / `ci:` 等。
- 一个提交聚焦一件事，尽量保持原子性。

## 代码风格

- **Rust**：`cargo fmt` + `cargo clippy`，保持无警告。
- **前端**：`ui/` 为原生 HTML/CSS/JS，遵循现有模块划分（`api/state/format/render/main`）。
- 所有文件统一 **UTF-8** 编码，中文内容不得出现乱码。

## 安全红线（务必遵守）

WinSweep 的核心价值是「绝不误删」，以下逻辑**不得弱化**：

- 受保护路径黑名单、允许根双重校验；
- 清理前的预览清单与二次确认；
- 默认「移入回收站」；永久删除需显式选择 + 二次确认；
- 实时监控在**类型层禁止永久删除**；
- 高风险类别默认不勾选。

## 不要提交的内容

`node_modules/`、`src-tauri/target/`、`src-tauri/gen/`、`release/*.zip`、`*.log`
以及任何本机隐私路径信息（详见 `.gitignore`）。

## Pull Request

1. Fork 并从 `main` 创建分支；
2. 完成改动并自测（`cargo build --release` 通过）；
3. 按模板填写 PR 描述，关联相关 Issue；
4. 等待 Review。

## 许可

向本项目提交贡献即表示你同意以 [MIT License](./LICENSE) 授权你的贡献。
Copyright (c) 2026 在心flow.
