//! 命令层 / IPC 门面：注册全部 `#[tauri::command]`、管理 `AppState`、收发事件、进度节流、取消位。

mod categories;
mod clean;
mod safety;
mod scan;
mod types;
mod watcher;
mod win;

// QA 独立验证测试（不参与 release 构建）。
#[cfg(test)]
mod qa_tests;

use crate::types::*;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};

// ─────────────────────────── 工具 ───────────────────────────

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("settings.json"))
}

fn load_settings(app: &AppHandle) -> Settings {
    if let Some(f) = settings_path(app) {
        if let Ok(txt) = std::fs::read_to_string(&f) {
            if let Ok(s) = serde_json::from_str::<Settings>(&txt) {
                return s;
            }
        }
    }
    Settings::default()
}

fn store_settings(app: &AppHandle, s: &Settings) -> AppResult<()> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::new(E_IO, e.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    let f = dir.join("settings.json");
    let txt = serde_json::to_string_pretty(s).map_err(|e| AppError::new(E_IO, e.to_string()))?;
    std::fs::write(f, txt)?;
    Ok(())
}

fn resolved_for_ids(ids: &[String], settings: &Settings) -> Vec<categories::ResolvedCategory> {
    let mut v = Vec::new();
    for id in ids {
        if let Some(rc) = categories::resolve_by_id(id, settings) {
            v.push(rc);
        }
    }
    v
}

fn category_result(rc: &categories::ResolvedCategory, one: scan::ScanOne) -> CategoryResult {
    CategoryResult {
        id: rc.id.clone(),
        name: rc.name.clone(),
        risk: rc.risk,
        default_selected: rc.default_selected,
        requires_admin: rc.requires_admin,
        semantics: rc.semantics,
        roots: rc.roots.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        total_size: one.total_size,
        file_count: one.file_count,
        status: one.status,
        error: one.error,
    }
}

// ─────────────────────────── 命令：类别 / 磁盘 ───────────────────────────

/// 返回全部类别定义（内置 + 自定义）。
#[tauri::command]
fn get_categories(state: State<AppState>) -> Vec<Category> {
    let settings = state.settings.lock().unwrap();
    categories::all_categories(&settings)
}

/// 默认 C:，供顶部可用空间条。
#[tauri::command]
fn get_disk_info(drive: Option<String>) -> AppResult<DiskInfo> {
    let d = drive.unwrap_or_else(|| "C:\\".to_string());
    let (total, free) = win::shell::get_disk_free(&d).map_err(|e| AppError::new(E_IO, e))?;
    Ok(DiskInfo {
        drive: d,
        total_bytes: total,
        free_bytes: free,
    })
}

// ─────────────────────────── 命令：扫描 ───────────────────────────

/// 全类并行扫描；过程 emit `scan://category`、`scan://progress`。
#[tauri::command]
fn scan_all(app: AppHandle, state: State<AppState>) -> AppResult<ScanReport> {
    if state.busy.swap(true, Ordering::SeqCst) {
        return Err(AppError::new(E_BUSY, "已有任务进行中，请稍候。"));
    }
    let result = run_scan_all(&app, &state);
    state.busy.store(false, Ordering::SeqCst);
    result
}

