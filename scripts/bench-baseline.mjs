#!/usr/bin/env node
// 多次采样 hot_path 基准；单次数字不作为阻塞依据，只产出可复核的分布。
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { arch, cpus, platform, release } from "node:os";

const runs = Math.max(1, Number(process.argv[2] || 5));
const multiplier = { ns: 1, "µs": 1e3, us: 1e3, ms: 1e6, s: 1e9 };

function toNanos(text) {
  const m = String(text).match(/^([\d.]+)(ns|µs|us|ms|s)$/);
  return m ? Number(m[1]) * multiplier[m[2]] : null;
}

const samples = [];
for (let i = 0; i < runs; i += 1) {
  const out = execFileSync("cargo", ["bench", "--bench", "hot_path", "--", "--quick"], {
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  const m = out.match(/total=([^\s]+)/);
  const ns = m && toNanos(m[1]);
  if (!ns) throw new Error(`第 ${i + 1} 次采样未解析 total：${out.slice(-300)}`);
  samples.push({ raw: m[1], ns });
}

const values = samples.map((s) => s.ns).sort((a, b) => a - b);
const median = values[Math.floor(values.length / 2)];
const mean = values.reduce((a, b) => a + b, 0) / values.length;
const variance = values.reduce((a, v) => a + (v - mean) ** 2, 0) / values.length;
const cv = mean === 0 ? 0 : Math.sqrt(variance) / mean;

const baseline = {
  schema_version: 1,
  bench: "hot_path",
  command: "cargo bench --bench hot_path -- --quick",
  runs,
  environment: {
    os: `${platform()} ${release()}`,
    arch: arch(),
    cpu: cpus()[0]?.model || "unknown",
    cores: cpus().length,
  },
  samples,
  stats_ns: {
    min: values[0],
    median,
    max: values[values.length - 1],
    mean,
    coefficient_of_variation: Number(cv.toFixed(4)),
  },
  noise_tolerance: "观察阈值取中位数 × 1.5；仅当变异系数 < 0.2 且连续多轮稳定时才考虑升级为阻塞 Gate",
  threshold_policy: "本轮不阻塞 PR；单次数字不作为回归结论",
};

writeFileSync(
  "docs/kernel-prod-001/bench-baseline.json",
  JSON.stringify(baseline, null, 2) + "\n",
  "utf8",
);
console.log(`bench baseline: ${runs} 次，median=${median}ns，cv=${baseline.stats_ns.coefficient_of_variation}`);
