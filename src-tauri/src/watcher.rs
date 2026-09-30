//! 实时监控：notify 监听 + 800ms 防抖 + 策略执行。
//!
//! 策略仅 `NotifyOnly` / `RecycleBin`，**类型层无 Permanent**（禁止实时永久删除）。
//! 默认关闭、默认仅 `%TEMP%`、默认「仅提醒」。

use crate::categories;
use crate::safety;
use crate::types::*;
use notify::{EventKind, RecommendedWatcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const DEBOUNCE: Duration = Duration::from_millis(800);

/// 监控句柄：持有 watcher（生命周期即监听生命周期）与去抖线程。
pub struct MonitorHandle {
    dirs: Vec<String>,
    policy: MonitorPolicy,
    events_count: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    watcher: Option<RecommendedWatcher>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MonitorHandle {
    /// 当前状态快照。
    pub fn status(&self) -> MonitorStatus {
        MonitorStatus {
            running: self.running.load(Ordering::Relaxed),
            dirs: self.dirs.clone(),
            policy: self.policy,
            events_count: self.events_count.load(Ordering::Relaxed),
        }
    }

    /// 停止监控：drop watcher 关闭通道，join 去抖线程。
    pub fn stop(mut self) -> MonitorStatus {
        self.running.store(false, Ordering::Relaxed);
        // drop watcher → sender 关闭 → recv 返回 Disconnected → 线程退出
        self.watcher.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        MonitorStatus {
            running: false,
            dirs: self.dirs.clone(),
            policy: self.policy,
            events_count: self.events_count.load(Ordering::Relaxed),
        }
    }
}

/// 启动监控。
pub fn start(config: &MonitorConfig, app: AppHandle) -> AppResult<MonitorHandle> {
    use notify::{RecursiveMode, Watcher};

    let dirs: Vec<PathBuf> = config
        .dirs
        .iter()
        .map(|d| PathBuf::from(categories::expand_env(d)))
        .filter(|p| p.exists())
        .collect();

    if dirs.is_empty() {
        return Err(AppError::new(
            E_INVALID,
            "没有可监控的目录（均不存在或未配置）。",
        ));
    }

    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default(),
    )
    .map_err(|e| AppError::new(E_IO, format!("创建监控器失败：{e}")))?;

    for d in &dirs {
        watcher
            .watch(d, RecursiveMode::Recursive)
            .map_err(|e| AppError::new(E_IO, format!("监控目录失败 {}：{e}", d.display())))?;
    }

    let events_count = Arc::new(AtomicU64::new(0));
    let running = Arc::new(AtomicBool::new(true));
    let policy = config.policy;

    let thread_app = app.clone();
    let thread_count = events_count.clone();
    let thread_running = running.clone();
    let thread = std::thread::spawn(move || {
        run_loop(rx, thread_app, policy, thread_count, thread_running);
    });

    let dir_strings: Vec<String> = dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    Ok(MonitorHandle {
        dirs: dir_strings,
        policy,
        events_count,
        running,
        watcher: Some(watcher),
        thread: Some(thread),
    })
}

fn run_loop(
    rx: std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
    app: AppHandle,
    policy: MonitorPolicy,
    events_count: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
) {
    let mut last: HashMap<String, Instant> = HashMap::new();

    while running.load(Ordering::Relaxed) {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(event)) => {
                let kind = match event.kind {
                    EventKind::Create(_) => "created",
                    _ => "modified",
                };
                let now = Instant::now();
                for p in event.paths {
                    let key = p.to_string_lossy().to_string();

                    // 800ms 防抖：同路径短时间多次事件合并
                    if let Some(prev) = last.get(&key) {
                        if now.duration_since(*prev) < DEBOUNCE {
                            last.insert(key, now);
                            continue;
                        }
                    }
                    last.insert(key.clone(), now);

                    // 安全守卫：黑名单永不处理
                    if safety::is_protected(&p) {
                        continue;
                    }

                    let size = if p.is_file() {
                        std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0)
                    } else {
                        0
                    };

                    let action = match policy {
                        MonitorPolicy::NotifyOnly => "notify",
                        MonitorPolicy::RecycleBin => {
                            if p.exists() && trash::delete(&p).is_ok() {
                                "trashed"
                            } else {
                                "notify"
                            }
                        }
                    };

                    let ev = MonitorEvent {
                        path: key,
                        size,
                        kind: kind.to_string(),
                        action: action.to_string(),
                        ts: now_rfc3339(),
                    };
                    let _ = app.emit("monitor://event", &ev);
                    events_count.fetch_add(1, Ordering::Relaxed);
                }

                // 防止去重表无限增长
                if last.len() > 4096 {
                    last.clear();
                }
            }
            Ok(Err(_)) => {}
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}
