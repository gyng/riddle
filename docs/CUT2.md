# Cut 2 — Pacing and progression: the hero's day

*Contract, 2026-09-16. Follows Cut 1 (`docs/CUT1.md`, addenda A–E). Tier-1 card on Cut 1:
61.6 promising, gated on `tension` and `expression`. The 14-day player simulation
(`examples/dayplayer.rs`) showed the deeper fault: every yield is scaled by run count (57 runs
per 8 h absence), so class L10 arrives on day 2, all unlocks are bought by day 5, facts
plateau on day 1, marks and gold pile up unspent, and the best depth stalls 9–11 days at D6–8.*

## Diagnosis in one line

Absence produces *many identical runs*; the player's yield should follow *what came home*, at a
rhythm a person can read.

## Design (additions to PLAN.md; PLAN's pillars and anti-pillars hold)

### 1. The hero's day (pacing)

- **Expeditions are longer.** Floors 32×32 (was ≤ 24), the chore threshold for `descend`
  stays 60% seen; target 6–12 minutes at 1×. `Snapshot.w/h` may reach 32; the client already
  crops to the visible field.
- **Camp rest follows every expedition**, as long as the expedition lasted, capped at 30 min:
  the heir heals, companions rest, eggs incubate (an egg hatches after 3 rests, replacing
  "5 expeditions"). A death is followed by a fixed **20-minute wake**: the lineage buries the
  heir and the next one arrives. Rest and wake are the same online and offline; **sending
  skips them** (the player chose to go: that is the presence bonus, inside the 1.5–3× band).
- Result to hit: **6–16 expeditions per 8 h absence** (gate), report reads `11 runs · 3 banked
  · 8 deaths`.
- `Lineage.rest_left_s: u32` (camp rest remaining at report time), `ReturnReport.rested_s`,
  `ReturnReport.banked`, `ReturnReport.returned`. The watch HUD's exit shows `rest 12m` and the
  camp `send` button reads `send` (skips it).

### 2. Yield follows the exit (progression)

- **Bank 100% · return 60% · death 0%** for gold, salvage, class XP and renown. Meta is never
  lost: facts, rules, kennel, forge counts, ledger, unlocks, vault items *not brought*.
- **Bones.** A dead heir's inventory and equipped kit stay on the floor as a `bones` pile
  (persistent per lineage, max 3 piles; oldest expires). A later heir that steps on it
  recovers the items and a `Note`; the bones of a named heir are a `Highlight` (`bones`, 6).
  Fact `bones:<depth>` is learned when a heir dies (so `depth>=N` rows can aim for it).
- Consequence, on purpose: the DEFAULT preset (no return row) yields facts and deaths only.
  The first patch a new player sees is `hp < 30% → return` with a real forecast delta.
- **Renown per absence = the best single run's score** (not the sum). Ranks at `100 r²`.
- **XP** only from banked/returned runs (×1.0 / ×0.6): `xp_to_next = 100 × level²`. L10 should
  land around day 10 for a player who banks.
- **Marks**: new depth (1 per floor), boss (3), trophy (2), rank (1). **No first-kill marks**
  (they front-loaded 15 marks on day 1; first kills stay in bests and the ledger).

### 3. Unlocks: ~30, escalating, unfolding

Catalogue (cost in marks; `needs` = fact/trophy gate; automation-as-reward is the genre's
hallmark and was missing):

| id | cost | needs | what |
|---|---|---|---|
| row5 · row6 · row7 · row8 | 2 · 4 · 7 · 11 | — | rule rows |
| party_slot_2 · party_slot_3 | 4 · 9 | tamed ≥ 1 · tamed ≥ 3 | party |
| vault2 · vault3 · vault4 | 3 · 6 · 10 | — | vault slots |
| rogue · ranger · caster | 4 · 6 · 8 | — · boss 1 · boss 2 | classes (verb ladders below) |
| tame · throw | 2 · 2 | `item:leash` · — | verbs |
| cond_alert · cond_turns · cond_loot · cond_on_kill · cond_on_see · cond_party_hp | 2 each | facts as sensible | condition tokens (no longer free) |
| corridor_fighting · kite_archers · stair_dance · gas_step · pack_break · thief_guard · boss_focus · last_stand | 3 each | a matching fact each (`foe:*:ranged` for kite…) | tactic cards |
| quartermaster | 5 | — | automation: keeps the best weapon *and* armour offline |
| auto_supply | 4 | — | automation: restocks last run's supplies each expedition |
| auto_insure | 6 | — | automation: insures the brought item if gold allows |
| incubator | 4 | egg ≥ 1 | eggs hatch after 1 rest |
| supply_cap_5 | 3 | — | supplies 3 → 5 |
| bone_sense | 3 | `bones:*` | the hero paths to known bones on that floor |
| third_tag | 6 | bred ≥ 1 | breeding may carry 3 tags |

Gate: over the 14-day simulated player, **≥ 10 of 14 days buy an unlock**, and **marks
unspent ≤ 8 at every check-in after day 2**.

### 4. Class ladders (expression)

`ranger`: L1 `shoot` (bow verb; the class starts with a bow), `kite` (keep range 3), L3
`volley` (hit all in a line, cooldown 8), L5 `trap` (stun tile 20 ticks), L7 `mark` (target
takes ×1.5, 30 ticks), L9 `double_shot`.
`caster`: L1 `bolt` (ranged 2–5, no ammo), `ward` (def +2, 30 ticks, cooldown 10), L3
`blink` (3 tiles, no scroll), L5 `slow` (target speed −5, 30 ticks), L7 `nova` (3×3 fire
around self, cooldown 15), L9 `drain` (steal 5 hp).
Each verb is a row token gated by level (as fighter/rogue).

