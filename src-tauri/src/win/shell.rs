//! 平台薄封装：Shell / 磁盘 API 的 `unsafe` FFI 收敛点。
//!
//! 直接以 `extern "system"` 绑定 `shell32.dll` / `kernel32.dll`。
//! 与架构 §6.1 规划的 windows crate 用法**语义一致**，但省去 crate 版本签名差异风险，
//! 并进一步压缩体积。

#![allow(non_snake_case)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

/// `SHQUERYRBINFO`（与 Windows SDK 定义布局一致：DWORD + __int64 + __int64 = 24 字节）。
#[repr(C)]
struct ShQueryRbInfo {
    cb_size: u32,
    i64_size: i64,
    i64_num_items: i64,
}

const SHERB_NOCONFIRMATION: u32 = 0x0000_0001;
const SHERB_NOPROGRESSUI: u32 = 0x0000_0002;
const SHERB_NOSOUND: u32 = 0x0000_0004;

#[link(name = "shell32")]
extern "system" {
    fn SHQueryRecycleBinW(
        pszRootPath: *const u16,
        pSHQueryRBInfo: *mut ShQueryRbInfo,
    ) -> i32;
    fn SHEmptyRecycleBinW(
        hwnd: *mut core::ffi::c_void,
        pszRootPath: *const u16,
        dwFlags: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailableToCaller: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> i32;
}

fn to_wide(s: &OsStr) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_wide().collect();
    v.push(0);
    v
}

/// 查询回收站占用：返回 `(字节数, 项数)`。
///
/// `pszRootPath = null` → 汇总所有驱动器。`cbSize` 调用前必设为 `size_of`。
pub fn query_recycle_bin() -> (u64, u64) {
    unsafe {
        let mut info = ShQueryRbInfo {
            cb_size: std::mem::size_of::<ShQueryRbInfo>() as u32,
            i64_size: 0,
            i64_num_items: 0,
        };
        let hr = SHQueryRecycleBinW(std::ptr::null(), &mut info);
        if hr >= 0 {
            (
                info.i64_size.max(0) as u64,
                info.i64_num_items.max(0) as u64,
            )
        } else {
            (0, 0)
        }
    }
}

/// 清空回收站（不可恢复）。带 `SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND`。
pub fn empty_recycle_bin() -> Result<(), String> {
    unsafe {
        let flags = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND;
        let hr = SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), flags);
        if hr >= 0 {
            Ok(())
        } else {
            Err(format!("SHEmptyRecycleBinW 失败 (HRESULT 0x{:08X})", hr))
        }
    }
}

/// 查询磁盘容量：返回 `(总字节, 可用字节)`。`drive` 形如 `C:\`。
pub fn get_disk_free(drive: &str) -> Result<(u64, u64), String> {
    let root = if drive.trim().is_empty() {
        "C:\\".to_string()
    } else if drive.ends_with('\\') || drive.ends_with('/') {
        drive.to_string()
    } else {
        format!("{drive}\\")
    };
    let w = to_wide(OsStr::new(&root));
    unsafe {
        let mut free_to_caller: u64 = 0;
        let mut total: u64 = 0;
        let mut total_free: u64 = 0;
        let ok = GetDiskFreeSpaceExW(
            w.as_ptr(),
            &mut free_to_caller,
            &mut total,
            &mut total_free,
        );
        if ok != 0 {
            Ok((total, total_free))
        } else {
            Err(format!("GetDiskFreeSpaceExW 失败：{root}"))
        }
    }
}
