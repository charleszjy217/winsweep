// state.js — 前端内存状态（扫描结果 / 勾选 / 模式 / 设置）

export const state = {
  /** @type {Array} 类别定义（get_categories 返回） */
  categories: [],
  /** @type {Map<string, object>} id -> CategoryResult */
  results: new Map(),
  /** @type {Set<string>} 已勾选类别 id */
  selected: new Set(),
  /** 清理模式: "recycleBin" | "permanent" */
  mode: "recycleBin",
  /** 模拟清理 dry-run 开关 */
  dryRun: false,
  /** @type {object|null} Settings */
  settings: null,
  /** 最近一次预览计划 */
  previewPlan: null,
  /** 监控开关 */
  monitorOn: false,
  /** 监控事件列表 */
  monitorEvents: [],
  /** 排除规则（本地记录展示） */
  excludeRules: [],
  /** 忙碌标志 */
  scanning: false,
  cleaning: false,
  /** 磁盘容量 */
  disk: null,
};

/** 全选（仅勾选可扫描/存在的类别，默认含高风险？否：全选=全部类别）。 */
export function selectAll(checked) {
  state.selected.clear();
  if (checked) {
    for (const c of state.categories) state.selected.add(c.id);
  }
}

/** 反选。 */
export function invertSelection() {
  const next = new Set();
  for (const c of state.categories) {
    if (!state.selected.has(c.id)) next.add(c.id);
  }
  state.selected = next;
}

/** 复位扫描结果（保留类别定义）。 */
export function resetResults() {
  state.results = new Map();
  state.selected = new Set();
}
