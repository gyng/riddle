#!/usr/bin/env node
// Fast feedback from saved camps; not an acceptance gate. Full output comparisons
// allow an optimization to be checked without replaying the camp's whole fortnight.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const MODES = ["offline", "packages", "wall", "clone", "setup"];
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
export function parseArgs(args) {
  const out = { saves: [], modes: ["offline"], repeat: 1, record: null, compare: null, help: false };
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--help") out.help = true;
    else if (arg === "--") { out.saves.push(...args.slice(i + 1)); break; }
    else if (["--mode", "--repeat", "--record", "--compare"].includes(arg)) {
      const value = args[++i];
      if (!value || value.startsWith("--")) throw new Error(`missing value for ${arg}`);
      if (arg === "--mode") out.modes = [...new Set(value.split(","))];
      else if (arg === "--repeat") out.repeat = Number(value);
      else out[arg.slice(2)] = value;
    } else if (arg.startsWith("-")) throw new Error(`unknown option ${arg}`);
    else out.saves.push(arg);
  }
  if (!Number.isSafeInteger(out.repeat) || out.repeat < 1 || out.repeat > 100) throw new Error("repeat must be an integer from 1 to 100");
  if (!out.modes.length || out.modes.some((m) => !MODES.includes(m))) throw new Error(`mode must be one or more of ${MODES.join(",")}`);
  if (out.record && out.compare) throw new Error("record and compare are mutually exclusive");
  if (!out.help && !out.saves.length) throw new Error("supply at least one saved camp");
  return out;
}

export function checkReference(manifest, inputs, modes) {
  if (manifest?.version !== 1 || !Array.isArray(manifest.inputs) || !Array.isArray(manifest.modes)
      || !Array.isArray(manifest.cases) || typeof manifest.binary !== "string") throw new Error("invalid reference manifest");
  if (JSON.stringify(manifest.inputs.map((x) => x.sha256)) !== JSON.stringify(inputs.map((x) => x.sha256))) throw new Error("reference camp inputs differ");
  if (JSON.stringify(manifest.modes) !== JSON.stringify(modes)) throw new Error("reference workloads differ");
  if (manifest.cases.length !== inputs.length * modes.length) throw new Error("incomplete reference");
  for (let i = 0; i < inputs.length; i++) for (let j = 0; j < modes.length; j++) {
    const c = manifest.cases[i * modes.length + j];
    if (c?.file !== `${i}-${modes[j]}.json` || typeof c.sha256 !== "string") throw new Error("invalid reference case");
  }
}

export function checkOutput(actual, expected, expectedHash) {
  if (sha(expected) !== expectedHash) throw new Error("reference output is corrupt");
  if (!actual.equals(expected)) throw new Error("simulation output changed");
}

function execute(cmd, args, options = {}) {
  const r = spawnSync(cmd, args, { cwd: ROOT, encoding: "utf8", maxBuffer: 32 * 1024 * 1024, ...options });
  if (r.error || r.status !== 0) throw new Error(`${cmd} failed: ${r.error?.message ?? r.signal ?? r.stderr ?? r.stdout}`);
  return r.stdout;
}

async function main(args) {
  const opt = parseArgs(args);
  if (opt.help) {
    console.log("usage: node tools/camp-check.mjs SAVE... [--mode offline,packages,wall,clone,setup] [--repeat N] [--record DIR | --compare DIR]\nEach workload loads a fresh camp. Diagnostic only; targeted rows and full acceptance remain required.");
    return;
  }
  const started = performance.now();
  const snapshots = opt.saves.map((path) => readFileSync(path));
  const inputs = opt.saves.map((path, i) => ({ path: resolve(path), sha256: sha(snapshots[i]) }));
  const record = opt.record && resolve(opt.record), reference = opt.compare && resolve(opt.compare);
  if (record && existsSync(record)) throw new Error("reference directory already exists; refusing to replace it");
  let previous = null;
  if (reference) {
    previous = JSON.parse(readFileSync(join(reference, "manifest.json"), "utf8"));
    checkReference(previous, inputs, opt.modes);
    // Reject corrupt references before spending time on any simulation.
    for (const c of previous.cases) if (sha(readFileSync(join(reference, c.file))) !== c.sha256) throw new Error("reference output is corrupt");
  }
  const buildStart = performance.now();
  const build = execute("cargo", ["build", "-q", "--profile", "fast", "-p", "riddle-core", "--example", "sim_perf", "--message-format=json"]);
  const executable = build.split("\n").filter(Boolean).map((s) => JSON.parse(s))
    .find((m) => m.reason === "compiler-artifact" && m.target.name === "sim_perf" && m.executable)?.executable;
  if (!executable) throw new Error("Cargo did not report the diagnostic executable");
  const buildSeconds = (performance.now() - buildStart) / 1000;
  const binary = sha(readFileSync(executable));
  const temporary = mkdtempSync(join(tmpdir(), "riddle-camp-check-"));
  const cases = [], outputs = new Map();
  try {
    for (let i = 0; i < inputs.length; i++) writeFileSync(join(temporary, `camp-${i}.json`), snapshots[i]);
    for (let i = 0; i < inputs.length; i++) for (const mode of opt.modes) {
      const file = `${i}-${mode}.json`, raw = join(temporary, file), samples = [], work = [];
      let first;
      for (let n = 0; n < opt.repeat; n++) {
        const measurement = JSON.parse(execute(executable, [join(temporary, `camp-${i}.json`), mode, raw]));
        if (!Number.isFinite(measurement.seconds) || measurement.seconds < 0) throw new Error("invalid diagnostic measurement");
        const body = readFileSync(raw);
        if (first && !first.equals(body)) throw new Error(`non-deterministic output: ${basename(inputs[i].path)} ${mode}`);
        first ??= body;
        samples.push(measurement.seconds);
        work.push(measurement.work);
        if (sha(readFileSync(inputs[i].path)) !== inputs[i].sha256) throw new Error("camp input changed during measurement");
      }
      if (reference) {
        const expected = previous.cases[cases.length];
        checkOutput(first, readFileSync(join(reference, expected.file)), expected.sha256);
      }
      const ordered = [...samples].sort((a, b) => a - b), middle = Math.floor(ordered.length / 2);
      const median = ordered.length % 2 ? ordered[middle] : (ordered[middle - 1] + ordered[middle]) / 2;
      cases.push({ file, sha256: sha(first), samples, work, medianSeconds: median });
      outputs.set(file, first);
      console.log(`${basename(inputs[i].path)} ${mode}: ${median.toFixed(3)}s${reference ? " · exact reference match" : ""}`);
    }
    if (record) {
      mkdirSync(dirname(record), { recursive: true });
      mkdirSync(record); // Exclusive: never overwrite a reference made by another invocation.
      for (const [file, bytes] of outputs) writeFileSync(join(record, file), bytes);
      writeFileSync(join(record, "manifest.json"), JSON.stringify({ version: 1, binary, inputs, modes: opt.modes, cases, buildSeconds }, null, 2) + "\n");
    }
    console.log(`camp-check: ${cases.length} workloads · build ${buildSeconds.toFixed(2)}s · total ${((performance.now() - started) / 1000).toFixed(2)}s · diagnostic only`);
  } finally { rmSync(temporary, { recursive: true, force: true }); }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((e) => { console.error(`camp-check: ${e.message}`); process.exitCode = 1; });
}
