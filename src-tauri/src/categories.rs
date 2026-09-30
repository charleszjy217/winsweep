//! 类别层：垃圾类别**定义**（静态常量表）+ 根路径解析（纯函数，易测）。
//!
//! 约束（架构 §8.1）：
//! - 类别 ID `snake_case`、稳定、不复用。
//! - 浏览器类别**只**枚举 `Cache / Code Cache / GPUCache / cache2`，硬编码排除
//!   `Cookies / Login Data / Bookmarks / History / Web Data`（绝不放宽）。
//! - 路径支持 `%ENV%` 展开与单层 `*` 通配（用于浏览器 profile 目录）。

use crate::types::{Category, CleanSemantics, CustomCategory, RiskLevel, Settings};
use std::path::PathBuf;

/// 静态类别规格。
pub struct CategorySpec {
    pub id: &'static str,
    pub name: &'static str,
    pub risk: RiskLevel,
    pub default_selected: bool,
    pub requires_admin: bool,
    pub semantics: CleanSemantics,
    pub description: &'static str,
    /// 根路径模板：支持 `%ENV%` 与单层 `*` 通配；可为文件路径（如缩略图缓存）。
    pub roots: &'static [&'static str],
}

/// 全部 18 个内置类别（顺序即 UI 展示顺序）。
pub static CATEGORY_SPECS: &[CategorySpec] = &[
    CategorySpec {
        id: "temp_user",
        name: "用户临时目录 %TEMP%",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "当前用户临时文件目录，可安全清理。",
        roots: &["%TEMP%"],
    },
    CategorySpec {
        id: "temp_windows",
        name: "系统临时目录",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: true,
        semantics: CleanSemantics::Normal,
        description: "C:\\Windows\\Temp 系统临时目录，清理需管理员权限。",
        roots: &["%WINDIR%\\Temp"],
    },
    CategorySpec {
        id: "recycle_bin",
        name: "回收站",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::EmptyRecycleBin,
        description: "回收站内容。清空后不可恢复（无法再次移入回收站）。",
        roots: &[],
    },
    CategorySpec {
        id: "windows_update",
        name: "Windows 更新缓存",
        risk: RiskLevel::Medium,
        default_selected: false,
        requires_admin: true,
        semantics: CleanSemantics::Normal,
        description: "Windows 更新下载缓存，清理需管理员权限。",
        roots: &["%WINDIR%\\SoftwareDistribution\\Download"],
    },
    CategorySpec {
        id: "delivery_optimization",
        name: "传递优化文件",
        risk: RiskLevel::Medium,
        default_selected: false,
        requires_admin: true,
        semantics: CleanSemantics::Normal,
        description: "Windows 传递优化（P2P 更新分发）缓存，清理需管理员权限。",
        roots: &[
            "%WINDIR%\\ServiceProfiles\\NetworkService\\AppData\\Local\\Microsoft\\Windows\\DeliveryOptimization",
        ],
    },
    CategorySpec {
        id: "pip_cache",
        name: "pip 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "Python pip 下载缓存，可安全清理（下次安装自动重下）。",
        roots: &["%LOCALAPPDATA%\\pip\\cache"],
    },
    CategorySpec {
        id: "npm_cache",
        name: "npm 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "npm 包缓存，可安全清理。",
        roots: &["%LOCALAPPDATA%\\npm-cache", "%APPDATA%\\npm-cache"],
    },
    CategorySpec {
        id: "yarn_cache",
        name: "yarn 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "Yarn 包缓存，可安全清理。",
        roots: &["%LOCALAPPDATA%\\Yarn\\Cache"],
    },
    CategorySpec {
        id: "gradle_cache",
        name: ".gradle 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "Gradle 依赖与构建缓存（caches 目录），清理后重新下载。",
        roots: &["%USERPROFILE%\\.gradle\\caches"],
    },
    CategorySpec {
        id: "m2_repo",
        name: "Maven .m2 仓库",
        risk: RiskLevel::Low,
        default_selected: false,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "Maven 本地仓库。重建成本较高，默认不勾选。",
        roots: &["%USERPROFILE%\\.m2\\repository"],
    },
    CategorySpec {
        id: "cargo_registry",
        name: "Cargo registry 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "Rust Cargo registry 的 cache 与 src 下载缓存。",
        roots: &[
            "%USERPROFILE%\\.cargo\\registry\\cache",
            "%USERPROFILE%\\.cargo\\registry\\src",
        ],
    },
    CategorySpec {
        id: "go_build_cache",
        name: "Go build cache",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "Go 构建缓存，可安全清理。",
        roots: &["%LOCALAPPDATA%\\go-build"],
    },
    CategorySpec {
        id: "browser_chrome",
        name: "Chrome 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "仅清理 Chrome 缓存（Cache / Code Cache / GPUCache），绝不触碰 Cookie/登录态/书签/历史。",
        roots: &[
            "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\*\\Cache",
            "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\*\\Code Cache",
            "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\*\\GPUCache",
        ],
    },
    CategorySpec {
        id: "browser_edge",
        name: "Edge 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "仅清理 Edge 缓存（Cache / Code Cache / GPUCache），绝不触碰 Cookie/登录态/书签/历史。",
        roots: &[
            "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\*\\Cache",
            "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\*\\Code Cache",
            "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\*\\GPUCache",
        ],
    },
    CategorySpec {
        id: "browser_firefox",
        name: "Firefox 缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "仅清理 Firefox 的 cache2 缓存目录，绝不触碰 Cookie/登录态/书签/历史。",
        roots: &["%LOCALAPPDATA%\\Mozilla\\Firefox\\Profiles\\*\\cache2"],
    },
    CategorySpec {
        id: "error_reports",
        name: "错误报告与转储",
        risk: RiskLevel::Medium,
        default_selected: false,
        requires_admin: true,
        semantics: CleanSemantics::Normal,
        description: "Windows 错误报告（WER）、崩溃转储与小型内存转储文件。",
        roots: &[
            "%PROGRAMDATA%\\Microsoft\\Windows\\WER",
            "%LOCALAPPDATA%\\CrashDumps",
            "%WINDIR%\\Minidump",
        ],
    },
    CategorySpec {
        id: "thumbnail_cache",
        name: "缩略图缓存",
        risk: RiskLevel::Low,
        default_selected: true,
        requires_admin: false,
        semantics: CleanSemantics::Normal,
        description: "资源管理器缩略图/图标缓存数据库（thumbcache_*.db / iconcache_*.db）。",
        roots: &[
            "%LOCALAPPDATA%\\Microsoft\\Windows\\Explorer\\thumbcache_*.db",
            "%LOCALAPPDATA%\\Microsoft\\Windows\\Explorer\\iconcache_*.db",
        ],
    },
    CategorySpec {
        id: "system_logs",
        name: "系统日志",
        risk: RiskLevel::High,
        default_selected: false,
        requires_admin: true,
        semantics: CleanSemantics::Normal,
        description: "【高风险】系统日志与安装日志。清理可能影响问题排查，默认不勾选。",
        roots: &["%WINDIR%\\Logs", "%WINDIR%\\Panther"],
    },
];

