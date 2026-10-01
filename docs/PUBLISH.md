# WinSweep 发布手册（PUBLISH）

> 适用范围：把 WinSweep v0.1.0 发布到 GitHub Releases / winget / Scoop / Chocolatey / Microsoft Store / 国内应用市场。
> 署名统一：**在心flow** ｜ 许可：MIT ｜ 便携 exe：`winsweep.exe`（2,970,112 B，sha256 `e2777f6482e12f0d091abec8a482e88d877fb44b74c4a404bc1d8506ad56c63b`）。

## 0. 当前状态

| 项 | 状态 |
|---|---|
| 本地 git 仓库 | ✅ 已初始化（`main`，2 次提交，`cb22a26 → e80722a`，**无 remote**） |
| 发布包 | ✅ `release/WinSweep-v0.1.0-win-x64.zip`（1,657,370 B，sha256 `3582e816b4c0063fa59d41364b4ffcd00c5442c7c46ae5d3b7dfd8b74b9047e7`） |
| 文档 | ✅ README（中英）、LICENSE、CHANGELOG、CONTRIBUTING、`docs/STORE-LISTING.md`（各渠道文案）、`docs/screenshots/`（3 张真实截图） |
| CI | ✅ `.github/workflows/build.yml`（windows-latest 构建 + tag 自动发 Release，已含 `permissions: contents: write`） |
| 隐私卫生 | ✅ 全仓无个人绝对路径/用户名残留 |

---

## 1. GitHub Releases（首要渠道）

### 需要你提供
1. **GitHub 账号授权**（三选一）
   - **A（推荐）**：在 WorkBuddy「连接器」面板授权 **GitHub**，之后由助手代为建仓/推送/发版。
   - **B**：在 `https://github.com/settings/tokens` 生成 classic token，勾选 **`repo` + `workflow`**，把 token 交给助手（用完可在同页 **Revoke**）。
   - **C**：本地安装 gh 并登录：`winget install GitHub.cli` → `gh auth login`。
2. **你的 GitHub 用户名或组织名**（决定仓库地址 `github.com/<你>/winsweep`）。
3. （可选）仓库名，默认 `winsweep`；是否**公开**（Public）仓库。

### 授权后可执行的命令（方式 C 示例）
```bash
cd "C:\Users\<用户名>\Desktop\垃圾检测清理工具"

# 1) 关联远端（把 <你> 换成你的用户名/组织）
git remote add origin https://github.com/<你>/winsweep.git

# 2) 推送主分支
git branch -M main
git push -u origin main

# 3) 打 tag 并推送（会触发 CI 自动构建并创建 Release）
git tag v0.1.0
git push origin v0.1.0

# 4) 若走 gh，也可直接发 Release 并附 exe
gh release create v0.1.0 "release/WinSweep-v0.1.0-win-x64.zip" \
  --title "WinSweep v0.1.0" --notes-file docs/STORE-LISTING.md
```

### 发布前建议在 GitHub 仓库设置
- **Description**：见 `docs/STORE-LISTING.md` §1.1 / §1.2
- **Topics**：见 §1.3（15 个）
- **Release 说明**：见 §6.1

---

## 2. winget / Scoop / Chocolatey（开发者渠道）

> 均需**以你的 GitHub 账号**提交 PR 或维护一个仓库；助手可生成全部清单文件。

| 渠道 | 提交方式 | 需要你提供 |
|---|---|---|
| **winget** | 向 `microsoft/winget-pkgs` 提 PR（`ZaiXinFlow.WinSweep`） | GitHub 授权即可（助手可生成 manifest 并提 PR） |
| **Scoop** | 自建 bucket 仓库或提 PR 到 `ScoopInstaller/Extras` | 你的 GitHub 账号 + bucket 仓库名 |
| **Chocolatey** | 在 chocolatey.org 注册后 `choco push` | Chocolatey API Key（个人免费账号可申请） |

manifest 关键字段（详见 `docs/STORE-LISTING.md` §6.2）：`Publisher=在心flow`、`License=MIT`、`PackageIdentifier=ZaiXinFlow.WinSweep`。

> ⚠️ winget 要求安装包有**稳定可下载 URL**（通常指向 GitHub Release 资产），所以请**先完成第 1 步**。

---

## 3. Microsoft Store（需开发者账号）

**你需要提供 / 完成：**
1. **微软合作伙伴中心（Partner Center）开发者账号** —— 个人约 **$19 一次性**，需实名。
2. **MSIX 打包 + 代码签名证书**（Store 提交要求 MSIX；当前产物是便携 exe，需新增打包步骤）。
3. Store listing：可直接复用 `docs/STORE-LISTING.md` §6.3（含系统要求 + 隐私说明）。

**助手可先做**：把 Tauri 的 `bundle.targets` 加上 `msix`、补 `bundle.publisher` 等配置并重新打包（**需要你确认后**才会重编 exe）。

---

## 4. 国内应用市场（应用宝 / 360 / 腾讯软件中心等）

**现实边界**：基本都要求**企业实名 + 人工审核**，通常无法由助手代提交。

**助手可先备齐的材料**：安装包/便携包、应用图标（多尺寸）、`docs/STORE-LISTING.md` §6.4 通用文案、§7 安全/隐私声明段落、截图（3 张，多为 1280×720 以上要求，可按需重截）。

---

## 5. 发布前检查清单

- [ ] 已选定 GitHub 授权方式，拿到用户名
- [ ] 隐私卫生：`git grep -n "13419"` 为空 ✅（已达成）
- [ ] exe sha256 与 Release 说明一致 ✅
- [ ] `permissions: contents: write` 已配置 ✅
- [ ] （可选）为 exe 做代码签名，避免 SmartScreen 拦截
- [ ] （可选）在 Release 说明中注明"定时任务/CLI/安装包属后续规划"

---

## 6. 给助手的常用指令

- 「用 GitHub 连接器把仓库推到 `<用户名>/winsweep` 并发布 v0.1.0」→ 助手建仓、推送、打 tag、发 Release
- 「生成 winget manifest 并提 PR」→ 助手生成 `ZaiXinFlow.WinSweep` 三件套并提交 PR
- 「帮我加 MSIX 打包并重新构建」→ 助手改 `tauri.conf.json` 并重编