fn run_scan_all(app: &AppHandle, state: &State<AppState>) -> AppResult<ScanReport> {
    state.cancel.store(false, Ordering::SeqCst);
    let resolved = {
        let settings = state.settings.lock().unwrap();
        categories::resolve_all(&settings)
    };
    let total_cats = resolved.len();
    let started_at = now_rfc3339();
    let t0 = Instant::now();
    let scan_id = format!("scan-{}", now_millis());

    let files = Arc::new(AtomicU64::new(0));
    let bytes = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicUsize::new(0));
    let cur = Arc::new(Mutex::new(String::new()));
    let stop = Arc::new(AtomicBool::new(false));

    // 进度线程（250ms 节流）
    let p_app = app.clone();
    let p_files = files.clone();
    let p_bytes = bytes.clone();
    let p_done = done.clone();
    let p_cur = cur.clone();
    let p_stop = stop.clone();
    let p_sid = scan_id.clone();
    let progress = std::thread::spawn(move || {
        while !p_stop.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(250));
            if p_stop.load(Ordering::Relaxed) {
                break;
            }
            let cp = p_cur.lock().map(|g| g.clone()).unwrap_or_default();
            let _ = p_app.emit(
                "scan://progress",
                ScanProgress {
                    scan_id: p_sid.clone(),
                    done_categories: p_done.load(Ordering::Relaxed),
                    total_categories: total_cats,
                    scanned_files: p_files.load(Ordering::Relaxed),
                    scanned_bytes: p_bytes.load(Ordering::Relaxed),
                    current_path: cp,
                },
            );
        }
    });

    let cancel = state.cancel.clone();
    let results: Vec<CategoryResult> = resolved
        .par_iter()
        .map(|rc| {
            let one = if rc.roots.is_empty() {
                scan::ScanOne {
                    total_size: 0,
                    file_count: 0,
                    status: ScanStatus::NotFound,
                    error: None,
                }
            } else {
                scan::scan_one(rc, &cancel, &files, &bytes, &cur)
            };
            let res = category_result(rc, one);
            let _ = app.emit("scan://category", &res);
            done.fetch_add(1, Ordering::SeqCst);
            res
        })
        .collect();

    stop.store(true, Ordering::SeqCst);
    let _ = progress.join();

    let total_size = results.iter().map(|r| r.total_size).sum();
    let total_files = results.iter().map(|r| r.file_count).sum();
    let disk = win::shell::get_disk_free("C:\\")
        .map(|(t, f)| DiskInfo {
            drive: "C:\\".to_string(),
            total_bytes: t,
            free_bytes: f,
        })
        .unwrap_or(DiskInfo {
            drive: "C:\\".to_string(),
            total_bytes: 0,
            free_bytes: 0,
        });

    // 注：若整体被取消，各类别 status 会标记为 Skipped，报告仍照常返回部分结果。

    Ok(ScanReport {
        scan_id,
        started_at,
        finished_at: now_rfc3339(),
        duration_ms: t0.elapsed().as_millis() as u64,
        categories: results,
        total_size,
        total_files,
        disk,
    })
}

/// 单类扫描（重扫某行用）。
#[tauri::command]
fn scan_category(id: String, state: State<AppState>) -> AppResult<CategoryResult> {
    let resolved = {
        let settings = state.settings.lock().unwrap();
        categories::resolve_by_id(&id, &settings)
            .ok_or_else(|| AppError::new(E_INVALID, format!("未知类别：{id}")))?
    };
    state.cancel.store(false, Ordering::SeqCst);
    let cancel = state.cancel.clone();
    let files = AtomicU64::new(0);
    let bytes = AtomicU64::new(0);
    let cur = Mutex::new(String::new());
    let one = if resolved.roots.is_empty() {
        scan::ScanOne {
            total_size: 0,
            file_count: 0,
            status: ScanStatus::NotFound,
            error: None,
        }
    } else {
        scan::scan_one(&resolved, &cancel, &files, &bytes, &cur)
    };
    Ok(category_result(&resolved, one))
}

// ─────────────────────────── 命令：预览 / 清理 ───────────────────────────

/// 生成清理清单 + 告警（不落盘、零改动）。
#[tauri::command]
fn preview_clean(req: PreviewRequest, state: State<AppState>) -> AppResult<PreviewPlan> {
    let resolved = {
        let settings = state.settings.lock().unwrap();
        resolved_for_ids(&req.category_ids, &settings)
    };
    Ok(clean::build_preview(&resolved, req.mode))
}

/// 双模式清理 + dry-run；校验 `confirmed`。
#[tauri::command]
fn execute_clean(
    app: AppHandle,
    req: CleanRequest,
    state: State<AppState>,
) -> AppResult<CleanReport> {
    if state.busy.swap(true, Ordering::SeqCst) {
        return Err(AppError::new(E_BUSY, "已有任务进行中，请稍候。"));
    }
    state.cancel.store(false, Ordering::SeqCst);
    let (resolved, cancel) = {
        let settings = state.settings.lock().unwrap();
        (resolved_for_ids(&req.category_ids, &settings), state.cancel.clone())
    };
    let result = clean::execute_clean(&app, &resolved, &req, &cancel);
    state.busy.store(false, Ordering::SeqCst);
    result
}

/// 置 `cancel=true`，中断扫描/清理。
#[tauri::command]
fn cancel_operation(state: State<AppState>) -> AppResult<()> {
    state.cancel.store(true, Ordering::SeqCst);
    Ok(())
}

// ─────────────────────────── 命令：设置持久化 ───────────────────────────

#[tauri::command]
fn get_settings(app: AppHandle, state: State<AppState>) -> AppResult<Settings> {
    let s = load_settings(&app);
    *state.settings.lock().unwrap() = s.clone();
    Ok(s)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings, state: State<AppState>) -> AppResult<Settings> {
    store_settings(&app, &settings)?;
    *state.settings.lock().unwrap() = settings.clone();
    Ok(settings)
}

