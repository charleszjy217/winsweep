# WinSweep 上架 / 推广文案集（STORE-LISTING）

> 版本：v0.1.0 ｜ 发布者署名：**在心flow** ｜ 许可：MIT（开源）
> 用途：GitHub 仓库、GitHub Releases、winget / Scoop / Chocolatey、Microsoft Store、国内应用市场通用文案。
> 说明：所有平台文案中，产品名统一为 **WinSweep**，作者 / 发布者统一署名 **在心flow**。标注「规划中」的能力（定时任务 / CLI / 安装包）不进入首版宣传口径。

---

## 1. GitHub 仓库元数据

### 1.1 Repo Description（中）

```
WinSweep — 面向开发者与 Windows 高级用户的暗色极客风磁盘垃圾扫描与安全清理工具。覆盖 18 类垃圾（系统 / 开发 / 浏览器缓存），便携单文件 exe 仅 2.83MB。默认移入回收站可恢复，不可逆删除需二次确认，机制上杜绝误删。MIT 开源，完全本地运行。
```

`140 字符（上限 350）`

### 1.2 Repo Description（EN）

```
WinSweep — a dark, geek-style Windows junk scanner & safe cleaner for developers and power users. 18 junk types (system / dev / browser caches). Portable single-file exe, just 2.83MB. Move-to-Recycle-Bin by default; irreversible deletes need an explicit second confirm. MIT, 100% offline, open source.
```

`301 字符（上限 350）`

### 1.3 推荐 Topics 标签（10–15 个，英文小写连字符）

```
windows  rust  tauri  disk-cleaner  junk-cleaner  system-cleanup  cache-cleaner  temp-files  developer-tools  portable-app  recycle-bin  storage-cleaner  privacy  open-source  desktop-app
```

`共 15 个`

### 1.4 README 顶部一句话 Tagline

**中**：`扫得全 · 看得清 · 删得稳 —— 2.83MB 的 Windows 磁盘垃圾扫描与安全清理工具`

**EN**：`Scan it all. See it clearly. Delete it safely. — a 2.83MB Windows junk scanner & safe cleaner.`

---

## 2. 短描述（应用市场列表页，≤80 字 / ≤80 字符）

**中（≤80 字）**
```
极客风 Windows 垃圾扫描与安全清理工具：18 类垃圾一键扫，单文件 exe 仅 2.83MB，默认移入回收站可恢复。
```
`62 字（含标点，上限 80）`

**EN（≤80 字符）**
```
Windows junk scanner & safe cleaner. 18 types, 2.83MB exe, Recycle-Bin default.
```
`79 字符`

---

## 3. 长描述（300–600 字，结构化）

### 3.1 中文版

**WinSweep · 扫得全、看得清、删得稳的 Windows 磁盘清理工具**

**产品定位**
WinSweep 是一款暗色极客风的 Windows 磁盘垃圾扫描与安全清理工具，面向开发者、Windows 高级用户与 C 盘告急的普通用户，以 2.83MB 便携单文件 exe 交付。

**核心功能**
- **18 类垃圾，一站扫全**：系统与用户临时目录、回收站、Windows 更新缓存、传递优化文件；pip / npm / yarn / .gradle / .m2 / cargo / go 构建缓存；Chrome / Edge / Firefox 缓存；错误报告与转储、缩略图缓存、系统日志。
- **看得清**：按类别列出大小、文件数与安全等级，可逐项勾选；清理前提供可搜索、可导出的**全量预览清单**。
- **删得稳**：默认**移入回收站**（可恢复），永久删除需二次确认；内置黑名单、占用跳过、dry-run 模拟清理。
- **实时监控**：可选开启、默认关闭，仅支持提醒或移入回收站，不含永久删除。

**为什么安全**
预览清单 → 二次确认 → 回收站兜底 → 黑名单 → 占用跳过 → dry-run，六重机制从设计上**杜绝误删**；浏览器清理**只动缓存，绝不触碰 Cookie、登录态与书签**。

