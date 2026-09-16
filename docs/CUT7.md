# Cut 7 — The first hour is not five floors

*Contract, 2026-09-17. Across four blind raters and two cohorts, `surprise`, `pacing`,
`expression` and `mastery` sat at exactly 0.6 every time. Their evidence shares one root:
every rater's hour ended at the D5 Warlord, and the 8-hour absence stayed there ("D5 was the
same warlord fight four times"; "heroes never leave L1 and the D6+ wall stayed"; "3 of 7 rows
are shipped defaults or patches"; "the next thing to learn is never shown"). Cuts 3–6 added
depth 30, story and clarity, but a new player still experiences five floors. This cut changes
the shape of the first hour, not the content.*

## Design

### 1. The wall arrives when the player has a policy, not a preset (core)

- **The Warlord moves to D8**; D5 hosts a *lieutenant* (Goblin Captain: rallies once, no
  shield wall) that the preset beats about half the time, so the first hour's spine is
  D1–D5 learning → D6–D8 a new biome's *first* trait (Fens gas) → the wall. The player meets
  three distinct problems in the first hour instead of one.
- **Wall bosses telegraph their counter as a row the hero *writes on the bestiary card*.**
  On the first surviving observation of a boss mechanic the fact reads
  `boss:goblin_warlord:counter=attack tag:boss` (Cut 6) and the *ledger entry* for the boss
  shows the counter row as a chip the player can tap into a set. Mastery becomes visible:
  "what I can do now" is a row I could not write an hour ago.
- **DEFAULT dies by D8 (was D6); EDITED (good.json) reaches D10 as before; TRIVIAL never passes
  D8.** Bars move with the wall, not weaker.

### 2. The player's rows outnumber the presets (core + client)

- The shipped preset shrinks to **two rows** and the editor opens with **two empty slots**
  (max_rows 4 as before). Patches are offered as before, but the third and fourth rows come
  from the player. A card's rows count as the player's.
- The forecast panel shows, under the bars, **`yours: 3 of 5 rows`** (copy:label ≤ 3 words) —
  the expression proxy made visible, nothing more.

### 3. Every floor band has a *situation the player must answer with a row* (core)

D1–5 already have shrine/vault/nest/stray. Add one per band that a fixed preset cannot pass but
one row solves, so `surprise` is directly encountered and changes play:
- **D3 the thief's den**: monkeys steal *from the vault-brought item first*; the row is
  `foe: thief → attack thief` or `on_see: thief → step back`.
- **D6 the gas lock**: a corridor filled with lingering gas between the stairs; `foe: gas ·
  adj → step back` or `alert ≥ 3 → descend` gets through; melee-everything sets bleed.
- **D9 the captive gate**: a captive chained across the only path; `captive → free` (an ally
  for the wall) or `captive → attack` (the coward's way, a trophy `no_friends`).
- **D12 the crypt's hunger**: wraiths drain max HP unless the hero rests in *light* (a lantern
  or a lit shrine); `hp < 60% · foes = 0 → rest` is not enough, `on_see: shrine → pray` is.
Each is a fact, a token and an episode. Gate: each situation appears on ≥ 90% of runs
reaching its depth; a fixed DEFAULT preset passes none of them on ≥ 80% of seeds.

### 4. The watch is a scene, not a walk (client + core)

Raters at 1× saw "30–60 s of pick up". With auto cadence they saw 1–2 min runs. The middle:
- **Rooms are scenes.** On entering a room with ≥ 2 hostiles the viewer holds 1× until the
  room is clear or left; corridors and empty rooms run at 8×. (Auto cadence today keys on
  "hostile in view"; this keys on rooms so the fight's *approach* is watchable.)
- **Ambient callouts at 8×**: `D3 · 4 rooms` on a floor change, `$47` on pickup, `alert 3`
  when the clock ticks — one per 10 s at most, so the fast stretches read as travel.
- **A run's end at 1×**: the last 30 ticks before any exit play at 1× (the bank walk-out, the
  killing blow), so the player always sees how it ended.

### 5. Levels arrive in the first hour (core)

"Heroes never leave L1." XP thresholds start lower: `xp_to_next = 60·L²` for L1–3 (L4+
unchanged), and a *watched* bank grants +50% XP (presence bonus inside the 1.5–3× band;
offline unchanged). L2 (fighter `cleave` at L3 comes one session earlier) lands in the first
hour for a player who banks once.

## Gates

| Gate | Bar |
|---|---|
| DEFAULT dies by ≤ D8 (was D6) | ≥ 80% seeds |
| TRIVIAL never passes D8 | ≥ 90% |
| EDITED reaches ≥ D10 | ≥ 50% |
| Situations (D3/D6/D9/D12) appear when the depth is reached | ≥ 90% |
| DEFAULT passes each situation | ≤ 20% of seeds |
| Dayplayer: first hour (3 check-ins) ends with best ≥ D6 and ≥ 2 player rows | ≥ 80% of seeds |
| Dayplayer: L2 by the end of day 1 for a banking player | ≥ 80% |
| All Cut 1–6 gates | unchanged (tune content, never bars) |
| Cohort 3: surprise, pacing, expression, mastery each ≥ 0.6 from both raters with at least two of them ≥ 0.8 | two blind cards, α ≥ 0.67 |

## Tracks

- **Core** (`crates/**`): §1, §2 preset, §3, §4 (`Snapshot.room: {id, hostiles}`; last-30-tick
  marker on the exit), §5, gates.
- **Client** (`web/src/**`): §2 `yours: n of m`, §4 cadence keyed on rooms + ambient callouts +
  1× ending, bestiary card counter chips (§1).
- **Art**: none.

## Recorded outcome (2026-09-17)

Core and client landed; all metrics gates green (DEFAULT ≤ D8 100%, TRIVIAL ≤ D8 100%,
EDITED ≥ D10 100%, COUNTERED ≥ D14 53%, FULL ≥ D29 87%, walls 100/100/97%, dice 4.2%);
situations appear 98–100% and DEFAULT passes 0/3/0/0%; first-hour dayplayer bars 100%.
Deviations: the Captain lets the preset through D5 in 41% (bar said ~60%); the lock's one-row
answer passes 60%. Verdict-time gate now measured single-threaded (0.10 s). Cohort 3 on build
207cc0e: raters E (seed 53) and F (seed 67).
