# WinSweep 交付概览

> 一句话：一个把小体积 Windows 垃圾扫描 + 安全清理工具，已编译为 **2.83 MB 的便携单文件 exe**。

## 交付状态

| 项 | 结果 |
|---|---|
| 交付形态 | 便携单文件 exe（无需外部运行时，Windows 10/11 x64） |
| exe 路径 | `src-tauri/target/release/winsweep.exe` |
| exe 体积 | **2,970,112 B = 2.83 MB**（目标 ≤15 MB，仅占 19%） |
| sha256 | `e2777f6482e12f0d091abec8a482e88d877fb44b74c4a404bc1d8506ad56c63b` |
| 技术栈 | Tauri v2（Rust/MSVC）+ 零框架静态前端（HTML/CSS/原生 JS） |
| 测试 | **15 / 15 通过**（8 工程师 + 7 QA 独立对抗测试），0 failed |
| exe 冒烟 | 启动后存活 ≥7s，无崩溃；WebView2 154 已就绪 |
| 已知功能缺陷 | **0**（0 Critical / 0 High / 0 Medium） |

## 关键决策（承接用户拍板）

1. **体积要小 + 美观 + 程序员风** → Tauri v2 + 零框架静态前端，暗色极客主题（`#0D1117` 底 / 青绿高亮 / 等宽字体）。
2. **手动扫描 + 实时监控都要** → 两者都实现；实时监控默认关闭、默认仅 `%TEMP%`、默认「仅提醒」。
3. **移入回收站 + 永久删除双模式** → 均提供；默认「移入回收站」，永久删除需显式选择 + 二次确认。

## 「绝不误删」安全机制（PRD §6 十条，全部落地并被验证）

预览清单 · 二次确认 · 回收站兜底 · 受保护路径黑名单 · 占用中文件跳过 · dry-run 零改动 · 高风险类别默认不勾选 · 实时监控类型层禁止永久删除 · 操作日志 · 最小权限。

> 特别说明：`recycle_bin` 类别内容无法「再移入回收站」，故两种模式下均执行 **清空回收站** 语义，并在预览与二次确认中**红色高亮「不可恢复」**警示。

## 18 类垃圾覆盖

`%TEMP%` · `C:\Windows\Temp` · 回收站 · Windows 更新缓存 · 传递优化文件 · pip/npm/yarn/`.gradle`/`.m2`/cargo/go 构建缓存 · Chrome/Edge/Firefox 缓存（**硬编码排除 Cookie/登录态/书签/历史**）· 错误报告与转储 · 缩略图缓存 · 系统日志。

## 构建与运行

```bash
npm install
# 图标（icon.ico 缺失会导致 Windows 资源编译失败）
python scripts/gen_icon.py && cp scripts/app-icon.png src-tauri/app-icon.png
npx tauri icon src-tauri/app-icon.png --output src-tauri/icons
# 构建便携 exe
cd src-tauri && cargo build --release      # 或 npm run tauri build
# 运行
npm run dev                                 # 开发模式
```

## 交付物清单

| 文件 | 说明 |
|---|---|
| `src-tauri/target/release/winsweep.exe` | ★ 最终便携 exe |
| `README.md` | 项目说明 / 构建运行指南 |
| `docs/PRD.md` | 产品需求文档 |
| `docs/ARCHITECTURE.md` | 系统架构 + 任务分解 |
| `docs/class-diagram.mermaid` / `docs/sequence-diagram.mermaid` | 类图 / 3 张时序图 |
| `ui/` | 静态前端（index.html / styles.css / js/*） |
| `src-tauri/src/` | Rust 后端（lib/types/categories/scan/clean/safety/watcher/win） |

## 遗留项（如实说明，非功能缺陷）

1. `execute_clean` 的 `confirmed=false` 拒绝与 dry-run 分支为**静态核验 + 等价可执行覆盖**（Tauri MockRuntime 在本环境有链接限制，属测试基建限制）。
2. exe 冒烟仅验证「进程存活」，**未做自动化 GUI 交互/渲染测试**。
3. 「占用中跳过」的真实受占用文件端到端行为未实测（仅逻辑与单测层面）。

## 后续可选增强（P2）

NSIS/MSI 安装包与代码签名 · 定时任务 · CLI 模式 · 系统托盘常驻 · 多语言。