**适用人群**
开发者（清理数 GB 的 pip、npm、gradle 缓存而不破坏依赖）、Windows 高级用户、C 盘告急的普通用户、测试与运维人员。

**体积与性能**
便携单文件 exe 仅 **2.83MB**，完全本地运行、不联网、不上传数据，MIT 开源可审计。

`591 字（含标点，区间 300–600）`

### 3.2 English Version

**WinSweep — a Windows junk cleaner that scans it all, shows it clearly, deletes it safely**

**Positioning**
WinSweep is a dark, geek-styled Windows disk-junk scanner and safe cleaner, built for developers, power users, and anyone whose C: drive is running out of space. It ships as a **2.83MB portable single-file exe** — no installer, no runtime required.

**Core features**
- **18 junk types, one scan**: system & user temp dirs, Recycle Bin, Windows Update cache, Delivery Optimization files; pip / npm / yarn / .gradle / .m2 / cargo / go build caches; Chrome / Edge / Firefox caches; error reports & dumps, thumbnail cache, system logs.
- **See it clearly**: results list size, file count and risk level per category, with per-item checkboxes and select-all / invert; a **full preview manifest** (searchable, exportable to .txt) before any cleanup.
- **Delete it safely**: **move to Recycle Bin** by default (recoverable); permanent deletion requires an explicit choice plus a second confirmation. Built-in protected-path blacklist, in-use files are skipped, and dry-run makes zero changes.
- **Realtime monitor** (opt-in, off by default): new junk in watched folders can be "notify only" or "auto move to Recycle Bin" — **no permanent delete path exists**.

**Why it's safe**
Preview manifest → second confirmation → Recycle Bin fallback → blacklist → in-use skip → dry-run: six layers designed to make accidental deletion impossible. Browser cleanup **touches caches only — never cookies, sessions or bookmarks**.

**Who it's for**
Developers (clear multi-GB pip / npm / gradle caches without breaking deps), Windows power users, everyday users out of disk space, and QA / ops teams cleaning many machines.

**Size & performance**
A portable 2.83MB single exe, fast cold start, low CPU under monitoring. Fully local, offline, zero data upload, MIT and open source — auditable.

`约 290 words`

---

## 4. 功能要点清单（商店 / 市场 bullet，8–12 条，每条 ≤20 字）

1. 18 类垃圾一键扫全
2. 便携单文件 exe 仅 2.83MB
3. 开发缓存专项清理
4. 浏览器缓存只清缓存
5. 清理前全量预览清单
6. 默认移入回收站可恢复
7. 永久删除需二次确认
8. 受保护路径黑名单保护
9. 占用中文件自动跳过
10. dry-run 模拟清理零改动
11. 实时监控可开关默认关
12. 完全本地零数据上传

`共 12 条`

---

## 5. 更新日志 · v0.1.0 Release Notes

### 5.1 中文版 (面向用户)

**WinSweep v0.1.0 — 首个公开发布版本** 🎉

这是我们第一次把一个完整可用的 WinSweep 交到你手上。

**新增**
- **18 类垃圾扫描**：系统 / 用户临时目录、回收站、Windows 更新缓存、传递优化文件；pip / npm / yarn / .gradle / .m2 / cargo / go 构建缓存；Chrome / Edge / Firefox 缓存；错误报告与转储、缩略图缓存、系统日志。
- **安全清理**：默认「移入回收站」可恢复；「永久删除」需显式选择 + 二次确认。
- **六重防误删机制**：预览清单、二次确认、回收站兜底、受保护路径黑名单、占用中文件跳过、dry-run 模拟清理。
- **结果可视化**：按类别展示大小 / 文件数 / 安全等级，支持勾选、全选 / 反选、预览清单导出。
- **实时监控（可选）**：默认关闭，支持「仅提醒」与「自动移入回收站」，不含永久删除。

**亮点**
- 暗色极客风界面，等宽字体 + 终端气质。
- 便携单文件 exe，体积仅 **2.83MB**，无需安装运行时。
- 完全本地运行、不联网、不上传任何数据；MIT 开源，代码可审计。

