// Entry. app.ts owns the state machine (client track); render/ owns the viewer (renderer track).
import "./styles.css";
import { start, type DevOptions } from "./app";
start(readDevParams());

/** Dev-only URL params, honoured in a dev build or anywhere with `?dev=1` (tools/dev.sh, tools/playtest.mjs):
 *    ?seed=N        new lineage with that seed on a fresh boot (ignored while a save exists, unless &fresh=1)
 *    ?fresh=1       clear the save first
 *    ?absent=8h     treat last_seen as 30m | 8h | 3d ago, so the offline report runs
 *    ?rules=<text>  url-encoded rule text for importRules, applied to the active set before anything else
 *    ?speed=4       start the watch at `fast` (1 slow · 4 fast · 8 auto, the default)
 *    ?autosend=1    send straight from boot
 *    ?fake_depth=N  with ?engine=fake: every run starts on depth N (boss floor at 5) — read in engine/fake.ts
 *  The one-shot ones (fresh, absent, rules, autosend) are stripped from the address bar so a reload does not
 *  wipe the save or rerun the absence. Returns null outside dev (no window.__riddle then). */
function readDevParams(): DevOptions | null {
  const q = new URLSearchParams(location.search);
  if (!import.meta.env.DEV && q.get("dev") !== "1") return null;
  const o: DevOptions = {};
  const seed = Number(q.get("seed")); if (q.has("seed") && Number.isFinite(seed)) o.seed = seed >>> 0;
  if (q.get("fresh") === "1") o.fresh = true;
  const absent = parseSpan(q.get("absent")); if (absent) o.absent = absent;
  const rules = q.get("rules"); if (rules) o.rules = rules;
  const speed = Number(q.get("speed")); if (q.has("speed") && speed > 0) o.speed = speed;
  if (q.get("autosend") === "1") o.autosend = true;
  let strip = false;
  for (const k of ["fresh", "absent", "rules", "autosend"]) if (q.has(k)) { q.delete(k); strip = true; }
  if (strip) history.replaceState(null, "", `${location.pathname}${q.size ? `?${q}` : ""}${location.hash}`);
  return o;
}

/** "30m" | "8h" | "3d" | "90s" | "3600" (seconds) → seconds; 0 when absent or malformed. */
function parseSpan(s: string | null): number {
  if (!s) return 0;
  const m = /^(\d+(?:\.\d+)?)\s*([smhd]?)$/i.exec(s.trim());
  if (!m) return 0;
  const unit = { "": 1, s: 1, m: 60, h: 3600, d: 86400 }[m[2].toLowerCase()] ?? 1;
  return Math.round(Number(m[1]) * unit);
}
