//! 类型层：全部 DTO / 枚举 / 错误 / 应用状态。
//!
//! 契约（见架构 §3.1 / §8.2）：
//! - 所有跨 IPC 结构体 `#[serde(rename_all = "camelCase")]` → 前端统一 camelCase。
//! - 枚举 `#[serde(rename_all = "lowercase")]` / `camelCase` → 线上小写/小驼峰。
//! - 时间统一 ISO 8601 UTC 字符串；字节统一 u64。

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// ── 枚举 ────────────────────────────────────────────────

/// 安全风险等级：`"low" | "medium" | "high"`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// 清理模式：`"recycleBin" | "permanent"`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanMode {
    RecycleBin,
    Permanent,
}

/// 类别的清理语义。`EmptyRecycleBin` 用于回收站类别（内容无法“再次移入回收站”）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanSemantics {
    Normal,
    EmptyRecycleBin,
}

/// 扫描状态：`"ok" | "notFound" | "skipped" | "error"`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanStatus {
    Ok,
    NotFound,
    Skipped,
    Error,
}

/// 实时监控策略：**无 Permanent**（禁止实时永久删除）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MonitorPolicy {
    NotifyOnly,
    RecycleBin,
}

// ── 类别定义（静态）──────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub risk: RiskLevel,
    pub default_selected: bool,
    pub requires_admin: bool,
    pub semantics: CleanSemantics,
    pub description: String,
}

// ── 扫描结果 ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResult {
    pub id: String,
    pub name: String,
    pub risk: RiskLevel,
    pub default_selected: bool,
    pub requires_admin: bool,
    pub semantics: CleanSemantics,
    pub roots: Vec<String>,
    pub total_size: u64,
    pub file_count: u64,
    pub status: ScanStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub drive: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub scan_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub categories: Vec<CategoryResult>,
    pub total_size: u64,
    pub total_files: u64,
    pub disk: DiskInfo,
}

// ── 预览 ────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    pub category_ids: Vec<String>,
    pub mode: CleanMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewEntry {
    pub category_id: String,
    pub category_name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPlan {
    pub category_ids: Vec<String>,
    pub mode: CleanMode,
    pub entries: Vec<PreviewEntry>,
    pub entries_truncated: bool,
    pub total_files: u64,
    pub total_size: u64,
    pub has_high_risk: bool,
    pub warnings: Vec<String>,
}

// ── 清理 ────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanRequest {
    pub category_ids: Vec<String>,
    pub mode: CleanMode,
    #[serde(default)]
    pub dry_run: bool,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCleanStat {
    pub id: String,
    pub name: String,
    pub freed_bytes: u64,
    pub deleted_count: u64,
    pub skipped_count: u64,
    pub failed_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipRecord {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub cleaned_at: String,
    pub mode: CleanMode,
    pub dry_run: bool,
    pub freed_bytes: u64,
    pub deleted_count: u64,
    pub skipped_count: u64,
    pub failed_count: u64,
    pub per_category: Vec<CategoryCleanStat>,
    pub skipped_samples: Vec<SkipRecord>,
    pub notes: Vec<String>,
}

// ── 实时监控 ────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorConfig {
    pub dirs: Vec<String>,
    pub policy: MonitorPolicy,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorStatus {
    pub running: bool,
    pub dirs: Vec<String>,
    pub policy: MonitorPolicy,
    pub events_count: u64,
}

/// 监控命中事件载荷（`monitor://event`）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorEvent {
    pub path: String,
    pub size: u64,
    pub kind: String,
    pub action: String,
    pub ts: String,
}

// ── 进度事件载荷 ────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub scan_id: String,
    pub done_categories: usize,
    pub total_categories: usize,
    pub scanned_files: u64,
    pub scanned_bytes: u64,
    pub current_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgress {
    pub processed: usize,
    pub total: usize,
    pub freed_bytes: u64,
    pub current_path: String,
}

// ── 设置 ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomCategory {
    pub id: String,
    pub name: String,
    pub path: String,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub default_mode: CleanMode,
    pub monitor_enabled: bool,
    pub monitor_dirs: Vec<String>,
    pub monitor_policy: MonitorPolicy,
    pub custom_categories: Vec<CustomCategory>,
    pub theme: String,
    pub font: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_mode: CleanMode::RecycleBin,
            monitor_enabled: false,
            monitor_dirs: vec!["%TEMP%".into()],
            monitor_policy: MonitorPolicy::NotifyOnly,
            custom_categories: vec![],
            theme: "dark".into(),
            font: "Consolas".into(),
        }
    }
}

// ── 应用状态 ────────────────────────────────────────────

pub struct AppState {
    /// 取消当前 scan/clean。
    pub cancel: Arc<AtomicBool>,
    /// 互斥：同一时刻只允许一个重任务。
    pub busy: Arc<AtomicBool>,
    pub settings: Mutex<Settings>,
    pub monitor: Mutex<Option<crate::watcher::MonitorHandle>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
            busy: Arc::new(AtomicBool::new(false)),
            settings: Mutex::new(Settings::default()),
            monitor: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ── 错误 ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn io(e: std::io::Error) -> Self {
        Self::new(E_IO, e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::io(e)
    }
}

// 错误码常量（前端按此分支）
pub const E_BUSY: &str = "E_BUSY";
pub const E_NOT_CONFIRMED: &str = "E_NOT_CONFIRMED";
#[allow(dead_code)]
pub const E_CANCELLED: &str = "E_CANCELLED";
#[allow(dead_code)]
pub const E_PERMISSION: &str = "E_PERMISSION";
pub const E_INVALID: &str = "E_INVALID";
pub const E_IO: &str = "E_IO";

// ── 工具函数 ────────────────────────────────────────────

/// 当前 UTC 时间，格式化为 ISO 8601（`YYYY-MM-DDTHH:MM:SSZ`）。
///
/// 手写实现，避免为省体积引入 chrono（架构 §6.1 允许）。
pub fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format_rfc3339(secs)
}

fn format_rfc3339(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, h, mi, s)
}

/// Howard Hinnant 的 civil_from_days 算法（days 自 1970-01-01）。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// 人类可读字节数（1024 进制），用于导出文本（前端另有一套格式化）。
pub fn format_bytes(b: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut v = b as f64;
    let mut i = 0usize;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} {}", b, UNITS[0])
    } else {
        format!("{:.2} {}", v, UNITS[i])
    }
}
