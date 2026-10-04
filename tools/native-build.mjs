import { spawnSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { readFileSync, writeFileSync, mkdirSync, renameSync, existsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { writeGenerated, publishMetadata } from "./native-codegen.mjs";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
// Publishing client metadata triggers Vite reloads. Keep it paired with the
// successful executable, never with an unfinished or failed compilation.
const generated = writeGenerated(false, false);
const tuning = !!process.env.RIDDLE_BALANCE_FILE || process.argv.includes('--tuning');
const r = spawnSync("cargo", ["build", "-q", "--profile", "fast", "-p", "riddle-core", "--example", "native_dev", ...(tuning ? ["--features", "dev-balance"] : []), "--message-format=json"], { cwd: root, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
if (r.status !== 0) {
  // Cargo JSON puts the useful compiler diagnostics on stdout, not stderr.
  const diagnostics = (r.stdout ?? '').split('\n').flatMap(line => {
    try { const m = JSON.parse(line); return m.reason === 'compiler-message' && m.message?.rendered ? [m.message.rendered] : []; }
    catch { return []; }
  }).join('');
  process.stderr.write(diagnostics + (r.stderr || 'native build failed'));
  process.exit(1);
}
const artifact = r.stdout.split("\n").filter(Boolean).map((s) => JSON.parse(s)).find((m) => m.reason === "compiler-artifact" && m.target.name === "native_dev" && m.executable);
if (!artifact) throw new Error("missing native executable");
// Keep default/tuning servers isolated when Cargo switches features at its
// shared example path. Replace a stable per-mode executable atomically.
const dir = resolve(root, 'target/native-dev', tuning ? 'tuning' : 'default');
mkdirSync(dir, { recursive: true });
const binary = resolve(dir, 'native_dev'), bytes = readFileSync(artifact.executable);
const sha = b => createHash('sha256').update(b).digest('hex');
if (!existsSync(binary) || sha(readFileSync(binary)) !== sha(bytes)) {
  const temporary = `${binary}.tmp-${process.pid}`;
  writeFileSync(temporary, bytes, { mode: 0o755 }); renameSync(temporary, binary);
}
publishMetadata(generated.metadata);
console.log(binary);
