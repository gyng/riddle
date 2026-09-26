// Wall-clock checks under machine load (docs/ITERATION_SPEED.md, round 3). A client gate that times the page (a paint ≤ 1.2 s, a
// card up ≤ 1.2 s, a stretch ≤ 5 s) reads the machine as well as the client: seven headless browsers rendering on the CPU beside it
// (the suite itself), a gate table, another agent's run. `measured` takes the first reading; when it fails while the machine is
// loaded it measures once more — the bar unchanged — and the line says both readings and the load. An idle machine never retries.
import os from "node:os";

/** The 1-minute load average against the cores; `high` from an eighth of them (4 on 32: the suite's own three or four
 *  SwiftShader browsers beside a timed test read 4–12; an idle box reads < 2). */
export function load() {
  const l1 = os.loadavg()[0], cores = os.availableParallelism();
  return { l1, cores, high: l1 >= cores / 8 };
}

/** `measure()` → `{ ok, line, … }`; retried once when it fails on a loaded machine. Resolves to the reading the gate stands on
 *  (the second's fields, its line carrying the first's). */
export async function measured(measure) {
  const first = await measure();
  if (first.ok) return first;
  const l = load();
  if (!l.high) return first;
  const again = await measure();
  return { ...again, line: `${again.line} [retried once: load ${l.l1.toFixed(1)} on ${l.cores} cores; first reading: ${first.line}]` };
}
