#!/usr/bin/env node
// 比对 src/lib.rs 的模块路径与根导出，防止未经审查的公开面变化。
import { readFileSync } from "node:fs";

const baselinePath = "docs/kernel-prod-001/api-baseline.json";
const baseline = JSON.parse(readFileSync(baselinePath, "utf8"));
const lib = readFileSync("src/lib.rs", "utf8");

const modules = [...lib.matchAll(/^pub mod (\w+);/gm)].map((m) => m[1]).sort();
const reexports = [...lib.matchAll(/pub use\s+[^;]*?\{([\s\S]*?)\}\s*;/g)]
  .flatMap((m) => m[1].split(/[,\n]/).map((s) => s.trim()).filter(Boolean))
  .sort();

function diff(actual, expected) {
  return {
    added: actual.filter((x) => !expected.includes(x)),
    removed: expected.filter((x) => !actual.includes(x)),
  };
}

const moduleDiff = diff(modules, baseline.modules);
const exportDiff = diff(reexports, baseline.root_reexports);
const problems = [];
if (moduleDiff.added.length || moduleDiff.removed.length) {
  problems.push(`模块：新增 ${moduleDiff.added.join(",") || "无"}，移除 ${moduleDiff.removed.join(",") || "无"}`);
}
if (exportDiff.added.length || exportDiff.removed.length) {
  problems.push(`根导出：新增 ${exportDiff.added.join(",") || "无"}，移除 ${exportDiff.removed.join(",") || "无"}`);
}

if (problems.length) {
  console.error("公开 API 基线不一致：\n" + problems.join("\n"));
  console.error(`确认是有意变化后，经 PR 审查更新 ${baselinePath} 与 docs/API.md；不得删除断言或关闭检查。`);
  process.exit(1);
}
console.log(`api baseline: OK（${modules.length} 模块，${reexports.length} 根导出）`);
