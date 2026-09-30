// main.js — 入口：绑定事件、初始化、串起 api / state / render

import { CMD, EVT, call, on, isTauri } from "./api.js";
import { state, selectAll, invertSelection, resetResults } from "./state.js";
import * as R from "./render.js";
import { formatBytes, formatCount } from "./format.js";

const $ = (id) => document.getElementById(id);

// 与 safety.rs 保持一致（仅用于设置面板只读展示）
const BLACKLIST = [
  "C:\\Windows\\System32", "C:\\Windows\\SysWOW64", "C:\\Windows\\WinSxS",
  "C:\\Windows\\Boot", "C:\\Windows\\Fonts", "C:\\Windows\\assembly",
  "C:\\Windows\\Microsoft.NET", "C:\\ProgramData\\Microsoft\\Windows\\Start Menu",
  "C:\\Program Files", "C:\\Program Files (x86)", "C:\\$Recycle.Bin",
  "C:\\Recovery", "C:\\Boot",
];

const DEFAULT_DRIVE = "C:\\";

// ─────────────────────────── 启动 ───────────────────────────

async function init() {
  if (!isTauri()) {
    R.toast("未检测到 Tauri 运行时，界面可能无法调用后端。", "err");
  }
  bindUi();
  subscribeEvents();

  try {
    state.categories = await call(CMD.getCategories);
    // 默认勾选 defaultSelected
    state.selected = new Set(state.categories.filter((c) => c.defaultSelected).map((c) => c.id));
  } catch (e) {
    R.toast(`加载类别失败：${e.message}`, "err");
  }

  try {
    const s = await call(CMD.getSettings);
    applySettings(s, false);
  } catch (e) {
    R.toast(`加载设置失败：${e.message}`, "err");
  }

  try {
    state.disk = await call(CMD.getDiskInfo, { drive: DEFAULT_DRIVE });
    R.renderDisk(state.disk);
  } catch (e) {
    /* 磁盘信息失败不阻塞 */
  }

  try {
    const st = await call(CMD.getMonitorStatus);
    state.monitorOn = !!st.running;
    R.renderMonitorToggle(state.monitorOn);
    R.setMonitorStatus(st.running ? `监控中 · 事件 ${st.eventsCount}` : "空闲");
  } catch (e) {
    /* ignore */
  }

  R.renderCategories();
  R.renderSummary();
  updateBusyButtons();
}

// ─────────────────────────── 事件订阅 ───────────────────────────

function subscribeEvents() {
  on(EVT.scanCategory, (res) => {
    state.results.set(res.id, res);
    R.updateRow(res.id);
    R.renderSummary();
  });

  on(EVT.scanProgress, (p) => R.updateProgress(p));
  on(EVT.cleanProgress, (p) => R.updateProgress(p));

  on(EVT.monitorEvent, (ev) => R.appendMonitorEvent(ev));

  on(EVT.monitorStatus, (st) => {
    state.monitorOn = !!st.running;
    R.renderMonitorToggle(state.monitorOn);
    R.setMonitorStatus(st.running ? `监控中 · ${st.dirs.length} 目录 · 事件 ${st.eventsCount}` : "空闲");
  });
}

// ─────────────────────────── UI 绑定 ───────────────────────────

