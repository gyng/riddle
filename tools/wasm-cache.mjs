#!/usr/bin/env node
// Called after Cargo checks/builds the release wasm. Reuse packaging only when its actual
// compiled input, tools/recipe/metadata, environment and every output file still match.
import { createHash } from "node:crypto";
import { createReadStream, existsSync, lstatSync, readdirSync, readFileSync, renameSync, writeFileSync, accessSync, constants } from "node:fs";
import { homedir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const pkg = "web/src/engine/pkg", stamp = join(pkg, ".riddle-build.json");
const sha = (text) => createHash("sha256").update(text).digest("hex");
async function fileHash(path) {
  const hash = createHash("sha256");
  for await (const bytes of createReadStream(path)) hash.update(bytes);
  return hash.digest("hex");
}
function executable(name) {
  for (const dir of (process.env.PATH ?? "").split(":")) {
    const path = resolve(dir || ".", name);
    try { accessSync(path, constants.X_OK); return path; } catch { /* continue search */ }
  }
  throw new Error(`missing ${name}`);
}
async function inputKey() {
  const metadata = spawnSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], { encoding: "utf8" });
  if (metadata.status !== 0) throw new Error("cargo metadata failed");
  const raw = join(JSON.parse(metadata.stdout).target_directory, "wasm32-unknown-unknown/release/riddle_wasm.wasm");
  const paths = new Set([raw, executable("wasm-pack"), fileURLToPath(import.meta.url), "tools/wasm.sh", "Cargo.lock", "crates/riddle-wasm/Cargo.toml"]);
  for (const optional of ["crates/riddle-wasm/README.md", "crates/riddle-wasm/LICENSE", "crates/riddle-wasm/LICENSE-MIT", "crates/riddle-wasm/LICENSE-APACHE"]) if (existsSync(optional)) paths.add(optional);
  const cache = join(process.env.XDG_CACHE_HOME || join(homedir(), ".cache"), ".wasm-pack");
  if (existsSync(cache)) for (const dir of readdirSync(cache).sort()) {
    if (!dir.startsWith("wasm-bindgen-") && !dir.startsWith("wasm-opt-")) continue;
    for (const name of ["wasm-bindgen", "wasm-opt", "bin/wasm-bindgen", "bin/wasm-opt"]) {
      const path = join(cache, dir, name);
      if (existsSync(path)) paths.add(path);
    }
  }
  for (const name of ["wasm-bindgen", "wasm-opt"]) { try { paths.add(executable(name)); } catch { /* wasm-pack can supply them from its cache */ } }
  // Store only a digest of build-related environment values, never their contents.
  const environment = Object.entries(process.env).filter(([k]) => /^(CARGO_|RUST|WASM|BINARYEN|PATH$)/.test(k)).sort(([a], [b]) => a.localeCompare(b));
  const inputs = await Promise.all([...paths].sort().map(async (path) => [path, await fileHash(path)]));
  return sha(JSON.stringify({ recipe: "wasm-pack release web", inputs, environment: sha(JSON.stringify(environment)) }));
}
async function outputs() {
  if (lstatSync(pkg).isSymbolicLink()) throw new Error("pkg must be a regular directory");
  const walk = (dir) => readdirSync(dir).sort().flatMap((name) => {
    if (name.startsWith(".riddle-build.json")) return [];
    const path = join(dir, name), stat = lstatSync(path);
    if (stat.isSymbolicLink()) throw new Error("pkg must contain regular files");
    return stat.isDirectory() ? walk(path) : [path];
  });
  return Promise.all(walk(pkg).map(async (path) => [path, await fileHash(path)]));
}
try {
  const mode = process.argv[2];
  if (mode !== "check" && mode !== "record") throw new Error("usage: wasm-cache.mjs check|record");
  if (mode === "check" && !existsSync(stamp)) process.exit(1);
  const key = await inputKey();
  if (mode === "check") {
    const saved = JSON.parse(readFileSync(stamp, "utf8"));
    if (saved.key !== key || JSON.stringify(saved.outputs) !== JSON.stringify(await outputs())) process.exit(1);
    console.log("wasm: shipping pkg unchanged (compiled input and all output hashes checked)");
  } else {
    const files = await outputs();
    if (!files.some(([p]) => p.endsWith("riddle_wasm_bg.wasm")) || !files.some(([p]) => p.endsWith("riddle_wasm.js"))) throw new Error("incomplete wasm package");
    writeFileSync(`${stamp}.tmp`, JSON.stringify({ key, outputs: files }, null, 2) + "\n");
    renameSync(`${stamp}.tmp`, stamp);
  }
} catch (error) {
  if (process.argv[2] === "record") { console.error(`wasm cache: ${error.message}`); process.exit(1); }
  // A missing, corrupt or incompatible cache falls through to the real build.
  process.exit(1);
}
