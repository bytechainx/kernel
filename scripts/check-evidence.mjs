#!/usr/bin/env node
// 校验 docs/kernel-prod-001 下所有 evidence.json；证据缺失或字段不实即 Gate 失败。
// 真实性校验（KERNEL-012，2026-11-08）：commit_sha 必须能在当前仓对象库解析且为 HEAD 祖先；
// evidence 目录名须等于 commit_sha；timestamp_utc 须严格可解析；PASS 的 ci_run_url 须指向本仓 Actions Run。
// 内容发现退出码 1；git 不可用或非仓库为工具错误退出码 2。
import { readFileSync, readdirSync, statSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { spawnSync } from "node:child_process";

const root = "docs/kernel-prod-001";
const required = ["goal_id", "task_id", "repository", "commit_sha", "toolchain", "checks", "result", "timestamp_utc"];
const ISO_8601 = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/;
const RUN_URL = /^https:\/\/github\.com\/bytechainx\/kernel\/actions\/runs\/\d+(\/job\/\d+)?\/?$/;

function toolError(message) {
  console.error(`Evidence Gate 工具错误：${message}`);
  process.exit(2);
}

function git(args) {
  const result = spawnSync("git", args, { encoding: "utf8" });
  if (result.error) toolError(`git ${args.join(" ")}: ${result.error.message}`);
  if (result.status === 128) toolError(`git ${args.join(" ")}: ${(result.stderr || "").trim()}`);
  return result;
}

function commitIntegrity(file, sha) {
  const problems = [];
  if (basename(dirname(file)) !== sha) {
    problems.push(`${file}: 目录名与 commit_sha 不一致`);
  }
  if (git(["cat-file", "-e", `${sha}^{commit}`]).status !== 0) {
    problems.push(`${file}: commit_sha ${sha.slice(0, 12)} 不在当前仓对象库`);
  } else if (git(["merge-base", "--is-ancestor", sha, "HEAD"]).status !== 0) {
    problems.push(`${file}: commit_sha ${sha.slice(0, 12)} 不是当前 HEAD 的祖先`);
  }
  return problems;
}

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
  if (!/^[0-9a-f]{40}$/.test(data.commit_sha || "")) {
    bad.push(`${file}: commit_sha 非 40 位小写 hex`);
  } else {
    bad.push(...commitIntegrity(file, data.commit_sha));
  }
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
  if (!ISO_8601.test(data.timestamp_utc || "") || Number.isNaN(Date.parse(data.timestamp_utc))) {
    bad.push(`${file}: timestamp_utc 非严格可解析 ISO-8601`);
  }
  if (data.result === "PASS") {
    if (!data.ci_run_url) {
      bad.push(`${file}: PASS 缺 ci_run_url`);
    } else if (!RUN_URL.test(data.ci_run_url)) {
      bad.push(`${file}: PASS 的 ci_run_url 不指向本仓 Actions Run`);
    }
  }
}

if (bad.length) {
  console.error("Evidence Gate 失败：\n" + bad.join("\n"));
  process.exit(1);
}
console.log(`evidence gate: OK（${files.length} 份证据，含真实性校验）`);
