// api.js — Tauri IPC 封装（命令名常量 + 事件订阅 + 统一错误处理）
// 静态前端经 withGlobalTauri 使用注入的全局 API，无需 import、无需 node_modules。

const g = window.__TAURI__ || {};
const invoke = g.core ? g.core.invoke : null;
const listen = g.event ? g.event.listen : null;

/** 命令名集中常量，杜绝手写字符串漂移。 */
export const CMD = {
  getCategories: "get_categories",
  getDiskInfo: "get_disk_info",
  scanAll: "scan_all",
  scanCategory: "scan_category",
  previewClean: "preview_clean",
  executeClean: "execute_clean",
  cancelOperation: "cancel_operation",
  getSettings: "get_settings",
  saveSettings: "save_settings",
  startMonitor: "start_monitor",
  stopMonitor: "stop_monitor",
  getMonitorStatus: "get_monitor_status",
  openPath: "open_path",
  exportPreview: "export_preview",
};

/** 事件名集中常量。 */
export const EVT = {
  scanCategory: "scan://category",
  scanProgress: "scan://progress",
  cleanProgress: "clean://progress",
  monitorEvent: "monitor://event",
  monitorStatus: "monitor://status",
};

/** 将任意错误规整为 { code, message }。 */
export function normalizeError(e) {
  if (e && typeof e === "object" && "code" in e && "message" in e) {
    return { code: String(e.code), message: String(e.message) };
  }
  if (e && typeof e === "object" && "message" in e) {
    return { code: "E_UNKNOWN", message: String(e.message) };
  }
  return { code: "E_UNKNOWN", message: String(e) };
}

/** 调用一个 Tauri 命令，失败时抛出规整后的错误。 */
export async function call(cmd, args = {}) {
  if (!invoke) {
    throw { code: "E_NO_TAURI", message: "Tauri 全局 API 未注入（请以桌面应用方式运行）。" };
  }
  try {
    return await invoke(cmd, args);
  } catch (e) {
    throw normalizeError(e);
  }
}

/** 订阅一个后端事件，返回 unlisten 句柄（Promise）。 */
export function on(evt, handler) {
  if (!listen) return Promise.resolve(() => {});
  return listen(evt, (e) => handler(e.payload));
}

export const isTauri = () => !!invoke;
