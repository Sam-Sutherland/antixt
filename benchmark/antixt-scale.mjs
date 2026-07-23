import { createHash } from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import {
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import http from "node:http";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";

const benchmarkDirectory = resolve(import.meta.dirname);
const workspace = resolve(benchmarkDirectory, "..");
const app = join(workspace, ".apps/antixt-scale");
const cli = join(workspace, "antixt/target/release/antixt");
const target = join(app, ".antixt/target");
const server = join(target, "release/antixt-app");
const leaf = join(app, "components/leaf.rs");
const port = 43000 + (process.pid % 1000);

function run(command, expectedStatus = 0) {
  const started = performance.now();
  const result = spawnSync(command[0], command.slice(1), {
    cwd: workspace,
    encoding: "utf8",
  });
  const ms = performance.now() - started;
  if (result.status !== expectedStatus) {
    throw new Error(
      `${command.join(" ")} exited ${result.status}\n${result.stderr || result.stdout || result.error?.message || "unknown process error"}`,
    );
  }
  return { ms, stdout: result.stdout };
}

function rounded(value) {
  return Math.round(value * 100) / 100;
}

function percentile(values, fraction) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * fraction))];
}

function request(path = "/route-0500") {
  const started = performance.now();
  return new Promise((resolveRequest, reject) => {
    const outgoing = http.get({ host: "127.0.0.1", port, path }, (response) => {
      let body = "";
      response.setEncoding("utf8");
      response.on("data", (chunk) => (body += chunk));
      response.on("end", () =>
        resolveRequest({ status: response.statusCode, body, ms: performance.now() - started }),
      );
    });
    outgoing.on("error", reject);
  });
}

async function waitForServer() {
  const started = performance.now();
  while (performance.now() - started < 10_000) {
    if (child?.exitCode !== null) {
      throw new Error(
        `scale server exited with ${child.exitCode}\n${childOutput || "no server output"}`,
      );
    }
    try {
      const response = await request("/");
      if (response.status === 200) return performance.now() - started;
    } catch {}
    await new Promise((resolveWait) => setTimeout(resolveWait, 20));
  }
  throw new Error(`scale server did not become ready\n${childOutput || "no server output"}`);
}

async function load(total, concurrency) {
  let cursor = 0;
  const latencies = [];
  const started = performance.now();
  async function worker() {
    while (cursor < total) {
      const index = cursor++;
      const response = await request(`/route-${String(index % 999).padStart(4, "0")}`);
      if (response.status !== 200 || !response.body.includes("shared leaf")) {
        throw new Error(`invalid scale response for request ${index}`);
      }
      latencies.push(response.ms);
    }
  }
  await Promise.all(Array.from({ length: concurrency }, worker));
  const elapsed = performance.now() - started;
  return {
    requests: total,
    concurrency,
    requestsPerSecond: rounded((total * 1000) / elapsed),
    p50Ms: rounded(percentile(latencies, 0.5)),
    p95Ms: rounded(percentile(latencies, 0.95)),
    p99Ms: rounded(percentile(latencies, 0.99)),
  };
}

function createFixture() {
  rmSync(app, { recursive: true, force: true });
  mkdirSync(join(app, "app"), { recursive: true });
  mkdirSync(join(app, "components"), { recursive: true });
  writeFileSync(
    join(app, "Cargo.toml"),
    `[package]\nname = "antixt-scale"\nversion = "0.4.0"\nedition = "2024"\npublish = false\n\n[[bin]]\nname = "antixt-app"\npath = ".antixt/generated/main.rs"\n\n[dependencies]\nantixt = { path = "../../antixt" }\n\n[package.metadata.antixt]\ngenerated = true\n`,
  );
  writeFileSync(join(app, "components/mod.rs"), "pub mod leaf;\n");
  writeFileSync(
    leaf,
    `use antixt::{Html, html};\n\npub fn leaf(index: usize) -> Html { html::p().text(format!("shared leaf {index}")) }\n`,
  );
  writeFileSync(
    join(app, "app/page.rs"),
    `use crate::components::leaf::leaf;\nuse antixt::{Context, Html};\n\npub fn page(_context: Context<'_>) -> Html { leaf(1000) }\n`,
  );
  for (let index = 0; index < 999; index += 1) {
    const directory = join(app, "app", `route-${String(index).padStart(4, "0")}`);
    mkdirSync(directory, { recursive: true });
    writeFileSync(
      join(directory, "page.rs"),
      `use crate::components::leaf::leaf;\nuse antixt::{Context, Html};\n\npub fn page(_context: Context<'_>) -> Html { leaf(${index}) }\n`,
    );
  }
}

let child;
let childOutput = "";
try {
  const fixtureStarted = performance.now();
  createFixture();
  const fixtureMs = performance.now() - fixtureStarted;
  run(["cargo", "build", "--release", "--manifest-path", "antixt/Cargo.toml"]);
  const scanMs = run([cli, "routes", app]).ms;

  rmSync(target, { recursive: true, force: true });
  const coldCheckMs = run([cli, "check", app]).ms;
  const warmCheckMs = run([cli, "check", app]).ms;
  rmSync(target, { recursive: true, force: true });
  const coldBuildMs = run([cli, "build", app]).ms;
  const noChangeBuildMs = run([cli, "build", app]).ms;

  const originalLeaf = readFileSync(leaf, "utf8");
  let leafEditBuildMs;
  try {
    writeFileSync(leaf, `${originalLeaf}\n`);
    leafEditBuildMs = run([cli, "build", app]).ms;
  } finally {
    writeFileSync(leaf, originalLeaf);
  }

  child = spawn(server, [], {
    cwd: workspace,
    env: { ...process.env, PORT: String(port) },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", (chunk) => (childOutput += chunk));
  child.stderr.on("data", (chunk) => (childOutput += chunk));
  child.on("error", (error) => (childOutput += `\n${error.stack || error.message}`));
  const startupMs = await waitForServer();
  const loadResult = await load(2000, 50);
  const rss = spawnSync("ps", ["-o", "rss=", "-p", String(child.pid)], {
    encoding: "utf8",
  });
  const rssBytes = Number.parseInt(rss.stdout.trim(), 10) * 1024;

  const result = {
    version: "0.4.0-scale",
    routes: 1000,
    fixtureMs: rounded(fixtureMs),
    routeScanMs: rounded(scanMs),
    coldCheckMs: Math.round(coldCheckMs),
    warmCheckMs: Math.round(warmCheckMs),
    coldBuildMs: Math.round(coldBuildMs),
    noChangeBuildMs: Math.round(noChangeBuildMs),
    sharedLeafEditBuildMs: Math.round(leafEditBuildMs),
    serverStartupMs: rounded(startupMs),
    serverBytes: statSync(server).size,
    generatedSourceBytes: statSync(join(app, ".antixt/generated/main.rs")).size,
    rssBytes,
    ...loadResult,
    outputSha256: createHash("sha256").update(readFileSync(leaf)).digest("hex"),
  };
  writeFileSync(
    join(benchmarkDirectory, "antixt-scale-results.json"),
    `${JSON.stringify({ generatedAt: new Date().toISOString(), result }, null, 2)}\n`,
  );
  console.table([result]);
} finally {
  if (child) child.kill("SIGTERM");
  rmSync(app, { recursive: true, force: true });
}
