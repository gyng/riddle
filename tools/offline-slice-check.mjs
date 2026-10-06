#!/usr/bin/env node
// Exact saved-state diagnostic for transport partitions of ONE absence.
// Separate real check-ins are deliberately not compared here.
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sha = (value) => createHash("sha256").update(value).digest("hex");
const canonical = (value) => JSON.stringify(value, (_key, v) =>
  v && typeof v === "object" && !Array.isArray(v)
    ? Object.fromEntries(Object.keys(v).sort().map((k) => [k, v[k]])) : v);

function differences(a, b, path = "", out = []) {
  if (canonical(a) === canonical(b)) return out;
  if (a && b && typeof a === "object" && typeof b === "object"
      && Array.isArray(a) === Array.isArray(b)) {
    for (const k of [...new Set([...Object.keys(a), ...Object.keys(b)])].sort()) {
      differences(a[k], b[k], `${path}/${k.replaceAll("~", "~0").replaceAll("/", "~1")}`, out);
    }
  } else out.push({ path, expected: a ?? null, actual: b ?? null });
  return out;
}

async function main(args) {
  if (args.includes("--help")) {
    console.log("usage: node tools/offline-slice-check.mjs SAVE --seconds N --out NEW_DIR\nCompares complete saved states for whole/30min/uneven/reloaded slices. Exit1 means mismatch; diagnostic only.");
    return;
  }
  let save, seconds, out;
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--seconds") seconds = Number(args[++i]);
    else if (args[i] === "--out") out = args[++i];
    else if (args[i].startsWith("-") || save) throw new Error(`unexpected argument ${args[i]}`);
    else save = args[i];
  }
  if (!save || !out || !Number.isSafeInteger(seconds) || seconds < 0) {
    throw new Error("supply SAVE, --seconds (nonnegative integer), and --out NEW_DIR");
  }
  const source = readFileSync(resolve(save), "utf8");
  const wasm = readFileSync(join(ROOT, "web/src/engine/pkg/riddle_wasm_bg.wasm"));
  const { default: init, Game } = await import(pathToFileURL(join(ROOT, "web/src/engine/pkg/riddle_wasm.js")));
  await init({ module_or_path: wasm });
  // Refuse to overwrite earlier evidence or a user's directory.
  out = resolve(out); mkdirSync(out);
  const cases = [];
  let reference, uninterrupted;
  for (const spec of [
    { id: "whole", widths: [Math.max(1, seconds)] },
    { id: "30min", widths: [1800] },
    { id: "uneven", widths: [1, 1799, 3601, 719] },
    { id: "reload", widths: [1800], reload: true },
    { id: "complete-api", widths: [Math.max(1, seconds)], complete: true },
  ]) {
    let game = new Game(1);
    const reports = [];
    try {
      game.load(source);
      let left = seconds, calls = 0;
      do {
        const n = Math.min(left, spec.widths[calls % spec.widths.length]);
        const last = left === n;
        reports.push(JSON.parse(spec.complete ? game.runOfflineQuick(n) : game.runOfflineSlice(n, last)));
        left -= n; calls++;
        if (spec.reload && !last) {
          const checkpoint = game.save(); game.free(); game = new Game(1); game.load(checkpoint);
        }
      } while (left > 0);
      const state = JSON.parse(game.save());
      reference ??= state;
      const diff = differences(reference, state);
      if (spec.id === "30min") uninterrupted = state;
      const reloadDiff = spec.reload ? differences(uninterrupted, state) : undefined;
      const file = `${spec.id}.json`;
      writeFileSync(join(out, file), JSON.stringify({ state, reports, differences: diff, reloadDifferences: reloadDiff }));
      const row = { id: spec.id, calls, file, stateSha256: sha(canonical(state)),
        mismatches: diff.length, firstPaths: diff.slice(0, 12).map((d) => d.path),
        ...(reloadDiff ? { reloadMismatches: reloadDiff.length, reloadPaths: reloadDiff.slice(0, 12).map((d) => d.path) } : {}),
        reportedSeconds: reports.reduce((n, r) => n + r.elapsed_s, 0),
        reportedRuns: reports.reduce((n, r) => n + r.runs, 0) };
      cases.push(row); console.log(JSON.stringify(row));
    } finally { game.free(); }
  }
  const pass = cases.every((c) => c.mismatches === 0 && c.reportedSeconds === seconds);
  writeFileSync(join(out, "manifest.json"), JSON.stringify({ version: 1, scope: "exact state parity; not a balance gate or timing claim",
    source: resolve(save), sourceSha256: sha(source), wasmSha256: sha(wasm), seconds, pass, cases }, null, 2));
  if (!pass) process.exitCode = 1;
}
main(process.argv.slice(2)).catch((e) => { console.error(e.message); process.exitCode = 2; });
