//! 清理引擎：预览生成（抽样 1000）、双模式执行、dry-run、回收站空清语义、报告。

use crate::categories::ResolvedCategory;
use crate::safety;
use crate::scan;
use crate::types::*;
use crate::win::shell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// 预览抽样上限（防 IPC 载荷过大）。
pub const MAX_PREVIEW_ENTRIES: usize = 1000;
/// 跳过样本上限。
pub const MAX_SKIP_SAMPLES: usize = 200;

/// 枚举某根的「可删候选」：
/// - 根为文件 → 候选即该文件；
/// - 根为目录 → 候选为其**直接子项**（不删除根目录本身，保护系统目录）。
pub fn candidates_of(root: &Path) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    if !root.is_dir() {
        return vec![];
    }
    let mut v = Vec::new();
    if let Ok(rd) = std::fs::read_dir(root) {
        for e in rd.flatten() {
            v.push(e.path());
        }
    }
    v
}

/// 生成清理预览计划（不落盘、零改动）。
pub fn build_preview(resolved: &[ResolvedCategory], mode: CleanMode) -> PreviewPlan {
    let mut entries: Vec<PreviewEntry> = Vec::new();
    let mut total_files: u64 = 0;
    let mut total_size: u64 = 0;
    let mut has_high_risk = false;
    let mut warnings: Vec<String> = Vec::new();
    let mut truncated = false;

    for rc in resolved {
        if rc.risk == RiskLevel::High {
            has_high_risk = true;
        }

        // 回收站语义：特殊终结操作
        if rc.semantics == CleanSemantics::EmptyRecycleBin {
            let (size, items) = shell::query_recycle_bin();
            total_size += size;
            total_files += items;
            if entries.len() < MAX_PREVIEW_ENTRIES {
                entries.push(PreviewEntry {
                    category_id: rc.id.clone(),
                    category_name: rc.name.clone(),
                    path: "回收站（将清空全部内容，不可恢复）".to_string(),
                    size,
                    is_dir: true,
                });
            } else {
                truncated = true;
            }
            warnings.push(
                "【不可恢复】回收站内容无法再次移入回收站；将执行『清空回收站』，此操作不可恢复。"
                    .to_string(),
            );
            continue;
        }

        if rc.requires_admin {
            warnings.push(format!("『{}』需要管理员权限，无权限时将被跳过。", rc.name));
        }
        if rc.risk == RiskLevel::High {
            warnings.push(format!("『{}』为高风险类别，请谨慎勾选与清理。", rc.name));
        }

        for root in &rc.roots {
            for c in candidates_of(root) {
                if safety::is_protected(&c) || !safety::is_within_allowed_roots(&c, &rc.roots) {
                    continue;
                }
                let size = scan::path_size(&c);
                total_size += size;
                total_files += scan::path_file_count(&c);
                if entries.len() < MAX_PREVIEW_ENTRIES {
                    entries.push(PreviewEntry {
                        category_id: rc.id.clone(),
                        category_name: rc.name.clone(),
                        path: c.to_string_lossy().to_string(),
                        size,
                        is_dir: c.is_dir(),
                    });
                } else {
                    truncated = true;
                }
            }
        }
    }

    if mode == CleanMode::Permanent {
        warnings.push("当前为『永久删除』模式：删除后不可恢复，请确认无误。".to_string());
    }

    PreviewPlan {
        category_ids: resolved.iter().map(|r| r.id.clone()).collect(),
        mode,
        entries,
        entries_truncated: truncated,
        total_files,
        total_size,
        has_high_risk,
        warnings,
    }
}

