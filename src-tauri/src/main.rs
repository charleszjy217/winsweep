// WinSweep 入口：Release 下隐藏控制台窗口（便携 exe）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    winsweep_lib::run()
}
