//! 安全守卫：删除前的**唯一闸门**。
//!
//! 提供：
//! - 受保护路径黑名单 `is_protected`（规范化后前缀匹配，大小写不敏感）；
//! - 允许根校验 `is_within_allowed_roots`（删除路径必须位于该类别已解析根之下）；
//! - 占用/无权限探测 `is_locked`（独占打开失败即判为占用/无权限 → 跳过）；
//! - 风险判定 `risk_of`。
//!
//! 关键：黑名单 ∩ 允许根 **双重校验** —— 即使前端伪造类别，也无法删除黑名单路径。

use std::path::{Path, PathBuf};

/// 受保护路径黑名单（规范化后前缀匹配，大小写不敏感）。
pub const BLACKLIST_PREFIXES: &[&str] = &[
    r"C:\Windows\System32",
    r"C:\Windows\SysWOW64",
    r"C:\Windows\WinSxS",
    r"C:\Windows\Boot",
    r"C:\Windows\Fonts",
    r"C:\Windows\assembly",
    r"C:\Windows\Microsoft.NET",
    r"C:\ProgramData\Microsoft\Windows\Start Menu",
    r"C:\Program Files",
    r"C:\Program Files (x86)",
    r"C:\$Recycle.Bin",
    r"C:\Recovery",
    r"C:\Boot",
];

/// 规范化路径为可比较字符串：取绝对路径（能 canonicalize 则用之），
/// 去除 Windows 长路径前缀 `\\?\`，统一反斜杠，转小写。
pub fn normalize(path: &Path) -> String {
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| {
        // canonicalize 失败（如文件被占用/不存在）：退化为绝对化 + 清理 `.`
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|c| c.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    });
    let mut s = canon.to_string_lossy().to_string();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        s = rest.to_string();
    }
    s.replace('/', "\\").to_lowercase()
}

/// path 是否命中受保护黑名单。
///
/// 注意：`normalize` 会转小写，故前缀亦须转小写后再比较（大小写不敏感）。
pub fn is_protected(path: &Path) -> bool {
    let n = normalize(path);
    BLACKLIST_PREFIXES.iter().any(|pre| {
        let p = pre.to_lowercase();
        n == p || n.starts_with(&format!("{p}\\"))
    })
}

/// path 是否位于任一允许根之下（含自身）。
pub fn is_within_allowed_roots(path: &Path, allowed_roots: &[PathBuf]) -> bool {
    let n = normalize(path);
    allowed_roots.iter().any(|r| {
        let rn = normalize(r);
        n == rn || n.starts_with(&format!("{rn}\\"))
    })
}

/// 尝试独占打开以探测「占用 / 无权限」。
///
/// 采用 `share_mode(0)`（不共享）打开：若被其他进程占用或无权限，则失败 → 判为需跳过。
/// 目录需附加 `FILE_FLAG_BACKUP_SEMANTICS`。
pub fn is_locked(path: &Path) -> bool {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

    let mut opts = std::fs::OpenOptions::new();
    opts.read(true).share_mode(0);
    if path.is_dir() {
        opts.custom_flags(FILE_FLAG_BACKUP_SEMANTICS);
    }
    opts.open(path).is_err()
}

/// 综合闸门：返回 `None` 表示可删；`Some(reason)` 表示应跳过并记录原因。
pub fn guard(path: &Path, allowed_roots: &[PathBuf]) -> Option<String> {
    if is_protected(path) {
        return Some("受保护路径（黑名单），已跳过".to_string());
    }
    if !is_within_allowed_roots(path, allowed_roots) {
        return Some("路径不在该类别允许根范围内，已跳过".to_string());
    }
    if is_locked(path) {
        return Some("文件被占用或无访问权限，已跳过".to_string());
    }
    None
}

/// 风险描述（供报告/警示文案）。
#[allow(dead_code)]
pub fn risk_of(risk: crate::types::RiskLevel) -> &'static str {
    match risk {
        crate::types::RiskLevel::Low => "低风险",
        crate::types::RiskLevel::Medium => "中风险",
        crate::types::RiskLevel::High => "高风险",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blacklist_hits() {
        assert!(is_protected(Path::new(r"C:\Windows\System32")));
        assert!(is_protected(Path::new(r"C:\Windows\System32\drivers\etc\hosts")));
        assert!(is_protected(Path::new(r"C:\Program Files\x\y.dll")));
        assert!(is_protected(Path::new(r"C:\$Recycle.Bin")));
    }

    #[test]
    fn blacklist_misses_targets() {
        // 这些是待清理目标，不应命中黑名单
        assert!(!is_protected(Path::new(r"C:\Windows\Temp")));
        assert!(!is_protected(Path::new(r"C:\Windows\Logs")));
        assert!(!is_protected(Path::new(r"C:\Windows\SoftwareDistribution\Download")));
    }

    #[test]
    fn allowed_roots_containment() {
        let roots = vec![PathBuf::from(r"C:\Temp\winsweep_test")];
        assert!(is_within_allowed_roots(
            Path::new(r"C:\Temp\winsweep_test\sub\a.txt"),
            &roots
        ));
        assert!(!is_within_allowed_roots(
            Path::new(r"C:\Windows\System32\cmd.exe"),
            &roots
        ));
    }

    #[test]
    fn guard_rejects_blacklisted_inside_allowed() {
        // 黑名单 ∩ 允许根 双重校验：即便在允许根内，命中黑名单也必须拒绝
        let roots = vec![PathBuf::from(r"C:\Windows")];
        assert!(guard(Path::new(r"C:\Windows\System32"), &roots).is_some());
    }
}