**说明**
- 定时任务、CLI 模式、安装包（MSI / NSIS）与代码签名属**后续版本规划**，本版未包含。
- 浏览器清理只处理缓存，不会删除 Cookie、登录态或书签。

*发布者：在心flow* ｜ *许可：MIT*

### 5.2 English Version (User-facing)

**WinSweep v0.1.0 — First public release** 🎉

The first complete, ready-to-use WinSweep is here.

**Added**
- **18 junk types**: system & user temp dirs, Recycle Bin, Windows Update cache, Delivery Optimization files; pip / npm / yarn / .gradle / .m2 / cargo / go build caches; Chrome / Edge / Firefox caches; error reports & dumps, thumbnail cache, system logs.
- **Safe cleanup**: "Move to Recycle Bin" by default (recoverable); "Permanent delete" needs an explicit choice + a second confirmation.
- **Six layers against accidental deletion**: preview manifest, second confirmation, Recycle Bin fallback, protected-path blacklist, in-use file skipping, and dry-run.
- **Clear results**: size / file count / risk level per category, with checkboxes, select-all / invert, and exportable preview manifest.
- **Realtime monitor (optional)**: off by default; "notify only" or "auto move to Recycle Bin" — no permanent delete path.

**Highlights**
- Dark, geek-style UI with monospace typography and a terminal feel.
- Portable single-file exe, only **2.83MB** — no runtime to install.
- Fully local and offline, zero data upload; MIT, open source, auditable.

**Notes**
- Scheduled tasks, CLI mode, installers (MSI / NSIS) and code signing are **planned for a later release** — not in this version.
- Browser cleanup handles caches only; it never removes cookies, sessions or bookmarks.

*Published by 在心flow* ｜ *License: MIT*

---

## 6. 各渠道适配

### 6.1 GitHub Releases（Markdown 版发布说明）

```markdown
## WinSweep v0.1.0 · 扫得全 · 看得清 · 删得稳

暗色极客风的 Windows 磁盘垃圾扫描与安全清理工具，首个公开发布版本。

### 亮点
- **扫得全**：18 类垃圾一站覆盖 —— 系统 / 用户临时目录、回收站、Windows 更新缓存、传递优化文件、pip/npm/yarn/.gradle/.m2/cargo/go 构建缓存、Chrome/Edge/Firefox 缓存、错误报告与转储、缩略图缓存、系统日志。
- **看得清**：按类别展示大小 / 文件数 / 安全等级，清理前全量预览（可搜索、可导出）。
- **删得稳**：默认移入回收站可恢复；永久删除需二次确认；黑名单 + 占用跳过 + dry-run 六重防护。
- **够轻**：便携单文件 exe，仅 **2.83MB**，无需安装运行时。

### 下载
| 文件 | 平台 | 说明 |
|---|---|---|
| `winsweep.exe` | Windows 10/11 x64 | 便携版，双击即用 |

`sha256: e2777f6482e12f0d091abec8a482e88d877fb44b74c4a404bc1d8506ad56c63b`

### 安全性
完全本地运行 · 不联网 · 不上传任何数据 · 浏览器清理只动缓存（绝不动 Cookie / 登录态 / 书签）· MIT 开源可审计。

### 已知说明
定时任务 / CLI / 安装包与代码签名属后续规划，本版未包含。

---
*发布者：在心flow ｜ 许可：MIT* ｜ 完整说明见 `docs/OVERVIEW.md`
```

### 6.2 winget / Scoop / Chocolatey（简短包描述 + 关键词）

**Package Description（EN，一句话，供 manifest `Description` 字段）**
```
WinSweep is a dark-styled, portable Windows junk scanner and safe cleaner. It covers 18 junk types (system, developer and browser caches), moves files to the Recycle Bin by default, and requires explicit confirmation for permanent deletes. Single-file exe, only 2.83MB. Fully local, no data upload, MIT licensed.
```