### 5. Facts unfold (three tiers)

`foe:<kind>` on sight (name token) → `foe:<kind>:<tag>` on observing (tag tokens) →
`foe:<kind>:studied` after 5 kills of the kind (unlocks `foe_hp<` targeting of that kind and
+20% tame chance). Ledger column `studied`. Facts gate the tactic cards and the conditions
above.

### 6. Patch candidates are family-shaped (failure, attribution)

The death screen is the main authored moment; the sim showed candidates like
`foe: caster → attack tag:water`. Generator rules: one candidate per family (retreat,
consumables, escape/return, dive, targeting, ally, ID, terrain) with sensible args only:
targeting tags must occur among foes in the trace; thresholds from {20, 30, 40, 50}; verbs
must have had a legal target or item in the last 10 actions; `fired` required in ≥ 50% of
replays; rank by survival edge then forecast delta; three shown, never two of one family.
Performance: verdict ≤ 0.4 s native (REPLAYS 20 → 12 with early exit; candidates ≤ 16).

### 7. Stakes visible while watching (tension)

HUD gains one line under the hp bar: `$47 · sword⚠ · return at D4` (loot on the hero,
brought items with the risk glyph, the row that would bank if any). It turns amber at
`hp < 40%` and reads `death: lose all` when no return row exists. Boss rooms: on `see` of a
`boss` entity the HUD shows `boss · counter: known|unknown` for 3 s and the viewer plays a
one-shot palette flash.

### 8. Copy stays inside budgets; no new surfaces.

## Wire additions

```ts
// Lineage
rest_left_s: number; bones: { depth: number; heir: number; items: number }[];
// ReturnReport
rested_s: number; banked: number; returned: number; bones_found: string[];
// Snapshot
stake: { loot: number; brought: { label: string; insured: boolean }[]; return_row?: number };
// Ev
| { t; k: "rest"; seconds: number }   // emitted at exit; the viewer shows `rest Nm`
| { t; k: "bones"; heir: number; items: number }
// Engine: no new methods. `unlocks()` carries `needs?: string` (human-readable gate) per entry.
```

## Gates (added to `examples/metrics.rs` and `examples/dayplayer.rs`; never weakened)

| Gate | Bar |
|---|---|
| Expeditions per 8 h (DEFAULT and EDITED) | 6–16 |
| Dayplayer: days with ≥ 1 unlock | ≥ 10 / 14 |
| Dayplayer: marks unspent at any check-in after day 2 | ≤ 8 |
| Dayplayer: empty check-ins | ≤ 15% |
| Dayplayer: class L10 not before | day 7 |
| Dayplayer: longest best-depth stall (counter fact known) | ≤ 3 days |
| Patch quality: share of shown patches whose row fired in ≥ 50% of replays | 100% |
| Verdict time (native, fast profile) | ≤ 0.4 s |
| DEFAULT yields 0 xp/gold over 8 h; EDITED banks ≥ 3 runs per 8 h | both |
| All Cut 1 gates | unchanged |

The dayplayer's player model: applies the top patch when it beats baseline by 0.15; when
stalled ≥ 2 days at a boss depth *and* the boss counter fact is known, inserts the counter
row (a human who read the fact would); buys the cheapest affordable unlock; fields the kennel.

## Tracks (disjoint files)

- **Core A** (`engine.rs`, `offline.rs`, `meta.rs`, `hero.rs`, `defs.rs`, `gen.rs`, `facts.rs`,
  `wire.rs`, `tests.rs`, `examples/dayplayer.rs`, `examples/metrics.rs`, presets): §1–§5, gates.
- **Core B** (`trace.rs` only, tests inside it): §6 + verdict speed.
- **Client** (`web/src/ui/**`, `app.ts`, `store.ts`, `engine/types.ts|fake.ts|wasm.ts|proxy.ts`): §7, rest
  display, bones lines in report, `needs` on unlock cards, class picker for four classes,
  studied glyph in the ledger.
- **Art** (`art/**`, `web/public/art/**`): `hero_ranger`, `hero_caster`, `bones` prop (8×8×2),
  boss flash palette entry.

Each track reports: what changed, tests/gates output, wire deviations recorded in
`crates/riddle-core/README.md`.

## Recorded outcome (2026-09-16)

All metrics gates pass on 30 seeds (expeditions per 8 h: DEFAULT 15.8, EDITED 12.8 with 7.2
banked; verdict 0.06 s; patches fired ≥ 50%: 100%). Dayplayer: marks-unspent, empty-check-ins
and L10-not-before-day-7 pass. **Deviation:** "days with ≥ 1 unlock ≥ 10/14" (8.5) and "stall
≤ 3 days" (4) fail because a competent simulated player finishes the 16-floor v1 dungeon on day
4–9 and then has little left to buy. The bars assume the 30-floor dungeon; boss HP was not
inflated to hide it (anti-pillar). They are printed by `tools/gates.mjs` and re-enforced when M7
content (5 biomes, depth 30) or ascension variants land. Rest floor (20–30 min) and the 40 000
tick run cap are recorded in the core README.