// ─────────────────────────── 命令：实时监控 ───────────────────────────

#[tauri::command]
fn start_monitor(
    config: MonitorConfig,
    app: AppHandle,
    state: State<AppState>,
) -> AppResult<MonitorStatus> {
    let mut guard = state.monitor.lock().unwrap();
    if let Some(h) = guard.as_ref() {
        if h.status().running {
            return Ok(h.status());
        }
    }
    let handle = watcher::start(&config, app.clone())?;
    let st = handle.status();
    *guard = Some(handle);
    let _ = app.emit("monitor://status", &st);
    Ok(st)
}

#[tauri::command]
fn stop_monitor(app: AppHandle, state: State<AppState>) -> AppResult<MonitorStatus> {
    let mut guard = state.monitor.lock().unwrap();
    let status = if let Some(h) = guard.take() {
        h.stop()
    } else {
        MonitorStatus {
            running: false,
            dirs: vec![],
            policy: MonitorPolicy::NotifyOnly,
            events_count: 0,
        }
    };
    let _ = app.emit("monitor://status", &status);
    Ok(status)
}

#[tauri::command]
fn get_monitor_status(state: State<AppState>) -> MonitorStatus {
    let guard = state.monitor.lock().unwrap();
    match guard.as_ref() {
        Some(h) => h.status(),
        None => MonitorStatus {
            running: false,
            dirs: vec![],
            policy: MonitorPolicy::NotifyOnly,
            events_count: 0,
        },
    }
}

// ─────────────────────────── 命令：外部打开 / 导出 ───────────────────────────

/// 用 explorer.exe 打开文件夹 / 回收站 / 选中文件（不引插件）。
#[tauri::command]
fn open_path(path: String) -> AppResult<()> {
    use std::process::Command;
    let p = path.trim().to_string();
    let spawn = if p.is_empty() || p == "recyclebin" || p.eq_ignore_ascii_case("shell:RecycleBinFolder")
    {
        Command::new("explorer.exe")
            .arg("shell:RecycleBinFolder")
            .spawn()
    } else {
        let pb = PathBuf::from(&p);
        if pb.is_file() {
            Command::new("explorer.exe").arg("/select,").arg(&p).spawn()
        } else {
            Command::new("explorer.exe").arg(&p).spawn()
        }
    };
    spawn.map(|_| ()).map_err(|e| AppError::new(E_IO, e.to_string()))
}

/// 解析导出目标路径：绝对路径原样返回；相对路径落到「桌面」（无桌面则用户主目录）。
fn resolve_export_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        return p;
    }
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    let desktop = PathBuf::from(&home).join("Desktop");
    let base = if desktop.exists() { desktop } else { PathBuf::from(&home) };
    base.join(p)
}

/// 将预览清单导出为 `.txt`（UTF-8）。返回实际写入的绝对路径。
#[tauri::command]
fn export_preview(path: String, plan: PreviewPlan) -> AppResult<String> {
    let mut out = String::new();
    out.push_str("WinSweep 清理清单\n");
    out.push_str("==================\n");
    out.push_str(&format!("生成时间: {}\n", now_rfc3339()));
    out.push_str(&format!("清理模式: {:?}\n", plan.mode));
    out.push_str(&format!(
        "合计: {} 项 · {}\n",
        plan.total_files,
        format_bytes(plan.total_size)
    ));
    for w in &plan.warnings {
        out.push_str(&format!("警告: {w}\n"));
    }
    out.push_str("\n--- 明细（大小\t路径）---\n");
    for e in &plan.entries {
        out.push_str(&format!("{}\t{}\n", format_bytes(e.size), e.path));
    }
    if plan.entries_truncated {
        out.push_str("\n（清单已截断，仅显示前 1000 条）\n");
    }
    let target = resolve_export_path(&path);
    std::fs::write(&target, out)?;
    Ok(target.to_string_lossy().to_string())
}

// ─────────────────────────── 入口 ───────────────────────────

/// 应用入口：注册状态 / 命令 / 初始化。
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            let handle = app.handle().clone();
            let s = load_settings(&handle);
            let state = app.state::<AppState>();
            *state.settings.lock().unwrap() = s;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_categories,
            get_disk_info,
            scan_all,
            scan_category,
            preview_clean,
            execute_clean,
            cancel_operation,
            get_settings,
            save_settings,
            start_monitor,
            stop_monitor,
            get_monitor_status,
            open_path,
            export_preview
        ])
        .run(tauri::generate_context!())
        .expect("运行 WinSweep 时发生错误");
}
