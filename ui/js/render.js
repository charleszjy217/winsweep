// render.js — DOM 渲染：类别表 / 汇总 / 磁盘条 / 进度 / 预览 / 确认 / 报告 / 监控 / 设置

import { state } from "./state.js";
import { formatBytes, formatCount, formatDuration, riskLabel } from "./format.js";

const $ = (id) => document.getElementById(id);

function make(tag, className, text) {
  const n = document.createElement(tag);
  if (className) n.className = className;
  if (text !== undefined && text !== null) n.textContent = String(text);
  return n;
}

const RISK_CLASS = { low: "risk-low", medium: "risk-medium", high: "risk-high" };
const RISK_TEXT = { low: "risk-text-low", medium: "risk-text-medium", high: "risk-text-high" };

// ── 磁盘条 ──
export function renderDisk(disk) {
  if (!disk) return;
  const used = Math.max(0, (disk.totalBytes || 0) - (disk.freeBytes || 0));
  const pct = disk.totalBytes > 0 ? (used / disk.totalBytes) * 100 : 0;
  $("diskLabel").textContent =
    `${(disk.drive || "C:\\").replace(/\\+$/, "\\")} 可用 ${formatBytes(disk.freeBytes)} / 总 ${formatBytes(disk.totalBytes)}`;
  $("diskFill").style.width = `${pct.toFixed(1)}%`;
}

export function setScanStatus(text) {
  $("scanStatus").textContent = text;
}

export function setScanTime(ms) {
  $("scanTime").textContent = `扫描耗时: ${formatDuration(ms)}`;
}

// ── 类别表 ──
export function renderCategories() {
  const body = $("catBody");
  body.textContent = "";
  const emptyEl = $("catEmpty");

  if (!state.categories.length) {
    emptyEl.classList.remove("hidden");
    return;
  }
  emptyEl.classList.add("hidden");

  for (const c of state.categories) {
    body.appendChild(buildRow(c));
  }
}

function statusFlag(res) {
  if (!res) return "";
  if (res.status === "notFound") return "未安装/不存在";
  if (res.status === "skipped") return "已跳过";
  if (res.status === "error") return "错误";
  return "";
}

function buildRow(cat) {
  const res = state.results.get(cat.id) || null;
  const row = make("div", "cat-row");
  row.dataset.id = cat.id;
  if (state.selected.has(cat.id)) row.classList.add("selected");
  if (cat.risk === "high") row.classList.add("high");

  // checkbox
  const checkCell = make("span", "col-check");
  const cb = document.createElement("input");
  cb.type = "checkbox";
  cb.dataset.id = cat.id;
  cb.checked = state.selected.has(cat.id);
  checkCell.appendChild(cb);
  row.appendChild(checkCell);

  // name + desc
  const nameCell = make("span", "col-name");
  const nameLine = make("span", "cat-name", cat.name);
  if (cat.semantics === "emptyRecycleBin") {
    nameLine.appendChild(make("span", "status-flag", "  [清空回收站·不可恢复]"));
  }
  if (cat.requiresAdmin) {
    nameLine.appendChild(make("span", "status-flag", "  ⚠需管理员"));
  }
  nameCell.appendChild(nameLine);
  nameCell.appendChild(make("span", "cat-desc", cat.description || ""));
  row.appendChild(nameCell);

  // size
  row.appendChild(make("span", "col-size", res ? formatBytes(res.totalSize) : "--"));
  // count
  row.appendChild(make("span", "col-count", res ? formatCount(res.fileCount) : "--"));

  // risk
  const riskCell = make("span", `col-risk ${RISK_TEXT[cat.risk]}`);
  riskCell.appendChild(make("span", `risk-dot ${RISK_CLASS[cat.risk]}`));
  riskCell.appendChild(document.createTextNode(cat.risk === "high" ? "高风险(默认不选)" : riskLabel(cat.risk)));
  row.appendChild(riskCell);

  // selected indicator
  const sel = make("span", "col-sel");
  const flag = statusFlag(res);
  sel.textContent = flag || (state.selected.has(cat.id) ? "✓ 已选" : "—");
  if (flag) sel.classList.add("status-flag");
  row.appendChild(sel);

  return row;
}

export function updateRow(id) {
  const old = document.querySelector(`.cat-row[data-id="${cssEscape(id)}"]`);
  if (!old) return;
  const cat = state.categories.find((c) => c.id === id);
  if (!cat) return;
  old.replaceWith(buildRow(cat));
}

