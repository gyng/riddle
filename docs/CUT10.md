# Cut 10 — Fights only, the wall as a ramp, and sound

*Contract, 2026-09-17. Cohort 6 (build 98b23c9): 74.1 · 75.1, α 0.969, mean 74.6, "fun". The
three unmoved load-bearing axes (pacing 0.6, clarity 0.6, mastery 0.7) and the two weight-1 axes
with no audio at all (feel 0.6, aesthetic 0.6) are this cut.*

## Design

### 1. The watch is the fights (pacing)

Every cohort: "20–30 s of *pick up*", "a D8 run costs 6–8 minutes of screen time". Auto cadence
made travel fast; it is still watched. Now it is not:

- **Default mode `fights`**: the viewer shows the map only as an **interstitial** between fights:
  a 1.5 s card `D3 · 4 rooms · $47` (the ambient line, already there) while the engine runs the
  travel at 16× underneath, then the fight frame cuts in at 1× the moment a scene opens. A run's
  screen time becomes its fights plus a card per floor. `fast` (old auto) and `▶▶|` stay; the
  buttons are `fights · fast · ▶▶|`.
- **Interstitial is expandable**: tapping it holds the map at 8× (today's auto) until the next
  fight, for the player who wants to see the route.
- **Skip is instant**: `▶▶|` in `fights` mode jumps to the next fight's first frame (one press),
  and in a fight to its end.
- Gate: a DEFAULT run's screen time at `fights` ≤ 90 s from send to exit on 30 seeds (median),
  with ≥ 3 fights shown.

### 2. The wall as a ramp (mastery)

K: "D9 0% on every configuration except the one I found; the patch quoted survives 8% for the
same rule — its position hid the real answer".

- The forecast row for a boss floor whose counter fact is known and whose row is absent from the
  set reads `D9 0% · warlord · try: attack boss` (copy: numbers + the counter text from
  `Lineage.counters`, ≤ 3 words after `try:`); tapping the row inserts the counter row at the
  **top** (position is the point).
- Boss-death patches pin the counter row at the top position first and show `survives` for
  *that* placement; the current pin allowed "before-most-fired", which is what hid K's answer.
- The bestiary card's counter chip (Cut 7) inserts at the top too.
- Gate: on 30 seeds where the set lacks the counter and the fact is known, the forecast names it
  and inserting it via the row lifts the boss floor's reach by ≥ 0.3.

### 3. Clarity traps, each one line (clarity)

| Quote | Fix |
|---|---|
| "rest 20m never explained" (K, L) | the chip reads `rest 20m · send skips` permanently, not on tap |
| "heal potion $40 greyed with $165" (K) | a greyed supply shows why: `3/3 slots` or `◆ identify` under the price |
| "1 RUNS · 0 DEATHS right after a death" (K) | the watched-run report counts the death it came from |
| "Ashar slain read as a foe; it was my jackal" (L) | companion deaths read `jackal Ashar fell` (the chronicle's verb) |
| "BANKED 0 beside fourteen $61 return lines" (L) | the tile row is `banked · returned · deaths` in that order with `returned` first when it is the larger; report exits read `returned $61` not `$61` |
| "3 over never explained" (K) | `3 over` becomes `3 hp short` |
| "$26 → $10 mid-run drop" (K) | a theft callout carries the amount: `stolen $16` |
| "2/4 rows made me waste ◆2 on +1 row" (L) | the row unlock card is dimmed while free rows exist, `needs: rows full` |
| "no move-up/down, only drag" (L) | each row gets `▲▼` chips (44 px) beside the drag handle |
| "card: gas step reach +47% did not move the forecast when bought" (L) | the card's delta is recomputed at the card's actual insert position (the end of the set), not at the top |
| "for ~15 s after the absence the screen showed only RUNS 9" (L) | the offline progress label includes `working…`? No copy: the tiles fade in only when the report is complete; until then the bar alone shows |

### 4. Sound (feel, aesthetic)

There is no audio. Sparse, synthesised in-browser (no assets, no loading): a **WebAudio** layer with
six cues, all ≤ 200 ms, all from the same two oscillators so they sound like one instrument:
`hit` (hero hurt: a low thud with a pitch tied to damage), `slay` (a short descending pair),
`rule` (a soft tick when a player row fires, not chores), `telegraph` (a rising two-note warning),
`exit` (bank: a resolved chord; return: the same unresolved; death: a single low note that fades
over 1 s), `level`/`unlock` (a bright arpeggio). Plus a **camp drone**: a very quiet two-note
pad while the camp is open, biome-tinted (Warrens olive → Fens teal → Crypt indigo maps to
three pentatonic roots). Mute in settings, default on after the first tap (autoplay policy),
`feel` unassessed no longer.

### 5. Gates

| Gate | Bar |
|---|---|
| DEFAULT run screen time at `fights` mode, median over 30 seeds | ≤ 90 s, ≥ 3 fights shown |
| `▶▶|` in `fights` reaches the next fight in one press | browser test |
| Forecast names the known-but-absent counter on a boss floor; inserting lifts reach ≥ 0.3 | 30 seeds |
| Every clarity row above has a test or a screenshot | checklist in the report |
| Audio: each cue fires on its event, none longer than 200 ms, mute works, no cue while muted | browser test with an AudioContext spy |
| Cohort 7: pacing ≥ 0.8 from one rater, clarity ≥ 0.8 from one, α ≥ 0.80 | two blind cards |

## Tracks

- **Client/renderer** (`web/src/**`): §1, §2 client (forecast row tap, top insert), §3, §4.
- **Core** (`crates/**`): §2 core (counter pin at the top, forecast row `try`), §3 core items
  (report death count, companion fell text, theft amount, card delta at insert position, row
  unlock `needs: rows full`).