function bindUi() {
  $("scanBtn").addEventListener("click", doScan);
  $("cancelBtn").addEventListener("click", doCancel);

  $("dryRunToggle").addEventListener("click", () => {
    state.dryRun = !state.dryRun;
    const b = $("dryRunToggle");
    b.textContent = `模拟清理 dry-run: ${state.dryRun ? "ON" : "OFF"}`;
    b.classList.toggle("active", state.dryRun);
  });

  $("openRecycleBtn").addEventListener("click", async () => {
    try { await call(CMD.openPath, { path: "recyclebin" }); }
    catch (e) { R.toast(`打开回收站失败：${e.message}`, "err"); }
  });

  $("selectAll").addEventListener("change", (e) => {
    selectAll(e.target.checked);
    R.renderCategories();
    R.renderSummary();
  });

  $("invertBtn").addEventListener("click", () => {
    invertSelection();
    R.renderCategories();
    R.renderSummary();
  });

  // 类别表：勾选与整行点击
  $("catBody").addEventListener("change", (e) => {
    const id = e.target && e.target.dataset ? e.target.dataset.id : null;
    if (!id) return;
    toggleSelect(id, e.target.checked);
  });
  $("catBody").addEventListener("click", (e) => {
    if (e.target.tagName === "INPUT") return;
    const row = e.target.closest(".cat-row");
    if (!row) return;
    const id = row.dataset.id;
    toggleSelect(id, !state.selected.has(id));
  });

  // 模式单选
  document.querySelectorAll('input[name="mode"]').forEach((r) => {
    r.addEventListener("change", (e) => { state.mode = e.target.value; });
  });

  // 预览 / 清理
  $("previewBtn").addEventListener("click", () => doPreview(true));
  $("cleanBtn").addEventListener("click", () => doPreview(false));

  $("previewSearch").addEventListener("input", (e) => R.renderPreviewList(e.target.value));
  $("previewCancel").addEventListener("click", R.closePreview);
  $("previewContinue").addEventListener("click", () => { R.closePreview(); R.openConfirm(state.previewPlan); });
  $("previewExport").addEventListener("click", doExport);

  $("confirmBack").addEventListener("click", () => { R.closeConfirm(); R.openPreview(state.previewPlan); });
  $("confirmGo").addEventListener("click", doExecute);

  $("reportClose").addEventListener("click", () => { R.closeReport(); refreshDisk(); });

  // 设置抽屉
  $("settingsBtn").addEventListener("click", openSettings);
  $("drawerClose").addEventListener("click", () => $("settingsDrawer").classList.add("hidden"));
  $("saveSettings").addEventListener("click", doSaveSettings);
  $("excludeAdd").addEventListener("click", () => {
    const v = $("excludeInput").value.trim();
    if (v && !state.excludeRules.includes(v)) {
      state.excludeRules.push(v);
      $("excludeInput").value = "";
      R.renderExcludeChips();
    }
  });
  $("excludeChips").addEventListener("click", (e) => {
    const rule = e.target && e.target.dataset ? e.target.dataset.rule : null;
    if (!rule) return;
    state.excludeRules = state.excludeRules.filter((r) => r !== rule);
    R.renderExcludeChips();
  });

  // 监控
  $("monitorToggle").addEventListener("click", toggleMonitor);
  $("monitorPanelClose").addEventListener("click", () => $("monitorPanel").classList.add("hidden"));
}

function toggleSelect(id, checked) {
  if (checked) state.selected.add(id); else state.selected.delete(id);
  R.updateRow(id);
  R.renderSummary();
}

function updateBusyButtons() {
  const busy = state.scanning || state.cleaning;
  $("scanBtn").disabled = busy;
  $("cleanBtn").disabled = busy;
  $("previewBtn").disabled = busy;
  $("cancelBtn").classList.toggle("hidden", !busy);
}

// ─────────────────────────── 扫描 ───────────────────────────

async function doScan() {
  if (state.scanning || state.cleaning) return;
  state.scanning = true;
  updateBusyButtons();
  resetResultsKeepingDefaults();
  R.renderCategories();
  R.renderSummary();
  R.showProgress();
  R.setScanStatus("扫描中…");

  try {
    const report = await call(CMD.scanAll);
    state.results = new Map(report.categories.map((c) => [c.id, c]));
    state.disk = report.disk;
    R.renderDisk(report.disk);
    R.renderCategories();
    R.renderSummary();
    R.setScanTime(report.durationMs);
    R.setScanStatus(
      `已扫描: ${report.categories.length} 类 · ${formatCount(report.totalFiles)} 文件 · ${formatBytes(report.totalSize)}`
    );
    R.flash($("summaryText"));
  } catch (e) {
    if (e.code === "E_CANCELLED") R.toast("扫描已取消", "warn");
    else R.toast(`扫描失败：${e.message}`, "err");
  } finally {
    state.scanning = false;
    R.hideProgress();
    updateBusyButtons();
  }
}

function resetResultsKeepingDefaults() {
  state.results = new Map();
  // 保留当前勾选（若为空则回退默认勾选）
  if (state.selected.size === 0) {
    state.selected = new Set(state.categories.filter((c) => c.defaultSelected).map((c) => c.id));
  }
}

async function doCancel() {
  try {
    await call(CMD.cancelOperation);
    R.toast("已请求取消当前操作", "warn");
  } catch (e) {
    R.toast(`取消失败：${e.message}`, "err");
  }
}

// ─────────────────────────── 预览 / 清理 ───────────────────────────

async function buildPreviewPlan() {
  const categoryIds = [...state.selected];
  if (categoryIds.length === 0) {
    R.toast("请先勾选要清理的类别", "warn");
    return null;
  }
  return await call(CMD.previewClean, { req: { categoryIds, mode: state.mode } });
}

async function doPreview(showPanel) {
  if (state.scanning || state.cleaning) return;
  try {
    const plan = await buildPreviewPlan();
    if (!plan) return;
    if (showPanel) R.openPreview(plan);
    else { state.previewPlan = plan; R.openConfirm(plan); }
  } catch (e) {
    R.toast(`生成预览失败：${e.message}`, "err");
  }
}