/// 执行清理（双模式 + dry-run + 回收站语义）。
pub fn execute_clean(
    app: &AppHandle,
    resolved: &[ResolvedCategory],
    req: &CleanRequest,
    cancel: &AtomicBool,
) -> AppResult<CleanReport> {
    if !req.confirmed {
        return Err(AppError::new(
            E_NOT_CONFIRMED,
            "未获得二次确认，已拒绝执行清理。",
        ));
    }

    let dry = req.dry_run;
    let mut report = CleanReport {
        cleaned_at: now_rfc3339(),
        mode: req.mode,
        dry_run: dry,
        freed_bytes: 0,
        deleted_count: 0,
        skipped_count: 0,
        failed_count: 0,
        per_category: Vec::new(),
        skipped_samples: Vec::new(),
        notes: Vec::new(),
    };

    // 预枚举全部候选，用于进度总量。
    let mut plan: Vec<(&ResolvedCategory, Vec<PathBuf>)> = Vec::new();
    let mut total: usize = 0;
    for rc in resolved {
        if rc.semantics == CleanSemantics::EmptyRecycleBin {
            plan.push((rc, Vec::new()));
            total += 1;
        } else {
            let mut cands: Vec<PathBuf> = Vec::new();
            for root in &rc.roots {
                cands.extend(candidates_of(root));
            }
            total += cands.len();
            plan.push((rc, cands));
        }
    }

    let mut processed: usize = 0;
    let mut last_emit = Instant::now();
    let mut freed_running: u64 = 0;
    let mut cancelled = false;

    for (rc, cands) in plan {
        let mut stat = CategoryCleanStat {
            id: rc.id.clone(),
            name: rc.name.clone(),
            freed_bytes: 0,
            deleted_count: 0,
            skipped_count: 0,
            failed_count: 0,
        };

        // ── 回收站类别：两种模式都执行 SHEmptyRecycleBinW ──
        if rc.semantics == CleanSemantics::EmptyRecycleBin {
            let (size, items) = shell::query_recycle_bin();
            if dry {
                stat.freed_bytes = size;
                stat.deleted_count = items;
                report.notes.push(format!(
                    "（dry-run）回收站：将执行『清空回收站』，预计释放 {}。",
                    format_bytes(size)
                ));
            } else {
                match shell::empty_recycle_bin() {
                    Ok(()) => {
                        stat.freed_bytes = size;
                        stat.deleted_count = items;
                        report
                            .notes
                            .push("回收站：已执行『清空回收站』（不可恢复）。".to_string());
                    }
                    Err(e) => {
                        stat.failed_count += 1;
                        push_skip(
                            &mut report.skipped_samples,
                            "回收站".to_string(),
                            e,
                        );
                    }
                }
            }
            report
                .notes
                .push("回收站为特殊终结操作：无论清理模式如何，均为『清空』且不可恢复。".to_string());
            if rc.requires_admin {
                report
                    .notes
                    .push("提示：部分系统级目录若无权限访问，请以管理员身份重启后再试。".to_string());
            }

            freed_running += stat.freed_bytes;
            report.per_category.push(stat);
            processed += 1;
            emit_progress(
                app,
                &mut last_emit,
                processed,
                total,
                freed_running,
                "回收站",
            );
            continue;
        }

        // ── 普通类别：逐个候选处理 ──
        for c in cands {
            processed += 1;
            if cancel.load(Ordering::Relaxed) {
                cancelled = true;
                break;
            }

            // 安全闸门（黑名单 ∩ 允许根 + 占用探测）
            if let Some(reason) = safety::guard(&c, &rc.roots) {
                stat.skipped_count += 1;
                push_skip(
                    &mut report.skipped_samples,
                    c.to_string_lossy().to_string(),
                    reason,
                );
                maybe_emit(
                    app,
                    &mut last_emit,
                    processed,
                    total,
                    freed_running,
                    &c,
                );
                continue;
            }

            let size = scan::path_size(&c);

            if dry {
                stat.freed_bytes += size;
                stat.deleted_count += 1;
                freed_running += size;
            } else {
                let result = match req.mode {
                    CleanMode::RecycleBin => {
                        trash::delete(&c).map_err(|e| format!("移入回收站失败：{e}"))
                    }
                    CleanMode::Permanent => remove_path(&c),
                };
                match result {
                    Ok(()) => {
                        stat.freed_bytes += size;
                        stat.deleted_count += 1;
                        freed_running += size;
                    }
                    Err(msg) => {
                        stat.failed_count += 1;
                        push_skip(
                            &mut report.skipped_samples,
                            c.to_string_lossy().to_string(),
                            msg,
                        );
                    }
                }
            }

            maybe_emit(app, &mut last_emit, processed, total, freed_running, &c);
        }

        if rc.requires_admin && (stat.skipped_count > 0 || stat.failed_count > 0) {
            report.notes.push(format!(
                "『{}』需管理员权限：部分文件被跳过，建议以管理员身份重启后再试。",
                rc.name
            ));
        }

        report.per_category.push(stat);
        if cancelled {
            break;
        }
    }

    if dry {
        report
            .notes
            .push("（dry-run）模拟清理：未对任何文件做实际改动。".to_string());
    }
    if cancelled {
        report.notes.push("操作已被用户取消。".to_string());
    }

    // 汇总
    report.freed_bytes = report.per_category.iter().map(|s| s.freed_bytes).sum();
    report.deleted_count = report.per_category.iter().map(|s| s.deleted_count).sum();
    report.skipped_count = report.per_category.iter().map(|s| s.skipped_count).sum();
    report.failed_count = report.per_category.iter().map(|s| s.failed_count).sum();

    Ok(report)
}

fn remove_path(p: &Path) -> Result<(), String> {
    let r = if p.is_dir() {
        std::fs::remove_dir_all(p)
    } else {
        std::fs::remove_file(p)
    };
    r.map_err(|e| format!("删除失败：{e}"))
}

fn push_skip(samples: &mut Vec<SkipRecord>, path: String, reason: String) {
    if samples.len() < MAX_SKIP_SAMPLES {
        samples.push(SkipRecord { path, reason });
    }
}

fn emit_progress(
    app: &AppHandle,
    last: &mut Instant,
    processed: usize,
    total: usize,
    freed: u64,
    current: &str,
) {
    let _ = app.emit(
        "clean://progress",
        CleanProgress {
            processed,
            total,
            freed_bytes: freed,
            current_path: current.to_string(),
        },
    );
    *last = Instant::now();
}

fn maybe_emit(
    app: &AppHandle,
    last: &mut Instant,
    processed: usize,
    total: usize,
    freed: u64,
    path: &Path,
) {
    if last.elapsed() >= Duration::from_millis(250) {
        emit_progress(
            app,
            last,
            processed,
            total,
            freed,
            &path.to_string_lossy(),
        );
    }
}
