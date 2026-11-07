#!/usr/bin/env node
// 校验依赖闭包许可证字段；缺失或不在允许列表即失败，避免静默引入风险。
import { execFileSync } from "node:child_process";

const allowed = new Set([
  "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib",
  "Unicode-3.0", "Unicode-DFS-2016", "CC0-1.0", "0BSD", "MPL-2.0",
]);

const meta = JSON.parse(
  execFileSync("cargo", ["metadata", "--format-version", "1"], {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  }),
);

// SPDX 的 OR 表示任选其一：存在一个全部组件都获准的选项即合规；AND/WITH 要求组件全获准。
function compliant(expression) {
  if (!expression) return false;
  return expression.split(/\s+OR\s+/).some((option) => {
    const ids = (option.match(/[A-Za-z0-9.+-]+/g) || [])
      .filter((token) => !["AND", "WITH"].includes(token));
    return ids.length > 0 && ids.every((token) => allowed.has(token));
  });
}

const bad = [];
for (const pkg of meta.packages) {
  if (pkg.name === "kernel") continue;
  if (!compliant(pkg.license)) {
    bad.push(`${pkg.name}@${pkg.version}: ${pkg.license || "许可证字段缺失"}`);
  }
}

if (bad.length) {
  console.error("许可证策略不通过：\n" + bad.join("\n"));
  console.error("按 docs/kernel-prod-001/SUPPLY-CHAIN.md 登记例外或升级依赖，不得无条件忽略。");
  process.exit(1);
}
console.log(`license policy: OK（${meta.packages.length - 1} 个依赖）`);
