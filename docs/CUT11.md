# Cut 11 — The death screen traces the chain

*Contract, 2026-09-17. Cohort 7 (build a7c47e9): 76.8 · 74.1, α 0.878, mean 75.5, "fun". Every
load-bearing axis has been awarded 0.8 by a blind rater; none has been awarded 1.0. The 1.0
anchor needs "no material lapse across the horizon and a named genre benchmark does not do it
better". The axes closest to it are `failure` and `attribution` ("the death screen is the game",
"every death is yours and it tells you why" — praised unprompted by all fourteen raters), and
their one named lapse is identical across cohorts: **the decisive cause is sometimes hidden.***

> M: "'R2 no item' on the death sheet, while 'A thief snatched the heal potion' lives only in
> the morgue text." · N: "monkeys stole my heal potions for five straight runs and the trace only
> ever said 'R1 no item'; survivor runs keep five turns of trace, so 'where did my potions go' is
> unanswerable." · K (cohort 6): "the patch's position hid the real answer." · J (cohort 5): "a
> death labelled dice got no patch even though 'ogre winds up' was right there."

No roguelike explains a death as a *chain*. NetHack's dumplog and DCSS's morgue list events;
Brogue names the blow. A death screen that says *"R1 drink heal did not fire: no heal held — the
den on D3 took it (tick 2140) — you had no thief row"* is the benchmark Riddle can set.

## Design

### 1. Causes have causes (core: `trace.rs`, `engine.rs`)

Every row-accounting reason that is a *state* (`no item`, `none held`, `no path`, `not in view`,
`cooldown`, `locked cond`) gets a **because**: the most recent event in the run that put the
state there. The engine keeps a small per-run **provenance log** (cap 64): for each inventory
slot, the last event that emptied or filled it (`stolen by monkey #14 on D3 t2140`, `drunk at
t1800`, `never found`); for each cooldown, the use that started it; for each `not in view`, the
tile where the target was last seen and when; for `no path`, the blocker (a chained captive, a
gas cloud, a sealed stair). `TraceTurn.rows[i].because?: { text: string; t: number; depth: number
}` (≤ 8 words: `den took the heal, D3`).

### 2. The chain on the death screen (client `death.ts`)

Under the trace, the row accounting becomes a **chain**: each `because` is a tappable link that
scrubs the replay (the renderer has `seek(t)`) to that tick in the fight frame, so the player
*sees* the theft. The chain reads top-down:

```
goblin archer · D6 · 1 hp short · gap
R1 drink heal   no item   ← den took the heal, D3        [watch]
R2 retreat      no path   ← gas cloud, this room          [watch]
R3 attack       fired
```

The **patch** family gains the chain's root: when the root cause is a theft, the top candidate
is `foe: thief → attack thief` (or the thief-guard card if owned) with its forecast delta, not the
symptom's `hp<N → drink unknown`. When the root is a locked condition, the patch is the unlock
(`◆2 cond: alert`) with its delta. Gate: on 30 seeds, ≥ 80% of deaths whose trace has a `because`
show a patch that addresses the root, and inserting it beats the symptom patch's delta.

### 3. Survivor runs keep a fuller trace (core)

Rater N: "survivor runs keep only five turns". The exit trace grows to the last 10 hero turns
plus every `because` event in the run (provenance is small). Report exit lines' `trace` chip
shows the chain too.

### 4. `dice` is never empty (core)

Rater J: "a death labelled dice offered nothing even though the telegraph was right there". A
`dice` verdict shows the chain (what would have had to be different) and, when a telegraph
preceded the blow, a `foe: telegraph → retreat` candidate labelled with its measured survival even
below the bar (`survives 40% · dice`), so the screen always names *the* alternative.

### 5. Money is a chain too (clarity)

N: "$166 → $58 across a run that returned $36". The exit ledger line already reconciles; the
report's exit line becomes tappable to the gold sheet filtered to that run (`−$40 heal · −$30
leash · +$36 returned D5 · −$34 salvage? no: +$8 salvage`), so a purse delta is always three
lines away.

## Gates

| Gate | Bar |
|---|---|
| Every state reason in a death trace carries a `because` when the run has one | ≥ 95% over 100 deaths |
| Root-cause patch offered and beats the symptom patch's delta | ≥ 80% of deaths with a theft/lock root, 30 seeds |
| Survivor exit traces carry 10 turns + provenance | test |
| `dice` deaths show a chain and a named alternative | 100% |
| Chain links scrub the replay to the tick (browser test) | passes |
| Cohort 8: `failure` or `attribution` awarded 1.0 by at least one rater; mean ≥ 78; α ≥ 0.80 | two blind cards |

## Tracks

- **Core** (`crates/**`): §1, §3, §4, the root-cause patch in §2, gates.
- **Client** (`web/src/**`): §2 chain UI + replay scrub, §5 gold sheet filter.
