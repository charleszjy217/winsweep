//! 扫描引擎：walkdir 遍历 + 体量/文件数统计 + 可取消。
//!
//! 并行策略（架构 §4.7）：**类别间** rayon 并行，**类别内** walkdir 单线程顺序遍历。

use crate::categories::ResolvedCategory;
use crate::safety;
use crate::types::ScanStatus;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use walkdir::WalkDir;

/// 单类别扫描结果。
pub struct ScanOne {
    pub total_size: u64,
    pub file_count: u64,
    pub status: ScanStatus,
    pub error: Option<String>,
}

/// 扫描单个类别。共享原子量用于跨类别进度统计。
///
/// - `files` / `bytes`：全局进度累加器；
/// - `cur`：当前正在扫描的根路径（供进度事件展示，成本极低）。
pub fn scan_one(
    rc: &ResolvedCategory,
    cancel: &AtomicBool,
    files: &AtomicU64,
    bytes: &AtomicU64,
    cur: &Mutex<String>,
) -> ScanOne {
    if rc.roots.is_empty() {
        return ScanOne {
            total_size: 0,
            file_count: 0,
            status: ScanStatus::NotFound,
            error: None,
        };
    }

    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;
    let mut error: Option<String> = None;

    for root in &rc.roots {
        if cancel.load(Ordering::Relaxed) {
            return ScanOne {
                total_size,
                file_count,
                status: ScanStatus::Skipped,
                error,
            };
        }
        if let Ok(mut g) = cur.lock() {
            *g = root.to_string_lossy().to_string();
        }

        // 文件型根（如缩略图缓存 _*.db）
        if root.is_file() {
            match root.metadata() {
                Ok(m) => {
                    let len = m.len();
                    total_size += len;
                    file_count += 1;
                    files.fetch_add(1, Ordering::Relaxed);
                    bytes.fetch_add(len, Ordering::Relaxed);
                }
                Err(e) => error = Some(e.to_string()),
            }
            continue;
        }

        for entry in WalkDir::new(root).follow_links(false).into_iter() {
            if cancel.load(Ordering::Relaxed) {
                return ScanOne {
                    total_size,
                    file_count,
                    status: ScanStatus::Skipped,
                    error,
                };
            }
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    if error.is_none() {
                        error = Some(e.to_string());
                    }
                    continue;
                }
            };
            let p = entry.path();
            if safety::is_protected(p) {
                continue;
            }
            if entry.file_type().is_file() {
                if let Ok(m) = entry.metadata() {
                    let len = m.len();
                    total_size += len;
                    file_count += 1;
                    files.fetch_add(1, Ordering::Relaxed);
                    bytes.fetch_add(len, Ordering::Relaxed);
                }
            }
        }
    }

    ScanOne {
        total_size,
        file_count,
        status: ScanStatus::Ok,
        error,
    }
}

/// 计算路径（文件或目录）的总体量（递归）。
pub fn path_size(p: &Path) -> u64 {
    if p.is_file() {
        return p.metadata().map(|m| m.len()).unwrap_or(0);
    }
    let mut s: u64 = 0;
    for e in WalkDir::new(p).follow_links(false).into_iter().flatten() {
        if e.file_type().is_file() {
            s += e.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    s
}

/// 计算路径下的文件数（文件自身计 1）。
pub fn path_file_count(p: &Path) -> u64 {
    if p.is_file() {
        return 1;
    }
    let mut c: u64 = 0;
    for e in WalkDir::new(p).follow_links(false).into_iter().flatten() {
        if e.file_type().is_file() {
            c += 1;
        }
    }
    c
}
