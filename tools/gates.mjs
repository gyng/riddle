#!/usr/bin/env node
// Bot gate table (examples/metrics.rs) on the `fast` cargo profile.
//   node tools/gates.mjs          quick: 8 seeds × 8 h × 3 verdicts, dayplayer 2 seeds (1 leave-one-out) alongside (~30–40 min fresh on the Cut 30 tree)
//   node tools/gates.mjs --full   30 seeds × 8 h × 8 verdicts, dayplayer 16 seeds, the leave-one-outs too (~1–1.5 h fresh; docs/ITERATION_SPEED.md 0e); the number that counts
// Cut 13 §6: the wire invariants (examples/qa.rs, 30 seeds, ~700 thread-s: ~45 s alone on the cores, ~120 s beside
// the table) run beside both as a third job; the run fails if they do. `METRICS_PHASES=1` prints the table's and
// qa's phase and job walls (qa: thread-seconds per leg). `QA_SHARE=0.5` gives qa that share of the cores (0.75).
//   node tools/gates.mjs --fresh  ignore the caches. Each leg's printout is kept under target/gates/<leg>-<sha1>.txt, keyed by
//                                 its own binary (+ what it reads at run time: the presets and the cohort cards for the table)
//                                 — a client-only commit reprints all three in 0.2 s, and an edit to one example (a new qa
//                                 invariant, a metrics row) reruns that leg alone (docs/ITERATION_SPEED.md §3.3, round 3)
//   node tools/gates.mjs --fast   the gated rows that fit a few minutes: the quick table less the retired progression
//                                 lineages (`metrics --fast`), qa on 10 seeds, the dayplayer's IDLE rows on 2 seeds — an
//                                 inner-loop check, never the gate (docs/ITERATION_SPEED.md, round 4)
// Round 4: the long jobs inside the legs are kept too (target/gates/{dp,prog,idle-snaps}/), keyed by the core's
// compiled native core (`RIDDLE_SRC_KEY`: production rlib, manifests, lockfile, rustc) and the harness file each
// job runs — an edit to a bar or a row reprints the leg from its jobs, a core edit replays them. `--fresh` replays
// them all (and keeps the new ones). `GATES_THREADS=N` (the cores unless set): each leg's threads.
//   node tools/gates.mjs [--full] --rows <ids or substrings> [--fail-fast] [dayplayer args]
//                                 TARGETED, a tuning loop's check (docs/ITERATION_SPEED.md §0h): the dayplayer leg alone,
//                                 only the configurations, seeds and days the named rows read (`dayplayer --rows`);
//                                 `--fail-fast` aborts on the first row settled FAIL. Seeds as the tier's (quick 2, --full
//                                 16) unless `--seeds N` etc. follow. Cached like the legs; never a gate pass.
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, renameSync, writeFileSync } from "node:fs";
import os from "node:os";
import { runtimeKey } from "./runtime-key.mjs";
const full = process.argv.includes("--full");
const fast = process.argv.includes("--fast");
const fresh = process.argv.includes("--fresh");
const rowsAt = process.argv.indexOf("--rows");
const rows = rowsAt >= 0 ? process.argv[rowsAt + 1] : null;
const failFast = process.argv.includes("--fail-fast");
const extra = process.argv.slice(2).filter((a, i) => a !== "--full" && a !== "--fresh" && a !== "--fast" && a !== "--fail-fast" && !(rowsAt >= 0 && (i + 2 === rowsAt || i + 2 === rowsAt + 1)));
if (rowsAt >= 0 && !rows) { console.error("gates --rows: name the rows (ids or substrings; `target/fast/examples/dayplayer --rows ?` lists them)"); process.exit(2); }
const b = spawnSync("cargo", ["build", "-q", "--message-format=json", "--profile", "fast", "-p", "riddle-core", ...(rows ? [] : ["--example", "metrics", "--example", "qa"]), "--example", "dayplayer"], { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
if (b.stderr) process.stderr.write(b.stderr);
let coreArtifact;
for (const line of (b.stdout ?? "").split("\n").filter(Boolean)) {
  const message = JSON.parse(line);
  if (message.reason === "compiler-message" && message.message.rendered) process.stderr.write(message.message.rendered);
  if (message.reason === "compiler-artifact" && message.target.name === "riddle_core" && message.target.kind.includes("lib") && !message.profile.test) coreArtifact = message;
}
if (b.status !== 0) process.exit(b.status ?? 1);
// The binaries hash every input exactly (sources, deps, rustc); the presets and the cohort cards are read at run time
// (metrics.rs `cohort_sets`: every eval/cards/*.rules.json — a new card changes the table without changing a binary).
const keyOf = (bin, parts) => {
  const h = createHash("sha1");
  h.update(readFileSync(bin));
  for (const [name, text] of parts) h.update(name).update(text);
  return h.digest("hex").slice(0, 16);
};
// Cargo's production library contains the actual native code and dependency metadata.
// Test-only modules do not change it; runtime inputs, flags and compiler changes do.
// New scheme: existing job records are not renamed or accepted under the new key.
const compiler = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
if (compiler.status !== 0) throw new Error("cannot identify simulation compiler");
const srcKey = runtimeKey(coreArtifact, process.cwd(), compiler.stdout);
const jobEnv = { RIDDLE_SRC_KEY: srcKey, ...(fresh ? { RIDDLE_CACHE_FRESH: "1" } : {}) };
const readResult = (file) => {
  try { return JSON.parse(readFileSync(file, "utf8")); }
  catch { return null; }   // missing/corrupt cache is a miss, never evidence
};
const writeResult = (file, result) => {
  mkdirSync("target/gates", { recursive: true });
  const temporary = `${file}.tmp-${process.pid}`;
  writeFileSync(temporary, JSON.stringify({ status: result.status, stdout: result.stdout }));
  renameSync(temporary, file);
};
// TARGETED (`--rows`): the dayplayer leg alone on the named rows — its printout kept like a leg's (the binary, the
// arguments), its full-length jobs kept and read like the gate's (`RIDDLE_SRC_KEY`). Never a gate pass: it prints
// no `gates: all PASS` and says so.
if (rows) {
  const tier = full ? ["--seeds", "16", "--loo-seeds", "16", "--tuned-seeds", "16"] : ["--seeds", "2", "--loo-seeds", "1", "--tuned-seeds", "2"];
  // (the dayplayer reads an argument's first occurrence: `extra` — `--seeds 8` … — before the tier's)
  const args = [...extra, ...tier, "--rows", rows, ...(failFast ? ["--fail-fast"] : []), "--threads", String(Number(process.env.GATES_THREADS ?? os.availableParallelism()))];
  const leg = `target/gates/dayplayer-rows-${keyOf("target/fast/examples/dayplayer", [["args", args.filter((a, i) => args[i - 1] !== "--threads" && a !== "--threads").join(" ")], ["src", srcKey]])}.txt`;
  let res = !fresh ? readResult(leg) : null;
  const hitRows = !!res;
  if (!res) {
    res = await new Promise((resolve) => {
      const p = spawn("target/fast/examples/dayplayer", args, { stdio: ["ignore", "pipe", "inherit"], env: { ...process.env, ...jobEnv } });
      let out = ""; p.stdout.on("data", (d) => (out += d));
      p.on("close", (status) => resolve({ status, stdout: out }));
    });
    writeResult(leg, res);
  }
  const out = res.stdout ?? "";
  const from = out.lastIndexOf("\nfail-fast:") >= 0 ? out.lastIndexOf("\nfail-fast:") : out.lastIndexOf("\nbar (targeted");
  process.stdout.write(from >= 0 ? out.slice(from + 1) : out);
  const ok = res.status === 0 && /targeted rows PASS/.test(out);
  console.log(`gates: TARGETED — dayplayer rows \`${rows}\`${failFast ? " (fail-fast)" : ""} ${ok ? "PASS" : "FAIL"}; not a gate pass (the gate is \`node tools/gates.mjs [--full]\`)${hitRows ? ` (cached: ${leg}; --fresh to rerun)` : ""}`);
  process.exit(ok ? 0 : 1);
}
const runtimeInputs = [
  ...readdirSync("crates/riddle-core/presets").sort().map((f) => [f, readFileSync(`crates/riddle-core/presets/${f}`)]),
  ...readdirSync("eval/cards").filter((f) => f.endsWith(".rules.json")).sort().map((f) => [f, readFileSync(`eval/cards/${f}`)]),
];
// Cut 30 §6: the idle bots over 16 seeds on the full table (the owner, round 5: the ratio rows — PICKED vs
// IDLE, RANDOM — are noisy at 8); TUNED and the leave-one-outs over 8 until the gate speed-up lands (the
// owner, round 6), then 16 — the speed-up landed (gate-speed, 2026-10-02): 16. 2 seeds and 1 leave-one-out on the quick table.
const seeds = full ? 16 : 2;
const looSeeds = full ? 16 : 1;
const tunedSeeds = full ? 16 : 2;
const dpArgs = ["--gate", "--seeds", String(seeds), ...(fast ? ["--bots", "idle", "--loo-seeds", "0"] : ["--loo-seeds", String(looSeeds), "--tuned-seeds", String(tunedSeeds)])];
const qaSeeds = fast ? 10 : 30;
const tableArgs = [...(full ? [] : [fast ? "--fast" : "--quick"]), ...extra];
const legs = {
  metrics: `target/gates/metrics-${keyOf("target/fast/examples/metrics", [...runtimeInputs, ["args", JSON.stringify({ full, fast, extra })]])}.txt`,
  qa: `target/gates/qa-${keyOf("target/fast/examples/qa", [["args", `--seeds ${qaSeeds}`]])}.txt`,
  dayplayer: `target/gates/dayplayer-${keyOf("target/fast/examples/dayplayer", [["args", dpArgs.join(" ")]])}.txt`,
};
const cached = (leg) => {
  return fresh || extra.length ? null : readResult(legs[leg]);
};
const keep = (leg, r) => {
  if (extra.length) return;
  writeResult(legs[leg], r);
};
const pendingLegs = new Map();
// Keep each completed leg immediately: a long dayplayer or an interrupted tuning session
// must not discard the already completed table and wire checks. Keys and checks are unchanged.
const finishLeg = (leg, promise) => {
  const started = Date.now();
  if (!hit[leg]) pendingLegs.set(leg, started);
  return promise.then((result) => {
    if (!hit[leg]) keep(leg, result);
    pendingLegs.delete(leg);
    console.error(`gates: ${leg} ${hit[leg] ? "cached" : `finished in ${((Date.now() - started) / 1000).toFixed(1)}s`} (exit ${result.status})`);
    return result;
  });
};
// The table fills every core seed by seed for ~40 s; the fourteen-day probe is a few long
// sequential chains (one per seed) that would otherwise run alone afterwards — they overlap.
// `signal`: a stderr line that resolves `started` (forwarded stderr otherwise untouched).
const run = (bin, args, signal, env) => {
  let started;
  const ready = new Promise((r) => (started = r));
  const done = new Promise((resolve) => {
    const p = spawn(bin, args, { stdio: ["ignore", "pipe", signal ? "pipe" : "inherit"], env: { ...process.env, ...jobEnv, ...(env ?? {}) } });
    let out = ""; p.stdout.on("data", (d) => (out += d));
    let buf = "";
    if (signal) {
      p.stderr.on("data", (d) => {
        buf += d;
        let i;
        while ((i = buf.indexOf("\n")) >= 0) {
          const line = buf.slice(0, i + 1); buf = buf.slice(i + 1);
          if (line.startsWith(signal)) started(); else process.stderr.write(line);
        }
      });
    }
    p.on("close", (status) => { if (signal && buf) process.stderr.write(buf); started(); resolve({ status, stdout: out }); });
  });
  return { ready, done };
};
// Every leg is CPU-bound and they run side by side, each on the whole budget (the cores share them out).
// Round 4: the dayplayer's fortnights are the gate's longest chains (its PICKED/TUNED groups, hours of CPU
// each): they start first, and its groups, like the table's chains (the progression lineages, the idle
// snapshots), widen their camp panels onto the threads the other jobs leave as they end — the tails run
// on every core instead of one (docs/ITERATION_SPEED.md, round 4).
const QA_SHARE = Number(process.env.QA_SHARE ?? 0.75);
const cores = Number(process.env.GATES_THREADS ?? os.availableParallelism());
const hit = { metrics: cached("metrics"), qa: cached("qa"), dayplayer: cached("dayplayer") };
const done = (r) => ({ ready: Promise.resolve(), done: Promise.resolve(r) });
const dpThreads = Math.max(4, cores);
const dayplayer = (hit.dayplayer ? done(hit.dayplayer) : run("target/fast/examples/dayplayer", [...dpArgs, "--threads", String(dpThreads)])).done;
const table = hit.metrics ? done(hit.metrics) : run("target/fast/examples/metrics", [...tableArgs, "--threads", String(Math.max(4, cores - 1))], "metrics: quiet ticks measured", { METRICS_QUIET_SIGNAL: "1" });
// The invariants (a job pool of seeds and their legs) start once the table's single-threaded quiet
// per-tick measurement is done (a few seconds), then take three quarters of the cores beside the
// table's all-but-one: at four threads they were the gate's critical path (305 s beside a 200 s
// table); at 24 they end in ~115 s and hand the cores back to the table, which is the critical path
// again (docs/ITERATION_SPEED.md, round 3).
// (a cached table leaves qa every core)
const qaThreads = hit.metrics ? cores : Math.max(4, Math.round(cores * QA_SHARE));
const qa = hit.qa ? Promise.resolve(hit.qa) : table.ready.then(() => run("target/fast/examples/qa", ["--seeds", String(qaSeeds), "--threads", String(qaThreads)]).done);
console.error(`gates: source ${srcKey}; threads metrics=${Math.max(4, cores - 1)} qa=${qaThreads} dayplayer=${dpThreads}; cache hits=${Object.entries(hit).filter(([, v]) => v).map(([k]) => k).join(",") || "none"}`);
const progress = setInterval(() => {
  if (pendingLegs.size) console.error(`gates: still running ${[...pendingLegs].map(([leg, start]) => `${leg} ${Math.round((Date.now() - start) / 1000)}s`).join(", ")}; completed legs already saved`);
}, 30000);
progress.unref();
const [r, p, q] = await Promise.all([finishLeg("metrics", table.done), finishLeg("dayplayer", dayplayer), finishLeg("qa", qa)]);
clearInterval(progress);
const say = (t) => process.stdout.write(t);
say(r.stdout ?? "");
if (r.status !== 0 || !/gates: all PASS/.test(r.stdout ?? "")) { console.error("gates: FAIL"); process.exit(1); }
say(q.stdout ?? "");
if (q.status !== 0 || !/qa: all PASS/.test(q.stdout ?? "")) { console.error("qa invariants: FAIL"); process.exit(1); }
// Fourteen-day pacing probe. Cut 29 §1: every bar is hard — the "informational until M7" carve-out for
// the unlock days (≥ 10/14) and the stall (≤ 3 d) is gone (the descent is 34 floors; they fail for the
// reasons docs/PROGRESSION.md measures, not for missing content).
const out = p.stdout ?? ""; say(out.slice(out.lastIndexOf("bar ")));
const hardFails = [...out.matchAll(/^[^\n]*\bFAIL$/gm)].map((m) => m[0]).filter((l) => !l.startsWith("dayplayer:"));
if (hardFails.length || p.status !== 0 || !/dayplayer: all PASS/.test(out)) { console.error("dayplayer bars: FAIL\n" + hardFails.join("\n")); process.exit(1); }
say("dayplayer: all bars pass\n");
const legsCached = Object.entries(hit).filter(([, v]) => v).map(([k]) => k);
if (legsCached.length) console.log(`gates: ${legsCached.join(", ")} cached (${legsCached.map((k) => legs[k]).join(" ")}; --fresh to rerun)`);
