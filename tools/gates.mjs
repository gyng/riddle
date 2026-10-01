#!/usr/bin/env node
// Bot gate table (examples/metrics.rs) on the `fast` cargo profile.
//   node tools/gates.mjs          quick: 8 seeds × 8 h × 3 verdicts, dayplayer 2 seeds alongside (~4 min: the per-set rows' edits and lanes do not scale with seeds)
//   node tools/gates.mjs --full   30 seeds × 8 h × 8 verdicts, dayplayer 3 seeds (~8 min on a shared box; docs/ITERATION_SPEED.md 0d); the number that counts
// Cut 13 §6: the wire invariants (examples/qa.rs, 30 seeds, ~700 thread-s: ~45 s alone on the cores, ~120 s beside
// the table) run beside both as a third job; the run fails if they do. `METRICS_PHASES=1` prints the table's and
// qa's phase and job walls (qa: thread-seconds per leg). `QA_SHARE=0.5` gives qa that share of the cores (0.75).
//   node tools/gates.mjs --fresh  ignore the caches. Each leg's printout is kept under target/gates/<leg>-<sha1>.txt, keyed by
//                                 its own binary (+ what it reads at run time: the presets and the cohort cards for the table)
//                                 — a client-only commit reprints all three in 0.2 s, and an edit to one example (a new qa
//                                 invariant, a metrics row) reruns that leg alone (docs/ITERATION_SPEED.md §3.3, round 3)
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import os from "node:os";
const full = process.argv.includes("--full");
const fresh = process.argv.includes("--fresh");
const extra = process.argv.slice(2).filter((a) => a !== "--full" && a !== "--fresh");
const b = spawnSync("cargo", ["build", "-q", "--profile", "fast", "-p", "riddle-core", "--example", "metrics", "--example", "dayplayer", "--example", "qa"], { stdio: "inherit" });
if (b.status !== 0) process.exit(b.status ?? 1);
// The binaries hash every input exactly (sources, deps, rustc); the presets and the cohort cards are read at run time
// (metrics.rs `cohort_sets`: every eval/cards/*.rules.json — a new card changes the table without changing a binary).
const keyOf = (bin, parts) => {
  const h = createHash("sha1");
  h.update(readFileSync(bin));
  for (const [name, text] of parts) h.update(name).update(text);
  return h.digest("hex").slice(0, 16);
};
const runtimeInputs = [
  ...readdirSync("crates/riddle-core/presets").sort().map((f) => [f, readFileSync(`crates/riddle-core/presets/${f}`)]),
  ...readdirSync("eval/cards").filter((f) => f.endsWith(".rules.json")).sort().map((f) => [f, readFileSync(`eval/cards/${f}`)]),
];
// Cut 30 §6: the idle bots over 8 seeds (full; the leave-one-outs over 4 — `--loo-seeds`), 2 seeds and 1 on the quick table
const seeds = full ? 8 : 2;
const looSeeds = full ? 4 : 1;
const legs = {
  metrics: `target/gates/metrics-${keyOf("target/fast/examples/metrics", [...runtimeInputs, ["args", JSON.stringify({ full, extra })]])}.txt`,
  qa: `target/gates/qa-${keyOf("target/fast/examples/qa", [["args", "--seeds 30"]])}.txt`,
  dayplayer: `target/gates/dayplayer-${keyOf("target/fast/examples/dayplayer", [["args", `--gate --seeds ${seeds} --loo-seeds ${looSeeds} --tuned-seeds ${seeds}`]])}.txt`,
};
const cached = (leg) => (!fresh && !extra.length && existsSync(legs[leg]) ? JSON.parse(readFileSync(legs[leg], "utf8")) : null);
const keep = (leg, r) => { if (!extra.length) { mkdirSync("target/gates", { recursive: true }); writeFileSync(legs[leg], JSON.stringify({ status: r.status, stdout: r.stdout })); } };
// The table fills every core seed by seed for ~40 s; the fourteen-day probe is a few long
// sequential chains (one per seed) that would otherwise run alone afterwards — they overlap.
// `signal`: a stderr line that resolves `started` (forwarded stderr otherwise untouched).
const run = (bin, args, signal, env) => {
  let started;
  const ready = new Promise((r) => (started = r));
  const done = new Promise((resolve) => {
    const p = spawn(bin, args, { stdio: ["ignore", "pipe", signal ? "pipe" : "inherit"], env: env ? { ...process.env, ...env } : process.env });
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
// The dayplayer's chains start first. They were the critical path while the table left them
// their cores; with the table at ~2.5 min and the chains at ~1 min alone (docs/ITERATION_SPEED.md,
// 2026-09-24), the table is, so it takes every core but one and shares them while the others run.
const QA_SHARE = Number(process.env.QA_SHARE ?? 0.75);
const cores = os.availableParallelism();
const hit = { metrics: cached("metrics"), qa: cached("qa"), dayplayer: cached("dayplayer") };
const done = (r) => ({ ready: Promise.resolve(), done: Promise.resolve(r) });
const dayplayer = (hit.dayplayer ? done(hit.dayplayer) : run("target/fast/examples/dayplayer", ["--gate", "--seeds", String(seeds), "--loo-seeds", String(looSeeds), "--tuned-seeds", String(seeds), "--threads", String(Math.max(4, Math.round(cores * 0.3)))])).done;
const table = hit.metrics ? done(hit.metrics) : run("target/fast/examples/metrics", [...(full ? [] : ["--quick"]), "--threads", String(Math.max(4, cores - 1)), ...extra], "metrics: quiet ticks measured", { METRICS_QUIET_SIGNAL: "1" });
// The invariants (a job pool of seeds and their legs) start once the table's single-threaded quiet
// per-tick measurement is done (a few seconds), then take three quarters of the cores beside the
// table's all-but-one: at four threads they were the gate's critical path (305 s beside a 200 s
// table); at 24 they end in ~115 s and hand the cores back to the table, which is the critical path
// again (docs/ITERATION_SPEED.md, round 3).
// (a cached table leaves qa every core)
const qaThreads = hit.metrics ? cores : Math.max(4, Math.round(cores * QA_SHARE));
const qa = hit.qa ? Promise.resolve(hit.qa) : table.ready.then(() => run("target/fast/examples/qa", ["--seeds", "30", "--threads", String(qaThreads)]).done);
const [r, p, q] = await Promise.all([table.done, dayplayer, qa]);
for (const [leg, res] of [["metrics", r], ["qa", q], ["dayplayer", p]]) if (!hit[leg]) keep(leg, res);
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
