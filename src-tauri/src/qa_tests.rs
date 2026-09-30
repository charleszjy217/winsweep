//! QA 独立验证测试（QA 工程师严过关 新增，独立于工程师原始单测）。
//!
//! ⚠️ 安全红线：本模块**只**在临时沙箱目录（`%TEMP%\winsweep_qa_*`）内运行，
//! 所有清理路径均为 **dry-run** 或**沙箱内删除**；**绝不**触碰任何真实系统目录，
//! **绝不**调用会清空真实回收站的逻辑（仅做 dry-run 分支证明 + 只读查询）。
//!
//! 覆盖：黑名单大小写对抗、双重校验、回收站语义、dry-run 零改动、
//!       未确认拒绝、类别完整性、缺失工具根解析。

#![cfg(test)]

use crate::categories::ResolvedCategory;
use crate::types::{CleanMode, CleanSemantics, RiskLevel, Settings};
use crate::{categories, clean, safety};
use std::path::{Path, PathBuf};

// ───────────────────────── 沙箱工具 ─────────────────────────

/// 临时沙箱目录，Drop 时自动清理。
struct Sandbox {
    dir: PathBuf,
}

impl Sandbox {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "winsweep_qa_{}_{}_{}",
            tag,
            std::process::id(),
            nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建沙箱目录");
        Sandbox { dir }
    }

    fn file(&self, name: &str, content: &[u8]) -> PathBuf {
        let p = self.dir.join(name);
        std::fs::write(&p, content).expect("写沙箱文件");
        p
    }

    fn count_files(&self) -> usize {
        count_files(&self.dir)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn count_files(dir: &Path) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .count()
}

/// 目录快照：相对路径 → 内容，已排序，用于断言「零改动」。
fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut v: Vec<(String, Vec<u8>)> = walkdir::WalkDir::new(dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            let rel = e
                .path()
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .to_string();
            (rel, std::fs::read(e.path()).unwrap_or_default())
        })
        .collect();
    v.sort();
    v
}

// ───────────────────────── 1. 黑名单大小写对抗 ─────────────────────────

/// 回归：工程师修复的「黑名单前缀大小写不匹配」bug。
/// 任意混合大小写的受保护路径都必须命中。
#[test]
fn qa_blacklist_case_insensitive_hits() {
    let cases = [
        r"c:\windows\system32\kernel32.dll",
        r"C:\WINDOWS\SYSTEM32\kernel32.dll",
        r"C:\Windows\System32\Kernel32.dll",
        r"C:\WiNdOwS\SySWoW64\foo.dll",
        r"C:\WINDOWS\winsxs\bar",
        r"C:\Program Files\App\x.exe",
        r"c:\program files\App\x.exe",
        r"C:\PROGRAM FILES (X86)\App\x.exe",
        r"c:\program files (x86)\app\x.exe",
        r"C:\Windows\Boot\EFI\x",
        r"c:\windows\fonts\a.ttf",
        r"C:\Windows\assembly\x",
        r"C:\Windows\Microsoft.NET\x",
        r"C:\ProgramData\Microsoft\Windows\Start Menu\x",
        r"C:\$Recycle.Bin",
        r"c:\$recycle.bin\S-1-5-18",
        r"C:\Recovery\x",
        r"C:\Boot\x",
    ];
    for c in cases {
        assert!(
            safety::is_protected(Path::new(c)),
            "混合大小写应命中黑名单: {c}"
        );
    }
}

/// 正常缓存/待清理路径必须**不**命中黑名单（防误伤）。
#[test]
fn qa_blacklist_negative_normal_targets() {
    let temp = std::env::var("TEMP").expect("%TEMP% 应存在");
    let cases = [
        temp.as_str(),
        r"C:\Windows\Temp",
        r"C:\Windows\SoftwareDistribution\Download",
        r"C:\Windows\Logs",
        r"C:\Windows\Panther",
        r"C:\Windows\Minidump",
        r"C:\Users\someone\.gradle\caches",
        r"C:\Users\someone\.cargo\registry\cache",
        r"C:\Users\someone\AppData\Local\pip\cache",
        r"C:\Program Files Backup\x",
        r"C:\Windows\System32Backup\x",
        r"C:\Windows\System32Old\x",
        r"C:\Bootstrapper\x",
    ];
    for c in cases {
        assert!(!safety::is_protected(Path::new(c)), "不应命中黑名单: {c}");
    }
}

// ───────────────────────── 2. 双重校验 ─────────────────────────

/// 黑名单 ∩ 允许根 双重校验：命中黑名单 或 不在允许根下 都必须被拒。
#[test]
fn qa_guard_double_validation() {
    let sb = Sandbox::new("guard");
    let good = sb.file("ok.tmp", b"x");

    let win_roots = vec![PathBuf::from(r"C:\Windows")];
    assert!(
        safety::guard(Path::new(r"C:\Windows\System32\kernel32.dll"), &win_roots).is_some(),
        "黑名单路径即便在允许根内也必须被拒"
    );

    let only_sandbox = [sb.dir.clone()];
    assert!(
        safety::guard(Path::new(r"C:\Windows\System32\kernel32.dll"), &only_sandbox).is_some(),
        "越界的路径必须被拒"
    );
    assert!(
        safety::guard(&good, &only_sandbox).is_none(),
        "合法沙箱文件应通过安全闸门"
    );
}

// ───────────────────────── 3. 回收站语义（纯逻辑） ─────────────────────────

