#!/usr/bin/env node
// 校验 docs/kernel-prod-001 下所有 evidence.json；证据缺失或字段不实即 Gate 失败。
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const root = "docs/kernel-prod-001";
const required = ["goal_id", "task_id", "repository", "commit_sha", "toolchain", "checks", "result", "timestamp_utc"];

function walk(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

const files = walk(root).filter((path) => path.endsWith("evidence.json"));
const bad = [];

if (!files.length) bad.push(`${root}: 未找到 evidence.json`);

for (const file of files) {
  let data;
  try {
    data = JSON.parse(readFileSync(file, "utf8"));
  } catch (error) {
    bad.push(`${file}: JSON 解析失败（${error.message}）`);
    continue;
  }
  for (const key of required) {
    if (!(key in data)) bad.push(`${file}: 缺字段 ${key}`);
  }
  if (!/^[0-9a-f]{40}$/.test(data.commit_sha || "")) bad.push(`${file}: commit_sha 非 40 位小写 hex`);
  if (!data.toolchain?.rustc || !data.toolchain?.cargo) bad.push(`${file}: toolchain 缺 rustc/cargo`);
  if (!Array.isArray(data.checks) || data.checks.length === 0) {
    bad.push(`${file}: checks 为空`);
  } else {
    for (const check of data.checks) {
      if (!check.name || !check.command || check.exit_code !== 0) {
        bad.push(`${file}: ${check.name || "无名检查"} 命令或退出码不完整`);
      }
      if (check.tests) {
        const { total, passed, failed } = check.tests;
        if (!(total > 0) || total !== passed || failed !== 0) {
          bad.push(`${file}: ${check.name} 测试数不守恒或为 0`);
        }
      }
    }
  }
  if (data.result === "PASS" && !data.ci_run_url) bad.push(`${file}: PASS 缺 ci_run_url`);
}

if (bad.length) {
  console.error("Evidence Gate 失败：\n" + bad.join("\n"));
  process.exit(1);
}
console.log(`evidence gate: OK（${files.length} 份证据）`);
