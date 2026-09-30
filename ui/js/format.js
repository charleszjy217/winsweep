// format.js — bytes / 数量 / 耗时格式化（1024 进制）

/** 字节数 → 人类可读（1024 进制，GB/MB 对齐）。 */
export function formatBytes(bytes) {
  const b = Number(bytes) || 0;
  if (b < 1024) return `${b} B`;
  const units = ["KB", "MB", "GB", "TB", "PB"];
  let v = b / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  const fixed = v >= 100 ? 0 : v >= 10 ? 1 : 2;
  return `${v.toFixed(fixed)} ${units[i]}`;
}

/** 数字千分位。 */
export function formatCount(n) {
  return (Number(n) || 0).toLocaleString("en-US");
}

/** 毫秒 → 人类可读耗时。 */
export function formatDuration(ms) {
  const m = Number(ms) || 0;
  if (m < 1000) return `${m} ms`;
  const s = m / 1000;
  if (s < 60) return `${s.toFixed(2)} s`;
  const min = Math.floor(s / 60);
  const rest = (s % 60).toFixed(1);
  return `${min} min ${rest} s`;
}

/** 风险等级 → 中文。 */
export function riskLabel(risk) {
  return risk === "low" ? "低风险" : risk === "medium" ? "中风险" : "高风险";
}

/** 截断中间路径，保留尾部。 */
export function shortPath(p, max = 90) {
  const s = String(p || "");
  return s.length > max ? "…" + s.slice(s.length - max) : s;
}