/// 已解析（root 展开）的类别，供 scan / clean 使用。
#[derive(Debug, Clone)]
pub struct ResolvedCategory {
    pub id: String,
    pub name: String,
    pub risk: RiskLevel,
    pub default_selected: bool,
    pub requires_admin: bool,
    pub semantics: CleanSemantics,
    pub description: String,
    pub roots: Vec<PathBuf>,
}

impl ResolvedCategory {
    fn from_spec(spec: &CategorySpec) -> Self {
        Self {
            id: spec.id.to_string(),
            name: spec.name.to_string(),
            risk: spec.risk,
            default_selected: spec.default_selected,
            requires_admin: spec.requires_admin,
            semantics: spec.semantics,
            description: spec.description.to_string(),
            roots: resolve_roots(spec.id),
        }
    }

    fn from_custom(cc: &CustomCategory) -> Self {
        let path = expand_env(cc.path.as_str());
        let pb = PathBuf::from(&path);
        let roots = if pb.exists() { vec![pb] } else { vec![] };
        Self {
            id: cc.id.clone(),
            name: cc.name.clone(),
            risk: cc.risk,
            default_selected: false,
            requires_admin: false,
            semantics: CleanSemantics::Normal,
            description: format!("自定义类别：{}", cc.path),
            roots,
        }
    }

    /// 转换为供前端的 `Category`。
    pub fn to_category(&self) -> Category {
        Category {
            id: self.id.clone(),
            name: self.name.clone(),
            risk: self.risk,
            default_selected: self.default_selected,
            requires_admin: self.requires_admin,
            semantics: self.semantics,
            description: self.description.clone(),
        }
    }
}

/// 全部内置类别定义（未解析 root），供 `get_categories`。
pub fn builtin_categories() -> Vec<Category> {
    CATEGORY_SPECS
        .iter()
        .map(|s| Category {
            id: s.id.to_string(),
            name: s.name.to_string(),
            risk: s.risk,
            default_selected: s.default_selected,
            requires_admin: s.requires_admin,
            semantics: s.semantics,
            description: s.description.to_string(),
        })
        .collect()
}

/// 全部类别（内置 + 自定义）定义，供 `get_categories`。
pub fn all_categories(settings: &Settings) -> Vec<Category> {
    let mut v = builtin_categories();
    for cc in &settings.custom_categories {
        v.push(
            ResolvedCategory::from_custom(cc)
                .to_category(),
        );
    }
    v
}

/// 解析全部类别（展开 root，仅保留存在的路径）。
pub fn resolve_all(settings: &Settings) -> Vec<ResolvedCategory> {
    let mut v: Vec<ResolvedCategory> = CATEGORY_SPECS
        .iter()
        .map(ResolvedCategory::from_spec)
        .collect();
    for cc in &settings.custom_categories {
        v.push(ResolvedCategory::from_custom(cc));
    }
    v
}