async function doExecute() {
  if (state.scanning || state.cleaning) return;
  R.closeConfirm();
  state.cleaning = true;
  updateBusyButtons();
  R.showProgress();
  R.setScanStatus(state.dryRun ? "模拟清理中…" : "清理中…");

  try {
    const report = await call(CMD.executeClean, {
      req: {
        categoryIds: [...state.selected],
        mode: state.mode,
        dryRun: state.dryRun,
        confirmed: true,
      },
    });
    R.hideProgress();
    R.openReport(report);
  } catch (e) {
    if (e.code === "E_NOT_CONFIRMED") R.toast("未确认，已取消清理", "warn");
    else if (e.code === "E_BUSY") R.toast("已有任务进行中", "warn");
    else R.toast(`清理失败：${e.message}`, "err");
  } finally {
    state.cleaning = false;
    R.hideProgress();
    updateBusyButtons();
  }
}

async function doExport() {
  try {
    const name = `WinSweep-清理清单-${Date.now()}.txt`;
    const full = await call(CMD.exportPreview, { path: name, plan: state.previewPlan });
    R.toast(`已导出清单：${full}`, "");
  } catch (e) {
    R.toast(`导出失败：${e.message}`, "err");
  }
}

async function refreshDisk() {
  try {
    state.disk = await call(CMD.getDiskInfo, { drive: DEFAULT_DRIVE });
    R.renderDisk(state.disk);
  } catch (e) { /* ignore */ }
}

// ─────────────────────────── 监控 ───────────────────────────

async function toggleMonitor() {
  try {
    if (state.monitorOn) {
      const st = await call(CMD.stopMonitor);
      state.monitorOn = !!st.running;
      R.renderMonitorToggle(state.monitorOn);
      R.setMonitorStatus("空闲");
      R.toast("实时监控已关闭", "");
    } else {
      const dirs = (state.settings && state.settings.monitorDirs && state.settings.monitorDirs.length)
        ? state.settings.monitorDirs : ["%TEMP%"];
      const policy = (state.settings && state.settings.monitorPolicy) || "notifyOnly";
      const st = await call(CMD.startMonitor, { config: { dirs, policy } });
      state.monitorOn = !!st.running;
      R.renderMonitorToggle(state.monitorOn);
      R.setMonitorStatus(`监控中 · ${st.dirs.length} 目录 · 事件 ${st.eventsCount}`);
      R.toast("实时监控已开启（默认仅提醒，不自动永久删除）", "");
    }
  } catch (e) {
    R.toast(`监控操作失败：${e.message}`, "err");
  }
}

// ─────────────────────────── 设置 ───────────────────────────

function openSettings() {
  const s = state.settings || {};
  $("themeSelect").value = s.theme || "dark";
  $("fontSelect").value = s.font || "Consolas";
  setRadio("defmode", s.defaultMode || "recycleBin");
  setRadio("monpolicy", s.monitorPolicy || "notifyOnly");
  $("monitorDirsInput").value = (s.monitorDirs || ["%TEMP%"]).join(";");
  R.renderBlacklist(BLACKLIST);
  R.renderCustomCats(s.customCategories || []);
  R.renderExcludeChips();
  $("settingsDrawer").classList.remove("hidden");
}

function setRadio(name, value) {
  document.querySelectorAll(`input[name="${name}"]`).forEach((r) => { r.checked = r.value === value; });
}

function getRadio(name) {
  const el = document.querySelector(`input[name="${name}"]:checked`);
  return el ? el.value : null;
}

async function doSaveSettings() {
  const prev = state.settings || {};
  const dirsRaw = $("monitorDirsInput").value.trim();
  const monitorDirs = dirsRaw ? dirsRaw.split(";").map((d) => d.trim()).filter(Boolean) : ["%TEMP%"];

  const settings = {
    defaultMode: getRadio("defmode") || "recycleBin",
    monitorEnabled: state.monitorOn,
    monitorDirs,
    monitorPolicy: getRadio("monpolicy") || "notifyOnly",
    customCategories: prev.customCategories || [],
    theme: $("themeSelect").value,
    font: $("fontSelect").value,
  };

  try {
    const saved = await call(CMD.saveSettings, { settings });
    applySettings(saved, true);
    $("settingsDrawer").classList.add("hidden");
    R.toast("设置已保存", "");
  } catch (e) {
    R.toast(`保存失败：${e.message}`, "err");
  }
}

function applySettings(s, syncMode) {
  state.settings = s;
  if (syncMode) {
    state.mode = s.defaultMode || "recycleBin";
  } else {
    state.mode = s.defaultMode || "recycleBin";
  }
  // 同步底部模式单选
  setRadio("mode", state.mode);
  // 主题
  document.documentElement.classList.toggle("theme-black", s.theme === "black");
  // 字体
  const font = s.font || "Consolas";
  document.documentElement.style.setProperty("--mono", `"${font}", "Cascadia Code", Consolas, monospace`);
}

// ─────────────────────────── go ───────────────────────────

window.addEventListener("DOMContentLoaded", init);