**中文包描述（国内包索引 / 镜像可用）**
```
WinSweep 是一款暗色极客风的 Windows 磁盘垃圾扫描与安全清理工具。覆盖 18 类垃圾（系统 / 开发 / 浏览器缓存），默认移入回收站可恢复，永久删除需二次确认。便携单文件 exe 仅 2.83MB，完全本地运行、不联网、MIT 开源。
```

**建议关键词（manifest tags / keywords）**
```
winsweep, disk-cleaner, junk-cleaner, cache-cleaner, temp-files, system-cleanup, developer-tools, portable, recycle-bin, tauri, rust
```

**Manifest 关键字段建议**
| 字段 | 建议值 |
|---|---|
| `PackageIdentifier` | `ZaiXinFlow.WinSweep` |
| `Publisher` | `在心flow` |
| `PackageName` | `WinSweep` |
| `License` | `MIT` |
| `ShortDescription` | 见 6.2 英文一句话 |
| `Tags` | 见 6.2 关键词 |

### 6.3 Microsoft Store（正式商店介绍）

**应用名称**：WinSweep
**发布者**：在心flow ｜ **版本**：0.1.0 ｜ **许可**：MIT（开源）

**产品简介**
WinSweep 是一款面向 Windows 10/11 的磁盘垃圾扫描与安全清理工具，采用暗色极客风界面，以 2.83MB 的便携单文件形式交付。它帮助开发者与普通用户扫描并安全释放系统、开发工具链与浏览器产生的冗余缓存，在释放磁盘空间的同时，从机制上杜绝误删。

**主要功能**
- **18 类垃圾覆盖**：系统 / 用户临时目录、回收站、Windows 更新缓存、传递优化文件；pip / npm / yarn / .gradle / .m2 / cargo / go 构建缓存；Chrome / Edge / Firefox 缓存；错误报告与转储、缩略图缓存、系统日志。
- **安全清理**：默认「移入回收站」可恢复；「永久删除」须显式选择并二次确认。
- **预览与审计**：清理前生成可搜索、可导出的完整预览清单。
- **防护机制**：受保护路径黑名单、占用中文件跳过、dry-run 模拟清理。
- **实时监控**：可选开启，默认关闭；仅支持提醒或移入回收站，不含永久删除。

**系统要求**
| 项 | 要求 |
|---|---|
| 操作系统 | Windows 10 1809（x64）及以上 / Windows 11 |
| 架构 | x64 |
| 运行时 | 无（便携单文件 exe，无需额外依赖） |
| 磁盘占用 | 约 3 MB |
| 网络 | 不需要（工具完全离线运行） |

**隐私说明**
本工具**完全本地运行、不联网、不上传任何数据**。它只读取本机磁盘上的垃圾文件信息用于展示与清理，不会采集、传输或共享任何个人信息。所有扫描与清理行为均在本机完成，浏览器缓存清理仅针对缓存文件，绝不访问或留存 Cookie、登录态、书签等个人数据。

**为什么选择 WinSweep**
- 一键释放 GB 级磁盘空间，扫得全、看得清、删得稳。
- 便携无依赖，下载即可运行。
- 开源可审计，透明可信。

### 6.4 国内应用市场通用版（应用宝 / 360 / 腾讯软件中心等）

**应用名称**：WinSweep · Windows 垃圾清理
**开发者 / 发布者**：在心flow
**软件大小**：2.83 MB ｜ **版本**：0.1.0 ｜ **授权**：免费 · 开源（MIT）

**应用简介**
WinSweep 是一款轻量、安全的 Windows 磁盘清理工具。它能一次性扫描系统、开发工具与浏览器产生的 18 类冗余缓存，帮助你在几秒内释放被占用的磁盘空间，界面采用清爽的暗色风格，简单易懂。

