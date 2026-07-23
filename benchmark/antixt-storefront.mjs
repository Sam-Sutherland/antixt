import { spawn, spawnSync } from "node:child_process";
import {
  mkdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import http from "node:http";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";

const benchmarkDirectory = resolve(import.meta.dirname);
const workspace = resolve(benchmarkDirectory, "..");
const app = join(workspace, ".apps/antixt-storefront");
const verifyOnly = process.argv.includes("--verify");
const profile = verifyOnly ? "debug" : "release";
const cli = join(workspace, `antixt/target/${profile}/antixt`);
const server = join(app, ".antixt/target/release/antixt-app");
const port = 44000 + (process.pid % 1000);

function run(command, cwd = workspace) {
  const started = performance.now();
  const result = spawnSync(command[0], command.slice(1), {
    cwd,
    encoding: "utf8",
  });
  const ms = performance.now() - started;
  if (result.status !== 0) {
    throw new Error(
      `${command.join(" ")} exited ${result.status}\n${result.stderr || result.stdout || result.error?.message || "unknown process error"}`,
    );
  }
  return { ms, stdout: result.stdout };
}

function write(relative, contents) {
  const target = join(app, relative);
  mkdirSync(resolve(target, ".."), { recursive: true });
  writeFileSync(target, contents);
}

function createFixture() {
  rmSync(app, { recursive: true, force: true });
  write(
    "Cargo.toml",
    `[package]\nname = "antixt-storefront"\nversion = "0.4.0"\nedition = "2024"\npublish = false\n\n[[bin]]\nname = "antixt-app"\npath = ".antixt/generated/main.rs"\n\n[dependencies]\nantixt = { path = "../../antixt" }\n\n[package.metadata.antixt]\ngenerated = true\n`,
  );
  write("components/mod.rs", "pub mod catalog;\npub mod state;\npub mod ui;\n");
  write(
    "components/state.rs",
    `use antixt::{RequestFinished, RequestLifecycle};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct StoreConfig {
    pub name: &'static str,
    pub currency: &'static str,
}

#[derive(Default)]
pub struct RequestMetrics {
    completed: AtomicU64,
}

impl RequestMetrics {
    pub fn completed(&self) -> u64 { self.completed.load(Ordering::Relaxed) }
}

impl RequestLifecycle for RequestMetrics {
    fn finished(&self, _request: &RequestFinished<'_>) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }
}
`,
  );
  write(
    "components/catalog.rs",
    `use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct Product {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub price: u32,
}

pub struct Catalog {
    products: Vec<Product>,
    loads: AtomicU64,
}

impl Catalog {
    pub fn seeded() -> Self {
        let products = (1..=12)
            .map(|index| Product {
                slug: format!("product-{index}"),
                name: format!("Native product {index}"),
                description: format!("A server-rendered product description for item {index}."),
                price: 1200 + index * 175,
            })
            .collect();
        Self { products, loads: AtomicU64::new(0) }
    }

    pub fn featured(&self) -> Vec<Product> {
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.products.clone()
    }

    pub fn product(&self, slug: &str) -> Option<Product> {
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.products.iter().find(|product| product.slug == slug).cloned()
    }

    pub fn loads(&self) -> u64 { self.loads.load(Ordering::Relaxed) }
}
`,
  );
  write(
    "components/ui.rs",
    `use crate::components::catalog::Product;
use antixt::{Html, html, view};

pub fn product_card(product: &Product, currency: &str) -> Html {
    view! {
        article [class = "product-card"] {
            h2 { (product.name.as_str()) }
            p { (product.description.as_str()) }
            strong { (format!("{currency}{:.2}", product.price as f64 / 100.0)) }
            (html::a().attr("href", format!("/products/{}", product.slug)).text("View product"))
        }
    }
}
`,
  );
  write(
    "app/config.rs",
    `use crate::components::catalog::Catalog;
use crate::components::state::{RequestMetrics, StoreConfig};
use antixt::{Application, StartupError};
use std::sync::Arc;

pub fn configure(application: &mut Application) -> Result<(), StartupError> {
    application.state(StoreConfig { name: "antixt supply", currency: "£" })?;
    application.state(Catalog::seeded())?;
    let metrics = Arc::new(RequestMetrics::default());
    application.state(Arc::clone(&metrics))?;
    application.lifecycle(metrics);
    Ok(())
}
`,
  );
  write(
    "app/layout.rs",
    `use crate::components::catalog::Catalog;
use crate::components::state::{RequestMetrics, StoreConfig};
use antixt::{Context, Html, html, view};
use std::sync::Arc;

pub fn layout(context: Context<'_>, children: Html) -> Html {
    let config = context.state::<StoreConfig>().expect("StoreConfig is configured");
    let catalog = context.state::<Catalog>().expect("Catalog is configured");
    let featured = context
        .memoize_sync("featured-products", || catalog.featured())
        .expect("request is active");
    let completed = context
        .state::<Arc<RequestMetrics>>()
        .expect("RequestMetrics is configured")
        .completed();
    let body = view! {
        body {
            header {
                (html::a().attr("href", "/").text(config.name))
                nav { (html::a().attr("href", "/products/product-1").text("Products")) }
            }
            (children)
            footer { "Rendered by antixt" }
        }
    }
    .attr("data-featured-count", featured.len())
    .attr("data-catalog-loads", catalog.loads())
    .attr("data-completed-requests", completed);
    view! {
        document [lang = "en"] {
            head {
                meta [charset = "utf-8"] {}
                meta [name = "viewport", content = "width=device-width, initial-scale=1"] {}
                title { (config.name) }
            }
            (body)
        }
    }
}
`,
  );
  write(
    "app/page.rs",
    `use crate::components::catalog::Catalog;
use crate::components::state::StoreConfig;
use crate::components::ui::product_card;
use antixt::{Context, Html, view};

pub fn page(context: Context<'_>) -> Html {
    let config = context.state::<StoreConfig>().expect("StoreConfig is configured");
    let catalog = context.state::<Catalog>().expect("Catalog is configured");
    let featured = context
        .memoize_sync("featured-products", || catalog.featured())
        .expect("request is active");
    let cards: Html = featured
        .iter()
        .map(|product| product_card(product, config.currency))
        .collect();
    view! {
        main {
            h1 { "Featured products" }
            p { "A realistic server-rendered catalog with shared typed data." }
            section [aria_label = "Product catalog"] { (cards) }
        }
    }
}
`,
  );
  write(
    "app/products/[slug]/page.rs",
    `use crate::components::catalog::Catalog;
use crate::components::state::StoreConfig;
use crate::components::ui::product_card;
use antixt::{Context, Html, Value, html};

pub struct Params<'a> { pub slug: Value<'a> }

pub fn page(context: Context<'_>, params: Params<'_>) -> Html {
    let config = context.state::<StoreConfig>().expect("StoreConfig is configured");
    let catalog = context.state::<Catalog>().expect("Catalog is configured");
    let slug = params.slug.decode().unwrap_or_default().into_owned();
    let product = context
        .memoize_sync(("product", slug.clone()), || catalog.product(&slug))
        .expect("request is active");
    match product.as_ref() {
        Some(product) => product_card(product, config.currency),
        None => html::main().child(html::h1().text("Product not found")),
    }
}
`,
  );
  write(
    "app/api/featured/get.rs",
    `use crate::components::catalog::Catalog;
use antixt::{Context, Response};

pub fn get(context: Context<'_>) -> Response {
    let catalog = context.state::<Catalog>().expect("Catalog is configured");
    let featured = context
        .memoize_sync("featured-products", || catalog.featured())
        .expect("request is active");
    Response::text(format!("{} featured products", featured.len()))
}
`,
  );
}

function verifyOutput(output) {
  const checks = [
    "Featured products",
    'data-featured-count="12"',
    'data-catalog-loads="1"',
    "Native product 12",
    "Rendered by antixt",
  ];
  for (const expected of checks) {
    if (!output.includes(expected)) throw new Error(`storefront output is missing ${expected}`);
  }
  if (output.includes("data-antixt-client")) {
    throw new Error("ordinary storefront unexpectedly shipped the client runtime");
  }
}

function request(path = "/") {
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

function percentile(values, fraction) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * fraction))];
}