/// 回收站类别：语义恒为 EmptyRecycleBin；两种清理模式下均产生「不可恢复」告警，
/// 且**不会**被当作普通目录逐文件处理（绝不直接 fs 删除 C:\$Recycle.Bin）。
#[test]
fn qa_recycle_bin_semantics_both_modes() {
    let settings = Settings::default();
    let rc = categories::resolve_by_id("recycle_bin", &settings).expect("内置 recycle_bin 应存在");
    assert_eq!(rc.semantics, CleanSemantics::EmptyRecycleBin);

    for mode in [CleanMode::RecycleBin, CleanMode::Permanent] {
        let plan = clean::build_preview(std::slice::from_ref(&rc), mode);
        assert!(
            plan.warnings.iter().any(|w| w.contains("不可恢复")),
            "mode={mode:?} 应包含不可恢复告警"
        );
        assert!(
            !plan
                .entries
                .iter()
                .any(|e| e.path.to_lowercase().contains("$recycle.bin")),
            "回收站内容不得逐文件枚举为可删项"
        );
    }
}

// ───────────────────────── 6. 类别完整性 ─────────────────────────

/// 内置类别数 = 18、id 唯一、回收站语义正确、高风险默认不勾选。
#[test]
fn qa_builtin_categories_integrity() {
    assert_eq!(categories::CATEGORY_SPECS.len(), 18, "内置类别数应为 18");

    let ids: Vec<&str> = categories::CATEGORY_SPECS.iter().map(|s| s.id).collect();
    let mut uniq = ids.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), 18, "类别 id 不得重复");

    let rb = categories::CATEGORY_SPECS
        .iter()
        .find(|s| s.id == "recycle_bin")
        .expect("应含 recycle_bin");
    assert_eq!(rb.semantics, CleanSemantics::EmptyRecycleBin);

    for s in categories::CATEGORY_SPECS {
        if s.risk == RiskLevel::High {
            assert!(!s.default_selected, "高风险类别 {} 必须默认不勾选", s.id);
        }
    }

    for id in ["browser_chrome", "browser_edge", "browser_firefox"] {
        let spec = categories::CATEGORY_SPECS
            .iter()
            .find(|s| s.id == id)
            .unwrap_or_else(|| panic!("应含 {id}"));
        for root in spec.roots {
            let lower = root.to_lowercase();
            for forbidden in ["cookie", "login data", "bookmark", "history", "web data"] {
                assert!(
                    !lower.contains(forbidden),
                    "浏览器类别 {id} 的根 {root} 不得包含敏感项 {forbidden}"
                );
            }
        }
    }
}

/// 未安装工具（yarn/go/m2/firefox）`resolve_roots` 返回空且不 panic；
/// 未知 id 亦返回空；解析出的根必须真实存在。
#[test]
fn qa_resolve_roots_missing_tools_empty_no_panic() {
    for id in [
        "yarn_cache",
        "go_build_cache",
        "m2_repo",
        "browser_firefox",
        "cargo_registry",
    ] {
        let roots = categories::resolve_roots(id);
        for r in &roots {
            assert!(r.exists(), "{id} 解析出的根必须真实存在: {r:?}");
        }
    }
    assert!(
        categories::resolve_roots("__no_such_category__").is_empty(),
        "未知类别应返回空"
    );
}

// ───────────────────────── 4. dry-run 计划生成零改动 ─────────────────────────

/// `build_preview`（dry-run 计划生成，契约「不落盘、零改动」）：
/// 沙箱内造文件 → 生成计划 → 文件数量/内容零变化，且统计正确。
#[test]
fn qa_build_preview_is_zero_change_on_sandbox() {
    let sb = Sandbox::new("preview");
    sb.file("a.bin", &vec![7u8; 100]);
    sb.file("b.bin", &vec![9u8; 50]);
    let sub = sb.dir.join("subdir");
    std::fs::create_dir_all(&sub).expect("建子目录");
    std::fs::write(sub.join("deep.bin"), vec![1u8; 25]).expect("写深层文件");

    let rc = ResolvedCategory {
        id: "qa_sandbox".to_string(),
        name: "QA 沙箱".to_string(),
        risk: RiskLevel::Low,
        default_selected: false,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "QA 沙箱类别".to_string(),
        roots: vec![sb.dir.clone()],
    };

    let before = snapshot(&sb.dir);
    let before_count = sb.count_files();

    // Permanent 模式下的计划生成同样不得改动任何文件
    let plan = clean::build_preview(std::slice::from_ref(&rc), CleanMode::Permanent);

    // 直接子项：a.bin + b.bin + subdir = 3；字节 100+50+25=175；文件数 3
    assert_eq!(plan.entries.len(), 3, "计划条目应为 3");
    assert_eq!(plan.total_size, 175, "计划总字节应为 175");
    assert_eq!(plan.total_files, 3, "计划文件数应为 3");

    assert_eq!(snapshot(&sb.dir), before, "计划生成后文件内容必须零变化");
    assert_eq!(sb.count_files(), before_count, "计划生成后文件数量必须零变化");
}

// ───────────────────────── 备注：AppHandle 耦合的可测性说明 ─────────────────────────
//
// `clean::execute_clean` 形参为具体运行时 `AppHandle`（默认 Wry），且命令体本身非
// 泛型，故无法用 `tauri::test` 的 MockRuntime 调用；在本机测试线程亦无法构建真实
// Wry 应用（tao 事件循环要求主线程）。因此对该函数的「未确认拒绝 / dry-run 分支」
// 以**静态核验**（见交付报告的源码行号引用）为准，其依赖的确定性逻辑
// `safety::guard` / `build_preview` / `candidates_of` 已在上方以可执行测试覆盖。