function cssEscape(s) {
  return String(s).replace(/["\\]/g, "\\$&");
}

export function renderSummary() {
  let count = 0;
  let total = 0;
  for (const id of state.selected) {
    count += 1;
    const r = state.results.get(id);
    if (r) total += r.totalSize || 0;
  }
  $("summaryText").textContent = `已选: ${count} 类 · 合计 ${formatBytes(total)}`;
  const allChecked = state.categories.length > 0 && state.selected.size === state.categories.length;
  $("selectAll").checked = allChecked;
}

// ── 进度 ──
export function showProgress() {
  $("progressWrap").classList.remove("hidden");
  $("progressFill").style.width = "0%";
  $("progressPath").textContent = "准备中…";
}

export function hideProgress() {
  $("progressWrap").classList.add("hidden");
}

export function updateProgress(p) {
  let pct = 0;
  if (typeof p.total === "number" && p.total > 0) pct = (p.processed / p.total) * 100;
  else if (typeof p.totalCategories === "number" && p.totalCategories > 0)
    pct = (p.doneCategories / p.totalCategories) * 100;
  $("progressFill").style.width = `${Math.min(100, pct).toFixed(1)}%`;
  if (p.currentPath) $("progressPath").textContent = p.currentPath;
  else if (typeof p.scannedFiles === "number")
    $("progressPath").textContent = `已扫描 ${formatCount(p.scannedFiles)} 个文件 · ${formatBytes(p.scannedBytes || 0)}`;
}

// ── 预览面板 ──
export function openPreview(plan) {
  state.previewPlan = plan;
  $("previewModal").classList.remove("hidden");
  const modeText = plan.mode === "permanent" ? "永久删除(不可恢复)" : "移入回收站(可恢复)";
  $("previewSummary").textContent =
    `将处理 ${plan.categoryIds.length} 个类别 · 共 ${formatCount(plan.totalFiles)} 个文件 · 合计 ${formatBytes(plan.totalSize)}    [模式: ${modeText}]`;

  const warnBox = $("previewWarnings");
  warnBox.textContent = "";
  for (const w of plan.warnings || []) {
    const danger = /不可恢复|永久删除/.test(w);
    warnBox.appendChild(make("div", `warn-item${danger ? " danger" : ""}`, w));
  }
  if (plan.entriesTruncated) {
    warnBox.appendChild(make("div", "warn-item", "清单条目较多，仅显示前 1000 条（完整数据仍按类别处理）。"));
  }
  $("previewSearch").value = "";
  renderPreviewList("");
}

export function closePreview() {
  $("previewModal").classList.add("hidden");
}

export function renderPreviewList(filter) {
  const list = $("previewList");
  list.textContent = "";
  const plan = state.previewPlan;
  if (!plan) return;
  const f = (filter || "").toLowerCase();
  const items = (plan.entries || []).filter((e) => !f || e.path.toLowerCase().includes(f));
  if (!items.length) {
    list.appendChild(make("div", "preview-item", "（无匹配条目）"));
    return;
  }
  for (const e of items) {
    const row = make("div", "preview-item");
    row.appendChild(make("span", "p-path", e.path));
    row.appendChild(make("span", "p-size", formatBytes(e.size)));
    list.appendChild(row);
  }
}

// ── 二次确认弹窗 ──
export function openConfirm(plan) {
  $("confirmModal").classList.remove("hidden");
  const body = $("confirmBody");
  body.textContent = "";
  const hasRecycle = (plan.categoryIds || []).length > 0 &&
    plan.warnings.some((w) => w.includes("回收站内容无法再次移入回收站"));
  const modeText = plan.mode === "permanent" ? "永久删除（不可恢复）" : "移入系统回收站（可从此电脑回收站恢复）";

  const l1 = make("div");
  l1.appendChild(make("span", "big", `⚠ 即将清理 ${plan.categoryIds.length} 个类别`));
  l1.appendChild(document.createTextNode(`，共 ${formatBytes(plan.totalSize)} / ${formatCount(plan.totalFiles)} 文件`));
  body.appendChild(l1);

  body.appendChild(make("div", "", `模式：${modeText}`));
  if (plan.hasHighRisk) {
    body.appendChild(make("div", "stat-bad", "包含高风险类别，已额外警示。"));
  }
  if (hasRecycle) {
    const w = make("div", "warn-item danger", "包含『回收站』：将执行『清空回收站』，回收站内容无法再次移入回收站，此操作不可恢复！");
    body.appendChild(w);
  }
  body.appendChild(make("div", "hint", "此操作不会删除任何受保护的系统文件（黑名单 + 允许根双重校验）。"));
}

export function closeConfirm() {
  $("confirmModal").classList.add("hidden");
}

// ── 结果报告 ──
export function openReport(report) {
  $("reportModal").classList.remove("hidden");
  const body = $("reportBody");
  body.textContent = "";

  const head = make("div");
  head.appendChild(make("div", `big ${report.dryRun ? "stat-warn" : "stat-ok"}`,
    report.dryRun ? "（模拟清理 dry-run，未改动任何文件）" : "清理完成"));
  body.appendChild(head);

  const sum = make("div");
  sum.appendChild(rowKV("释放空间", formatBytes(report.freedBytes), "stat-ok"));
  sum.appendChild(rowKV("成功处理", formatCount(report.deletedCount), "stat-ok"));
  sum.appendChild(rowKV("跳过", formatCount(report.skippedCount), "stat-warn"));
  sum.appendChild(rowKV("失败", formatCount(report.failedCount), "stat-bad"));
  sum.appendChild(rowKV("模式", report.mode === "permanent" ? "永久删除" : "移入回收站", ""));
  body.appendChild(sum);

  if (report.perCategory && report.perCategory.length) {
    const table = make("table");
    const thead = make("tr");
    for (const h of ["类别", "释放", "成功", "跳过", "失败"]) thead.appendChild(make("th", "", h));
    table.appendChild(thead);
    for (const c of report.perCategory) {
      const tr = make("tr");
      tr.appendChild(make("td", "", c.name));
      tr.appendChild(make("td", "stat-ok", formatBytes(c.freedBytes)));
      tr.appendChild(make("td", "", formatCount(c.deletedCount)));
      tr.appendChild(make("td", "stat-warn", formatCount(c.skippedCount)));
      tr.appendChild(make("td", "stat-bad", formatCount(c.failedCount)));
      table.appendChild(tr);
    }
    body.appendChild(table);
  }

  if (report.skippedSamples && report.skippedSamples.length) {
    const h = make("div", "notes", `跳过样本（最多 200 条）：`);
    body.appendChild(h);
    const ul = make("ul", "notes");
    for (const s of report.skippedSamples.slice(0, 50)) {
      ul.appendChild(make("li", "", `${s.path} — ${s.reason}`));
    }
    body.appendChild(ul);
  }

  if (report.notes && report.notes.length) {
    const ul = make("ul", "notes");
    for (const n of report.notes) ul.appendChild(make("li", "", n));
    body.appendChild(ul);
  }
}

function rowKV(k, v, cls) {
  const r = make("div", "r-row");
  r.appendChild(make("span", "", k));
  r.appendChild(make("span", cls || "", v));
  return r;
}

export function closeReport() {
  $("reportModal").classList.add("hidden");
}

// ── 监控 ──
export function renderMonitorToggle(on) {
  const btn = $("monitorToggle");
  btn.classList.toggle("on", on);
  $("monitorLabel").textContent = on ? "ON" : "OFF";
  $("monitorPanel").classList.toggle("hidden", !on);
}

export function appendMonitorEvent(ev) {
  state.monitorEvents.unshift(ev);
  if (state.monitorEvents.length > 200) state.monitorEvents.pop();
  const list = $("monitorList");
  const row = make("div", "monitor-item");
  row.appendChild(make("span", "m-path", ev.path));
  row.appendChild(make("span", `m-action ${ev.action}`, ev.action === "trashed" ? "已移入回收站" : "提醒"));
  list.prepend(row);
}

export function setMonitorStatus(text) {
  $("monitorStatus").textContent = text;
}

// ── 设置抽屉 ──
export function renderBlacklist(prefixes) {
  const box = $("blacklistView");
  box.textContent = "";
  for (const p of prefixes) box.appendChild(make("code", "", p));
}

export function renderCustomCats(cats) {
  const box = $("customCatsView");
  box.textContent = "";
  if (!cats || !cats.length) {
    box.appendChild(make("div", "none", "（暂无自定义类别）"));
    return;
  }
  for (const c of cats) box.appendChild(make("code", "", `${c.name} → ${c.path} [${c.risk}]`));
}

export function renderExcludeChips() {
  const box = $("excludeChips");
  box.textContent = "";
  for (const r of state.excludeRules) {
    const chip = make("span", "chip", "");
    chip.appendChild(document.createTextNode(r));
    const x = make("button", "", "✕");
    x.dataset.rule = r;
    chip.appendChild(x);
    box.appendChild(chip);
  }
}

// ── Toast ──
export function toast(msg, type) {
  const box = $("toastWrap");
  const t = make("div", `toast${type ? " " + type : ""}`, msg);
  box.appendChild(t);
  setTimeout(() => t.remove(), 3800);
}

// ── 数字动效（克制的滚动/微光） ──
export function flash(el) {
  if (!el) return;
  el.style.transition = "color .2s";
  el.style.color = "var(--ok)";
  setTimeout(() => { el.style.color = ""; }, 260);
}