**功能特色**
- **一键扫描，18 类垃圾全搞定**：临时文件、回收站、系统更新缓存、开发缓存、浏览器缓存等一并呈现。
- **清理前先预览，绝不乱删**：动手前把要删除的清单完整列出，看得清、可导出，放心点确认。
- **默认移到回收站，删错了能还原**：默认不永久删除，误删可从回收站找回；确需彻底删除时，软件会再次向你确认。
- **绿色便携，无依赖**：单个文件即可运行，体积仅 2.83 MB，无需安装额外组件。
- **完全本地，干净放心**：软件在你自己电脑上运行，**不联网、不上传任何数据、无广告、无捆绑、无后台推广**。

**安全与合规**
- 本地运行：所有扫描与清理均在本机完成，数据不出本机。
- 零上传：不采集、不传输任何个人信息与文件内容。
- 可恢复：默认清理为「移入回收站」，支持还原。
- 开源可审计：源码公开（MIT 许可），欢迎监督。

**适用人群**：程序员 / 开发者、Windows 高级用户、C 盘空间不足的普通用户、测试与运维人员。

**为什么值得信赖**
- 只清理系统与软件的临时 / 缓存文件，不碰你的文档、照片与个人资料。
- 清理浏览器缓存时只处理缓存，不删除 Cookie、登录信息与书签。
- 开源透明、无广告、无捆绑，安装与运行全程干净。

---

## 7. 必备合规 / 安全声明段落（各市场可复用）

> **安全与隐私声明（标准段落）**
>
> WinSweep **完全本地运行，不联网、不上传任何数据**。所有扫描与清理动作均在你自己的电脑上完成：软件只在本地读取垃圾文件的位置与大小用于展示和清理，不采集、不传输、不共享任何个人信息或文件内容。
>
> 本工具**默认可恢复**：清理默认采用「移入系统回收站」模式，误删可随时从回收站还原；仅当你显式选择「永久删除」并完成二次确认时，操作才不可逆。
>
> 本工具**开源可审计**：代码以 MIT 许可公开，任何人可查阅、验证其行为；无广告、无捆绑、无后台推广、无数据回传。
>
> 本工具在清理浏览器缓存时**只处理缓存文件，绝不删除 Cookie、登录状态或书签**等个人数据。受保护的系统目录与关键组件列入黑名单，永远不会进入可删除列表。

**English (reusable)**
> **Security & Privacy Statement**
>
> WinSweep runs **fully locally — it is offline and uploads no data**. All scanning and cleaning happen on your own machine: it reads only the location and size of junk files for display and cleanup, and never collects, transmits, or shares any personal information or file contents.
>
> **Recoverable by default**: cleanup uses "Move to Recycle Bin", so accidental deletions can be restored; an operation is irreversible only if you explicitly choose "Permanent delete" and pass a second confirmation.
>
> **Open source and auditable**: the code is published under the MIT license for anyone to inspect and verify. No ads, no bundles, no background promotion, no telemetry.
>
> When cleaning browser data, WinSweep **touches caches only — never cookies, sessions or bookmarks**. Protected system directories and critical components are blacklisted and never enter the deletable list.

---

## 8. 关键词 / 搜索词

### 8.1 中文（10 个）

```
Windows清理 磁盘清理工具 C盘清理 垃圾清理 临时文件清理 缓存清理 开发缓存清理 pip缓存 npm缓存 gradle缓存 便携清理工具
```

### 8.2 English (10)

```
windows cleaner  disk cleanup  junk cleaner  cache cleaner  temp file cleaner  developer cache  pip cache  npm cache  portable cleaner  recycle bin
```

---

## 附：文案使用备注

- **统一署名**：所有渠道「发布者 / 作者 / Publisher」一律为 **在心flow**。
- **不夸大**：定时任务、CLI、安装包（MSI / NSIS）、代码签名均标注为「后续规划」，首版文案不得出现「已支持」。
- **核心口径**：`扫得全 · 看得清 · 删得稳`；`2.83MB 便携单文件`；`默认移入回收站可恢复`；`完全本地 · 零上传 · 开源可审计`。
- **安全红线**：涉及浏览器必须写明「只清缓存，不动 Cookie / 登录态 / 书签」。

*文档作者：产品经理 许清楚 ｜ 产品：WinSweep v0.1.0 ｜ 发布者：在心flow*
