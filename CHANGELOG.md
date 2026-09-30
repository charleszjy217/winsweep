# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-01

### Added

- **18 类垃圾扫描**：覆盖 `%TEMP%`、`C:\Windows\Temp`、回收站、Windows 更新缓存、
  传递优化文件、pip/npm/yarn/`.gradle`/`.m2`/cargo/go 构建缓存、Chrome/Edge/Firefox
  浏览器缓存（硬编码排除 Cookie / 登录态 / 书签 / 历史）、错误报告与转储、缩略图缓存、
  系统日志。
- **双模式清理**：默认「移入回收站」（可恢复兜底），并可选「永久删除」（需显式选择）。
- **预览 + 二次确认**：清理前提供全量可搜索预览清单，支持导出 `.txt`；执行前弹出
  二次确认，永久删除路径红色高亮「不可恢复」警示。
- **实时监控**：基于 `notify` 的文件系统监听 + 800ms 防抖，默认关闭、默认仅监控
  `%TEMP%`、默认「仅提醒」；**类型层禁止永久删除**。
- **安全守卫**：受保护路径黑名单、允许根双重校验、占用中文件跳过、`dry-run` 零改动、`.txt` 操作日志。
- **便携单文件 exe**：**2.83 MB**（2,970,112 B）绿色便携 exe，无需外部运行时，
  支持 Windows 10/11 x64。
- 暗色极客风 UI（`#0D1117` 底 / 青绿高亮 / 等宽字体），零框架静态前端。

### Notes

- exe 平台为 Windows x64（PE32+ GUI）。
- 本版本未包含安装包（NSIS/MSI）与代码签名，采用便携 zip 分发。