async function load(total, concurrency) {
  let cursor = 0;
  const latencies = [];
  const started = performance.now();
  async function worker() {
    while (cursor < total) {
      const index = cursor++;
      const path = index % 4 === 0 ? `/products/product-${(index % 12) + 1}` : "/";
      const response = await request(path);
      if (response.status !== 200 || !response.body.includes("antixt supply")) {
        throw new Error(`invalid storefront response for request ${index}`);
      }
      latencies.push(response.ms);
    }
  }
  await Promise.all(Array.from({ length: concurrency }, worker));
  const elapsed = performance.now() - started;
  return {
    requests: total,
    concurrency,
    requestsPerSecond: Math.round((total * 100000) / elapsed) / 100,
    p50Ms: Math.round(percentile(latencies, 0.5) * 100) / 100,
    p95Ms: Math.round(percentile(latencies, 0.95) * 100) / 100,
    p99Ms: Math.round(percentile(latencies, 0.99) * 100) / 100,
  };
}

let child;
let childOutput = "";
try {
  createFixture();
  run(["cargo", "build", "--manifest-path", "antixt/Cargo.toml", ...(verifyOnly ? [] : ["--release"])]);
  const check = run([cli, "check", app]);
  const firstBuild = run([cli, "build", app]);
  const secondBuild = run([cli, "build", app]);
  const render = run([server, "--render", "/"]);
  verifyOutput(render.stdout);
  const renderSamples = [render.ms];
  if (!verifyOnly) {
    for (let sample = 1; sample < 15; sample += 1) {
      renderSamples.push(run([server, "--render", "/"]).ms);
    }
  }

  if (verifyOnly) {
    console.log("Verified typed state, shared request memoization, lifecycle configuration, and zero-JS storefront output.");
  } else {
    child = spawn(server, [], {
      cwd: workspace,
      env: { ...process.env, PORT: String(port) },
      stdio: ["ignore", "pipe", "pipe"],
    });
    child.stdout.on("data", (chunk) => (childOutput += chunk));
    child.stderr.on("data", (chunk) => (childOutput += chunk));
    const waiting = performance.now();
    while (performance.now() - waiting < 10_000) {
      try {
        const response = await request("/");
        if (response.status === 200) break;
      } catch {}
      await new Promise((resolveWait) => setTimeout(resolveWait, 20));
    }
    const loadResult = await load(2000, 50);
    const result = {
      version: "0.4.0-storefront",
      routes: 3,
      products: 12,
      checkMs: Math.round(check.ms),
      firstBuildMs: Math.round(firstBuild.ms),
      noChangeBuildMs: Math.round(secondBuild.ms),
      renderMs: Math.round(percentile(renderSamples, 0.5) * 100) / 100,
      outputBytes: Buffer.byteLength(render.stdout),
      serverBytes: statSync(server).size,
      zeroJavascript: !render.stdout.includes("data-antixt-client"),
      ...loadResult,
    };
    writeFileSync(
      join(benchmarkDirectory, "antixt-storefront-results.json"),
      `${JSON.stringify({ generatedAt: new Date().toISOString(), result }, null, 2)}\n`,
    );
    console.table([result]);
  }
} finally {
  if (child) child.kill("SIGTERM");
  rmSync(app, { recursive: true, force: true });
}
