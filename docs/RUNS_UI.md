# Runs UI — the climb goes on; watch it live or look back

*2026-10-02. The owner: "the ui needs to be clearer that runs are always going on in background, and a player can either live watch
or review past runs. upgrade the ui for this keeping in mind tech tree/future multiruns". Addendum: "also keeping in mind initial manual
unlocks for new gamer, but ux affordance is good to have". Builds on docs/UI.md (the frame), docs/TOWN.md (the town), docs/AUTOMATION_TREE.md
§4 and docs/CUT30_5.md (the manual phase: sends by hand until the scout), PROGRESSION_V2 (Cut 31: a second hero, expeditions).*

## 1. The model as it was (read from the code, played on real wasm)

| Question | Before this change |
|---|---|
| Does a run progress while the app is open? | **No.** Only a watched run moves. The town showed `heir rests 20m` as a still number; nothing counted it down, nothing sent him when it ran out. The client wrote `last_seen` every 30 s, so the time the app stood open was never played at all: open 80 s on a rested hero, reload → `rest 1200 s`, the same runs (played, `scratchpad/runsui/` step 1). A hidden tab's time was lost the same way (the absence ran only at boot). |
| How does an absence play? | At boot, `elapsed = now − last_seen ≥ 60 s` → `runOfflineSlice` in 30-min slices (the core: rest, then runs, a begun run finished past the budget, stall-sampling after 20 like runs), merged into one report; then the town walks the runs out of the mouth. |
| What is the watch? | A **live sim**: `send()` (start or resume), `fold()` (the floors the set clears ≥ 95 %), then `step(n)` batches paced by the mode (fights 1×, travel 16×). Not a replay. |
| What history exists? | `chronicle` (one line per ended heir, 40), `graveyard` (heir, depth, cause, `death_id` → `death(id)` for the last kept deaths), `gold_ledger` (20 movements), `meters.runs` (the last 2), `ReturnReport.exits` (a report's exit lines, not kept), `last_run` (one brief). The client keeps **one** watched run's events in memory (`runlog.ts`) for the death chain's clips and the fold's replay. |
| Can a past run be re-simulated? | Determinism holds (same seed + rules + elapsed ⇒ identical events; `replay_hash_per_route`), and a death's verdict replays its last ~100 ticks from checkpoints. There was **no** way to replay an arbitrary past run. |

## 2. Always-on runs, made visible — the lane

The core plays the open app's time as it plays an absence's (`advance(ms)`, §5): the rest runs out, the next run goes down, a run in
flight plays on unwatched. The client asks every 2 s while a run is live and once when a rest falls due (and each minute, so the core's
rest keeps with the town's countdown). A hidden tab's time is the next tick's, or — 60 s and more — an absence with its report, as at boot.
The watch drives its own run; the clock pauses there.

**The lane** takes the rest line's place, under the town, above the console: one row per hero, `≤ 3` rows, then `+N · k live`.

```
phone 400 × 800 (the town above, the console below)

 ┌──────────────────────────────────────────────────┐
 │ (◉) 1st heir  ● live D3  ▕█████▁▁▏28/36  ↻ auto  ▸│ ┌────┐
 └──────────────────────────────────────────────────┘ │ ☰ 3│  ← the log, its new runs
                                                      │log │
   rests:  (◉) 1st heir  ● rests 18m ▕████▁▁▁▁▏ ↻ auto     └────┘
   waits:  (◉) 1st heir  ● waits ▸ send          ⊘ auto 1/3     (the gem lit: SEND 1/3)
```

- **live**: the floor, his hp as a bar, a quick ember beat. A tap watches it. The gem reads `watch` while he is down there.
- **rests**: the countdown and the rest draining, a slow moon beat; `goes down` as it runs out. The gem stays `send` (go now).
- A lane taps only while live (it watches, as the gem and the mouth then do); at home it is a reading, its tip on long-press.
- **waits** (before the scout — the manual phase): `waits ▸ send` in gilt, the lane's edge gilt, the gem pulsing with the scout's count
  (`SEND 1/3`). The lane is the affordance; it points at the gem and the mouth, it does not send itself (day 0 keeps its four surfaces).
- **`auto`** is the quiet preview that runs go on by themselves: `⊘ auto 1/3` greyed until the scout (his count), `↻ auto` in gilt once
  he is hired. No sentence; its tip says `sends him each rest · hired · auto`.
- Tips (long-press / hover): `lane` "a hero's runs · live, resting or waiting", `live` "the run going on now · watch or not",
  `log` "every run · by absence · replays", `replay` "the run again · same rolls".

## 3. Watch live, leave, look back

- **Into the watch**: the lane (live), the gem (`watch`), or the mouth. The watch resumes the run in flight (`send()` resumes a begun run;
  nothing is counted twice).
- **Out of it**: a `town ↻` tile on the console. The run is not stopped: back in the town the lane shows it live, the open app's clock
  plays it on unwatched, and its exit settles as an unwatched one does (the keep order). The `↻` is the mark; the tip says the rest.
- **The runs log** (the lane's `☰ log`, its count the runs not yet looked at): every run the core keeps (`Lineage.runs`, the last 60),
  newest first, **folded by absence** — `▸ away · 17 runs · D9★ · $1 240 · 3h` is one line until opened; the runs played while the app
  was open fold as `here`. An entry:

```
 ▾ away · 17 runs · D9★ · $1 240                      3h
     #41  ⌂ D9★  $212  ✦2  4m                          ▶
          banks every record · 3h
     #40  ☠ D7   $0         3m                          ▶      ← a tap opens its verdict
          slain · goblin · 3h
     +12 runs · like these                                     ← the absence's sampled runs
 ▸ here · 3 runs · D8 · $310                           now
```

  the run's number · how it ended (`⌂` bank · `↩` return · `☠` death) and the exit's reason (≤ 3 words, the core's) with when · its
  floor, `★` a new best · the gold home · finds `✦` · its length · `◉` watched · `▶` replay. A death's entry opens its death screen.
  A tap on an entry opens **the run's card** (c305-runclear's clear screen: the end's seal, the reason, the floor and `new best`, the
  gold, the finds in their rarity rims) with its `▶` under it — the run-clear card is one entry of the history. The finds' mark `◆N`
  takes the rarest find's pigment.
- **Replay**: `▶` asks the core for the run re-simulated from its send (`replay(id)`, §5): the floors one after another in the map frame
  at 16×, floor chips to jump. It is the same run: the same events, byte for byte (the gate compares hashes).
- **The chronicle folds in**: the log's second tab, `heirs` (one line an heir, a kept death opens its verdict); its console tile goes. The
  `ledger` tile is the bestiary (no overlap) and stays. The report's `runs` tile opens the log on the absence's fold.

## 4. IA and density

| Place | What it gets |
|---|---|
| top bar | nothing new |
| town (well) | the mouth still sends (and watches a live run); the hero's **tent** keeps his log once he has runs (the hero sheet before; his class and look stay on the portrait) |
| under the town | **the lanes** (the rest line's row) + the log's stud |
| console | the gem: `send` · `watch` (live) · `SEND 1/3` (waits); no new tile |
| watch console | `town ↻` (the 7th of 8 tiles) |
| sheets | **log** (`runs` · `heirs`), a run's **card**, **replay** (sheets over it) |
| desktop 1440 × 900 | the same lanes in the centre column under the town (the frame's `rest` row), the log a sheet |

```
desktop 1440 × 900
┌──────────────────────── top bar ──────────────────────────────────────────────┐
│ rules / packages │                 THE TOWN                    │  shaft        │
│                  │                                             │               │
│                  │                                             │  meters       │
│                  ├─────────────────────────────────────────────┤               │
│                  │ (◉) 1st heir ● live D3 ▕████▁▏ ↻ auto ▸ │☰log│               │
│                  │ (◉) 2nd hero ● rests 9m ▕██▁▁▏ ↻ auto   │    │  ← Cut 31     │
├──────────────────┴─────────────────────────────────────────────┴───────────────┤
│ (portrait)   [ forge · vault · kennel · bank · packages · … ]          (SEND)  │
└────────────────────────────────────────────────────────────────────────────────┘
```

Budgets (docs/UI.md §6, the density memory): ≤ 12 elements above the fold at 400 × 800. The mid-game town already stood at 12 (purse,
settings, mouth, tent, crate, blacksmith, storehouse, staked plot, pill, look, packages, gem), so the log had to take a place, not add one:
the **tent** (the hero's own, which opened a sheet repeating the portrait's class and level) keeps his log, and the lane's `log` stud is a
second door to it; the **chronicle** tile (from the 5th heir) folds into the log's `heirs` tab. A lane taps only while live, where it
does what the gem and the mouth do (watch). Measured: day 0 7, mid-game 12, three lanes (fake) 11. Day 0 keeps ≤ 5 surfaces (the lane
is inert, the log hidden until a run); labels ≤ 2 words (`live`, `rests`, `waits`, `auto`, `log`, `here`, `away`, `town`, `runs`,
`heirs`); no sentences; five new tooltip terms (lane, live, log, replay, away).

## 5. Core support (all truth in Rust)

- `advance(elapsed_ms) → { ended, live }`: the open app's clock — `run_offline`'s loop in a mode that leaves a run in flight at the
  budget's end, never samples, keeps renown per run and opens no system by itself; sub-tick time carried. Before the scout it does
  nothing (the hero waits).
- `Lineage.live` (the run under way: floor, hp, tick), `Lineage.runs` (the log, 60: via `away|town|watched`, the absence, the clock
  at the end, start, depth, tier, reason, gold, found, turns, best, death_id; a `sampled` record per sampled absence),
  `Lineage.absences`, `Lineage.clock_s`, `Lineage.replays`.
- **Replays**: at each real send the core keeps a capsule — the game as the run began (the lineage, the run at tick 0, the events the send
  left) and the run's inputs (`choose`, `bail`) — for the last 40 runs, in memory (not the save: a reload keeps the log, not the
  capsules; an absence's runs are replayable after it because the absence is played in this session). `replay(id)` plays the capsule to
  the end: per floor its first snapshot (later entities, items and seen tiles folded in) and its events, and the FNV-1a hash of the
  events. Harnesses that run real games at scale pay nothing (the history switch).

## 6. Future-proofing — no redesign for Cut 31

- **2–3 heroes**: `Lineage.heroes` (reserved) is the lanes' array; each row is a hero (its face, name, state, auto). A fourth folds into
  `+1 · 1 live`. Each live lane opens the watch on its hero's run (the watch takes a hero id then).
- **An expedition** is a lane with `kind: expedition` (a `⚑` face): `live D14 · expedition`, then `rests`. Its runs fold in the log
  under their own `away`/`here` folds, tagged by hero.
- **The scout and auto-send** are the `auto` slot: greyed with his count before, gilt after; switched off in settings, the lane goes
  back to `waits ▸ send` — the same row.
- **Auto-lanes** (PROGRESSION_V2 rung 11) light each lane's `auto`.

## 7. The manual phase (the first session, AUTOMATION_TREE §4 unchanged)

The first sends are taps: the lane reads `waits ▸ send` with `⊘ auto 0/3`, and the gem `SEND 0/3`. After a send the lane is `live D1`
(leave the watch: he keeps going); back home it is `waits` again, the count `1/3`, and the log holds run 1. The chests stay the porter's
chore (the town's chest). At the scout's hire the slot turns `↻ auto` and the lane reads `rests 20m` — from then on the runs go on
without the player, which the lane now shows.

## 8. Gates (`web/tests/runsui.mjs`, real wasm unless named)

- the lane shows **waits** (fresh, before the scout), **live** (a run in flight) and **rests** (home after it, the scout hired);
- the lane opens the live watch on the run in flight (the same run id);
- leaving the watch keeps the run going: the run's tick advances in the town and it ends in the log as `town`;
- the log lists runs grouped by absence (`away` folds, `here` folds), a death opens its verdict;
- a replay opens and plays a past run identically (the hash of the run as watched = the replay's);
- density: ≤ 12 elements above the fold at 400 × 800, labels ≤ 2 words, no sentences, the new terms carry tips;
- the lanes render N rows from an array (1 and 3 heroes, fake), a 4th folds into `+1`;
- the manual phase: day 0 `waits` with the gem lit and the scout's count, the lane inert, the log hidden; after run 1 the log has one entry.

## 9. Results

*(filled at the end of the build)*
