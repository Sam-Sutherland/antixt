import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { spawnSync } from "node:child_process";

const benchmarkDirectory = resolve(import.meta.dirname);
const workspace = resolve(benchmarkDirectory, "..");
const manifest = join(workspace, "antixt/Cargo.toml");
const cli = join(workspace, "antixt/target/release/antixt");
const projectName = "antixt-benchmark";
const project = join(workspace, ".apps", projectName);
const projectBuild = join(project, ".antixt");
const server = join(projectBuild, "target/release/antixt-app");
const generatedSource = join(projectBuild, "generated/main.rs");
const cliTarget = join(workspace, "antixt/target");
const pageSource = join(project, "app/page.rs");
const islandSource = join(project, "client/counter.js");

function run(command, expectedStatus = 0) {
  const started = performance.now();
  const result = spawnSync(command[0], command.slice(1), {
    cwd: workspace,
    encoding: "utf8",
  });
  const ms = performance.now() - started;
  if (result.status !== expectedStatus) {
    throw new Error(`${command.join(" ")} exited ${result.status}\n${result.stderr || result.stdout || result.error?.message || "unknown process error"}`);
  }
  return { ms, stdout: result.stdout };
}

function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}

function rounded(value) {
  return Math.round(value * 100) / 100;
}

function rustSources(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...rustSources(path));
    else if (entry.isFile() && entry.name.endsWith(".rs")) files.push(path);
  }
  return files;
}

rmSync(project, { recursive: true, force: true });
process.on("exit", () => rmSync(project, { recursive: true, force: true }));
rmSync(cliTarget, { recursive: true, force: true });
const cliBuildMs = run(["cargo", "build", "--release", "--manifest-path", manifest]).ms;
run([cli, "create", projectName]);
mkdirSync(join(project, "app/docs/[...path]"), { recursive: true });
writeFileSync(
  join(project, "app/docs/[...path]/page.rs"),
  `use antixt::{Context, Html, Value, html};

pub struct Params<'a> {
    pub path: Value<'a>,
}

pub fn page(_context: Context<'_>, params: Params<'_>) -> Html {
    html::p().text(params.path.decode().unwrap_or_default())
}
`,
);
mkdirSync(join(project, "app/stream"), { recursive: true });
writeFileSync(
  join(project, "app/stream/get.rs"),
  `use antixt::{Context, Response};

pub fn get(_context: Context<'_>) -> Response {
    Response::stream(
        "text/html; charset=utf-8",
        ["<p>first</p>", "<p>second</p>"],
    )
}
`,
);
const cliStartupMs = median(Array.from({ length: 7 }, () => run([cli, "version"]).ms));
const routeScanMs = median(Array.from({ length: 7 }, () => run([cli, "routes", project]).ms));

rmSync(join(projectBuild, "target"), { recursive: true, force: true });
const coldCheckMs = run([cli, "check", project]).ms;
const warmCheckMs = median(Array.from({ length: 7 }, () => run([cli, "check", project]).ms));

rmSync(join(projectBuild, "target"), { recursive: true, force: true });
const coldBuildMs = run([cli, "build", project]).ms;
const rebuildMs = run([cli, "build", project]).ms;

const originalPage = readFileSync(pageSource, "utf8");
let incrementalBuildMs;
try {
  writeFileSync(pageSource, `${originalPage}\n`);
  incrementalBuildMs = run([cli, "build", project]).ms;
} finally {
  writeFileSync(pageSource, originalPage);
}

const pages = {
  "GET /": run([server, "--render", "/"]).stdout,
  "GET /about": run([server, "--render", "/about"]).stdout,
  "GET /blog/:slug": run([server, "--render", "/blog/benchmark-route"]).stdout,
  "GET /docs/*path": run([server, "--render", "/docs/bench/catch-all"]).stdout,
  "GET /api/status": run([server, "--render", "/api/status?name=Benchmark"]).stdout,
  "GET /stream": run([server, "--render", "/stream"]).stdout,
  "POST /newsletter": run([server, "--render", "/newsletter", "POST"]).stdout,
};
if (!pages["GET /"].includes("Fast &lt; simple &amp; safe")) {
  throw new Error("typed HTML escaping verification failed");
}
if (!pages["GET /about"].includes("about-shell")) {
  throw new Error("nested Rust layout verification failed");
}
if (!pages["GET /blog/:slug"].includes("benchmark-route")) {
  throw new Error("typed dynamic route verification failed");
}
if (!pages["GET /docs/*path"].includes("bench/catch-all")) {
  throw new Error("catch-all route verification failed");
}
if (!pages["GET /api/status"].includes("Hello, Benchmark")) {
  throw new Error("async query route verification failed");
}
if (!pages["GET /stream"].includes("first</p><p>second")) {
  throw new Error("streaming route verification failed");
}
if (pages["GET /"].includes("data-antixt-dev")) {
  throw new Error("production output contains development JavaScript");
}
if (!pages["GET /"].includes("data-antixt-client")) {
  throw new Error("enhanced production page is missing opt-in client runtime");
}
if (pages["GET /about"].includes("<script")) {
  throw new Error("ordinary production route unexpectedly contains JavaScript");
}
run([server, "--render", "/missing"], 2);

const renderMs = median(Array.from({ length: 7 }, () => run([server, "--render", "/"]).ms));
const sourceFiles = rustSources(join(project, "app")).concat(rustSources(join(project, "components")));
const result = {
  version: "0.4.0-full-stack",
  routes: Object.keys(pages).length,
  rustSourceFiles: sourceFiles.length,
  rustSourceBytes: sourceFiles.reduce((total, file) => total + statSync(file).size, 0),
  cliBuildMs: Math.round(cliBuildMs),
  cliStartupMs: rounded(cliStartupMs),
  routeScanMs: rounded(routeScanMs),
  coldCheckMs: Math.round(coldCheckMs),
  warmCheckMs: rounded(warmCheckMs),
  coldBuildMs: Math.round(coldBuildMs),
  rebuildMs: Math.round(rebuildMs),
  incrementalBuildMs: Math.round(incrementalBuildMs),
  renderMs: rounded(renderMs),
  cliBytes: statSync(cli).size,
  serverBytes: statSync(server).size,
  generatedSourceBytes: statSync(generatedSource).size,
  enhancedPageInlineJavaScriptBytes: Buffer.byteLength(
    pages["GET /"].match(/<script data-antixt-client>([\s\S]*?)<\/script>/)?.[1] || "",
  ),
  embeddedIslandJavaScriptBytes: statSync(islandSource).size,
  zeroJavaScriptProductionRoute: true,
  outputSha256: createHash("sha256").update(Object.values(pages).join("\n")).digest("hex"),
};
const previous = JSON.parse(
  readFileSync(join(benchmarkDirectory, "antixt-rust-v02-results.json"), "utf8"),
).result;
const comparison = {
  previousVersion: previous.version,
  previousBuildMs: previous.coldBuildMs,
  previousRebuildMs: previous.rebuildMs,
  previousRenderMs: previous.renderMs,
  buildDeltaMs: result.coldBuildMs - previous.coldBuildMs,
  rebuildDeltaMs: result.rebuildMs - previous.rebuildMs,
  renderDeltaMs: rounded(result.renderMs - previous.renderMs),
};

writeFileSync(
  join(benchmarkDirectory, "antixt-rust-results.json"),
  `${JSON.stringify({ generatedAt: new Date().toISOString(), result, comparison }, null, 2)}\n`,
);
console.table([{ ...result, outputSha256: result.outputSha256.slice(0, 12) }]);
console.table([comparison]);
run([cli, "destroy", projectName, "--force"]);