/// 按 id 解析单个类别。
pub fn resolve_by_id(id: &str, settings: &Settings) -> Option<ResolvedCategory> {
    if let Some(spec) = CATEGORY_SPECS.iter().find(|s| s.id == id) {
        return Some(ResolvedCategory::from_spec(spec));
    }
    settings
        .custom_categories
        .iter()
        .find(|c| c.id == id)
        .map(ResolvedCategory::from_custom)
}

/// 解析某内置类别的根路径（展开 `%ENV%` 与 `*`，仅保留存在者）。
pub fn resolve_roots(id: &str) -> Vec<PathBuf> {
    let Some(spec) = CATEGORY_SPECS.iter().find(|s| s.id == id) else {
        return vec![];
    };
    let mut out: Vec<PathBuf> = Vec::new();
    for tmpl in spec.roots {
        let expanded = expand_env(tmpl);
        for p in expand_glob(&expanded) {
            if p.exists() {
                out.push(p);
            }
        }
    }
    // 去重
    out.sort();
    out.dedup();
    out
}

/// 展开 `%VAR%` 形式的环境变量。
pub fn expand_env(input: &str) -> String {
    let mut s = input.to_string();
    // 常见 Windows 环境变量名（大小写在 Windows 上不敏感，此处用大写形式匹配 %VAR%）
    for var in [
        "TEMP",
        "TMP",
        "LOCALAPPDATA",
        "APPDATA",
        "USERPROFILE",
        "PROGRAMDATA",
        "WINDIR",
    ] {
        if let Ok(val) = std::env::var(var) {
            s = s.replace(&format!("%{var}%"), &val);
        }
    }
    s
}

/// 展开路径中的单层 `*` 通配（逐段枚举）。
///
/// 例：`C:\...\User Data\*\Cache` → 枚举 `User Data` 下所有「*」子目录的 `Cache`。
pub fn expand_glob(pattern: &str) -> Vec<PathBuf> {
    let comps: Vec<&str> = pattern.split('\\').collect();

    // 1) 找到第一个含 `*` 的段；其之前的段构成 base。
    let mut base = PathBuf::new();
    let mut base_ready = false;
    let mut i = 0usize;
    while i < comps.len() {
        let c = comps[i];
        if c.is_empty() {
            i += 1;
            continue;
        }
        if c.contains('*') {
            break;
        }
        if !base_ready && c.len() == 2 && c.ends_with(':') {
            // 盘符，如 "C:"
            base = PathBuf::from(format!("{c}\\"));
            base_ready = true;
        } else {
            base.push(c);
            base_ready = true;
        }
        i += 1;
    }

    // 2) 余下段（含通配）逐层展开。
    let tail: Vec<&str> = comps[i..].iter().copied().filter(|s| !s.is_empty()).collect();
    let mut current: Vec<PathBuf> = vec![base];
    for seg in tail {
        let mut next: Vec<PathBuf> = Vec::new();
        for b in &current {
            if seg.contains('*') {
                let idx = seg.find('*').unwrap();
                let prefix = &seg[..idx];
                let suffix = &seg[idx + 1..];
                if let Ok(rd) = std::fs::read_dir(b) {
                    for e in rd.flatten() {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with(prefix) && name.ends_with(suffix) {
                            next.push(b.join(&name));
                        }
                    }
                }
            } else {
                let p = b.join(seg);
                if p.exists() {
                    next.push(p);
                }
            }
        }
        current = next;
    }

    current.into_iter().filter(|p| p.exists()).collect()
}

/// 判断某 id 是否为内置类别。
#[allow(dead_code)]
pub fn is_builtin(id: &str) -> bool {
    CATEGORY_SPECS.iter().any(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_expansion_works() {
        std::env::set_var("WINSWEEP_TESTVAR", "X");
        assert_eq!(expand_env("%TEMP%"), std::env::var("TEMP").unwrap());
    }

    #[test]
    fn resolve_roots_missing_tool_is_empty_not_panic() {
        // 不存在的用户工具目录应返回空（而非 panic）
        let roots = resolve_roots("yarn_cache");
        // 允许为空；只验证不 panic
        let _ = roots;
    }

    #[test]
    fn builtin_count_is_18() {
        assert_eq!(CATEGORY_SPECS.len(), 18);
    }

    #[test]
    fn glob_single_star() {
        let tmp = std::env::temp_dir();
        let dir = tmp.join("winsweep_glob_test");
        let sub = dir.join("profile_a").join("Cache");
        std::fs::create_dir_all(&sub).unwrap();
        let pattern = format!("{}\\*\\Cache", dir.to_string_lossy());
        let got = expand_glob(&pattern);
        assert!(got.iter().any(|p| p == &sub));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
