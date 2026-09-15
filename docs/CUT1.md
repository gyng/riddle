# Cut 1 — v1 playable: implementation contract

Scope: `PLAN.md` M0–M6 at v1 content counts, built as four parallel tracks that meet at the
wire types in this file. All game truth lives in Rust. TypeScript renders, edits and infers
presentation only. Determinism: same lineage seed + same rules + same elapsed ⇒ identical
events forever (no wall clock, no HashMap iteration order in results).

## Repository layout

```text
Cargo.toml                  workspace: crates/*
crates/riddle-core/         sim, rules, facts, forecast, offline, chronicle, sifter, meta, save; tests
  src/{lib,rng,geom,tiles,gen,descent,defs,monster,item,hero,rules,tokens,engine,turn,ai,
       facts,trace,chronicle,sifter,forecast,offline,meta,save,probes}.rs
  examples/metrics.rs       bot gates (PASS/FAIL table)
  examples/cli.rs           text playthrough: run a rule set, print the trace
crates/riddle-wasm/         wasm-bindgen bridge; JSON strings in/out
web/                        Vite + TS (no framework) + three.js
  src/engine/{types.ts,wasm.ts,fake.ts}   Engine interface; fake for UI dev
  src/ui/                   editor, return report, death screen, vault, camp, settings
  src/render/               three.js replay viewer (pixel pipeline)
  src/app.ts                state machine: camp ⇄ watch ⇄ death ⇄ report
  public/art/               packed sprites (from art/)
art/                        manifest.json, prompts, generated/, pack.py, art-qc.py
tools/                      verify.sh, gates.mjs, copy-lint.mjs, webmcp-inspector.mjs
docs/CUT1.md                this file
```

## Game rules (authoritative summary; details are the core's tests)

**Grid and turns.** Floors ≤ 24×24. Tiles: `floor wall door stairs_down stairs_up water
chasm`. Overlays with TTL: `gas` (caustic, 3 dmg/turn to anything inside), `fire` (5 dmg,
spreads to adjacent floor once). 8-connected movement, 4-connected corridors. Vision: radius
7, line of sight. One hero action per turn, then every monster acts once (fast monsters:
twice every other turn).

**Hero.** No XP. `hp/max_hp`, `atk (min,max)`, `def`, `speed`, class verbs, inventory (10
slots), equipped weapon + armour. Damage = roll(atk) − def, min 0, 80% to hit. Deterministic
crit: none. Power comes from gear and potions found.

**Classes (v1).** `fighter`: verbs `attack shield_bash(stun 1 turn, cooldown 5)`.
`rogue`: verbs `attack vanish(untargetable 3 turns, cooldown 12) throw`. Both share the
movement/item verbs.

**Exits.** `stairs_up` on every floor (D1's leads to camp). `bank` at stairs_up ends the run
with 100% of run loot; `return` verb anywhere ends with 60%; death 30%. Vault items brought
are lost on death only. Meta is never lost.

**Descent (per lineage seed).** Fixed biome order and bosses; floors regenerate per run.
v1: D1–5 *Warrens* (packs, thieves; boss D5 Goblin Warlord: summons, shield-buffs allies),
D6–10 *Fens* (water, gas, bloats, jellies; boss D10 Bloat Mother: pops into rooms of gas,
must be killed at range or stepped away from), D11–15 *Crypt* (undead, drain, reflect; boss
D15 Lich: reflects thrown items, summons skeletons; counter: melee only, kill summons
first). Reaching D16's stairs is the ending. Grudge monsters: a named monster of the biome
that killed an heir gets +10% stats and a name; it lives on the floor it killed on.

**Monsters (15).** Tags in brackets are facts learned by observation.
`rat` · `jackal [pack, fast]` · `goblin` · `goblin_archer [ranged; telegraph: draws]` ·
`goblin_conjurer [caster; summons 2 blades]` · `monkey [thief; steals, flees]` · `ogre [heavy;
telegraph: winds up, 2× dmg next turn]` · `bloat [gas; pops on death]` · `pink_jelly
[splitter]` · `eel [water; only attacks from water]` · `skeleton [undead]` · `ghoul [undead,
pack; paralyse 1 turn on hit 20%]` · `wraith [undead; drains max_hp 1]` · `captive [ally
candidate]` · bosses as above `[boss]`.

**Items (25).** Weapons `dagger(2-4) sword(3-7) axe(4-9, -1 speed) bow(2-6 ranged)`; armour
`leather(+1) mail(+3, -1 speed) plate(+5, -2)`; potions (flavour randomised per lineage,
identified on drink/throw, fact persists) `heal(+50%) strength(+1 atk permanent) speed(3
turns) invisibility(5 turns) poison(throw: 8 dmg over 4) caustic(throw: gas 3×3) confusion
(throw: 3 turns) fire(throw: fire 3×3)`; scrolls `teleport blink(3 tiles) fear(foes in view
flee 5) mapping identify enchant(+1 to equipped) darkness(foes lose sight 4) summon_ally
(spectral hound 20 turns) aggravate(malevolent: all foes on floor come)`; misc `gold` (loot
value). Unknown items show as `potion? scroll?` with a benevolent/malevolent hint 50% of the
time (Brogue).

**Facts.** `foe:<name>` on first sight; `foe:<name>:<tag>` on first observation of the tag
behaviour; `item:<flavour>=<kind>` on identify; `biome:<name>` on first entry; `boss:<name>:
counter` on surviving the telegraph once. Facts gate condition tokens (below) and are the
first block of the return report.

**Rules.** Ordered rows, ≤ 8, each `conds (≤ 2) → verb(arg?)`. Evaluated top-down each turn;
first row whose conds hold and whose verb can execute fires. If none fires: engine chore
(`explore` nearest unseen, picking up items; when nothing unseen, `descend`). Sanity: no
drink at full HP, no read of an owned-known-useless scroll, no `retreat` with no foe in view.

Condition tokens (gated by facts where marked):
`hp<N hp>N foes>=N adj>=N foe_tag:T* foe_hp<N item:K* unknown_item floor_seen>=N depth>=N
alert>=N in_corridor path_stairs ally loot>=N turns>N on_hurt on_kill on_see`
(`*` = gated: `foe_tag:T` needs any fact with tag T; `item:K` needs the kind identified).
Verb tokens (class/unlock gated): `attack(nearest|lowest|tag:T) retreat back_corridor
drink(K|unknown) read(K|unknown) throw(K, nearest|tag:T) descend bank return rest pick_up
free_captive shield_bash vanish`.

**Traits (one per heir).** `greedy` (picks up adjacent items even with foes adjacent),
`cowardly` (retreats at hp<50% before any row), `curious` (uses an unknown item when safe
even if no row says so), `brave` (never retreats from a single foe). Every deviation emits
`RuleFired{row:-1, trait}`.

**Forecast.** 50 sims on fresh floor seeds of the same descent; reports reach% per depth and
top 3 death causes, **only up to `lineage.best_depth + 1`**; deeper is `?`.

**Offline.** Turn rate 1/s. `run_offline(elapsed_s)` consumes turns across consecutive
expeditions; the hero is left mid-run where the budget ran out. After 20 consecutive runs
with no new fact and no new best, sample 20 and extrapolate; report `sampled: true`.

**Meta.** `marks` earned on new bests only: first reach of each depth (1), first kill of
each monster (1), each boss (3), each trophy (2). Unlocks (cost): rows 5/6/7/8 (2/3/4/5),
class rogue (4), vault slots 2/3 (3/5), tactic cards `corridor_fighting` `kite_archers`
`stair_dance` (3 each), verbs `throw` for fighter (2). Trophies (v1): `no_heal_D5`,
`ranged_only_D5`, `pacifist_floor` (descend a floor with 0 kills), `boss_untouched`.
Facts gate condition tokens; marks gate everything else.

**Chronicle and sifter.** Every run emits `Note`s (DCSS auto-notes). Sifter patterns with
scores: near_death (hp ≤ 10% then floor survived, 5), comeback (hp ≤ 20% → boss killed, 8),
first_kill (3), ally_lost (4), gamble (unknown item used; +2 if malevolent survived),
stolen (2), boss (6). Reel = top 5 by score.

**Death verdict.** `gap` if any *available* token combination (one added or edited row from
the unlocked vocabulary) survives the last 10 turns in ≥ 60% of 20 replays from the turn-10
state; else `dice`. Candidate patches = up to 3 such rows, ranked by survival, each with the
full-forecast delta at the death's depth.

## Wire types (`web/src/engine/types.ts`; Rust mirrors with serde, snake_case JSON)

```ts
export type Cond = { k: string; n?: number; t?: string };          // {k:"hp<",n:40} {k:"foe_tag",t:"pack"}
export type Verb = { v: string; a?: string };                      // {v:"drink",a:"heal"} {v:"attack",a:"tag:caster"}
export type Row  = { conds: Cond[]; verb: Verb };
export type RuleSet = { rows: Row[]; name?: string };

export type Vocabulary = { conds: Cond[]; verbs: Verb[]; max_rows: number };  // what the editor may offer

export type Tile = "floor"|"wall"|"door"|"stairs_down"|"stairs_up"|"water"|"chasm";
export type Overlay = { x: number; y: number; k: "gas"|"fire"; ttl: number };
export type Entity = { id: number; kind: string; name?: string; x: number; y: number;
                       hp: number; max_hp: number; tags: string[]; ally?: boolean; telegraph?: string };
export type FloorItem = { id: number; x: number; y: number; kind: string; known: boolean; label: string };
export type Snapshot = {
  depth: number; biome: string; w: number; h: number; tiles: Tile[]; seen: boolean[]; visible: boolean[];
  overlays: Overlay[]; hero: Entity & { inv: InvItem[]; weapon?: string; armour?: string; class: string; trait: string };
  entities: Entity[]; items: FloorItem[]; alert: number; turn: number; loot: number;
  run: { id: number; heir: number; started_turn: number };
};
export type InvItem = { id: number; kind: string; known: boolean; label: string; hint?: "benevolent"|"malevolent" };

export type Ev =
  | { t: number; k: "move"; id: number; x: number; y: number }
  | { t: number; k: "attack"; src: number; dst: number; dmg: number; hit: boolean; verb?: string }
  | { t: number; k: "hurt"; id: number; dmg: number; hp: number; cause: string }
  | { t: number; k: "die"; id: number; cause: string }
  | { t: number; k: "rule"; row: number; verb: Verb; text: string }        // row -1 = trait, -2 = chore
  | { t: number; k: "telegraph"; id: number; what: string }
  | { t: number; k: "pickup"; id: number; item: string }
  | { t: number; k: "use"; item: string; outcome: string }
  | { t: number; k: "fact"; fact: string }
  | { t: number; k: "overlay"; x: number; y: number; ov: "gas"|"fire"; ttl: number }
  | { t: number; k: "spawn"; e: Entity }
  | { t: number; k: "steal"; id: number; item: string }
  | { t: number; k: "ally"; id: number; state: "freed"|"lost" }
  | { t: number; k: "descend"; depth: number; biome: string }
  | { t: number; k: "exit"; tier: "bank"|"return"|"death"; loot_kept: number }
  | { t: number; k: "note"; text: string }                                  // chronicle line, ≤ 8 words
  | { t: number; k: "callout"; text: string };                              // ≤ 3 words, for the renderer

export type StepResult = { events: Ev[]; snapshot: Snapshot; run_over: boolean };

export type Forecast = { depths: { depth: number; reach: number }[]; causes: { cause: string; share: number }[];
                         known_to: number };
export type Trace = { turns: { t: number; row: number; verb: Verb; hp: number; foes: number; telegraphs: string[] }[] };
export type Death = { run_id: number; depth: number; cause: string; margin: string; verdict: "gap"|"dice";
                      trace: Trace; patches: { row: Row; insert_at: number; survive: number; forecast_delta: number }[];
                      morgue: string };
export type Highlight = { pattern: string; score: number; t: number; run_id: number; text: string };
export type ReturnReport = {
  elapsed_s: number; runs: number; sampled: boolean;
  learned: string[]; bests: string[]; found: InvItem[]; deaths: { cause: string; n: number }[];
  pending: string[]; reel: Highlight[]; marks_earned: number; worst_death?: Death; live: Snapshot;
};
export type Lineage = { seed: number; heir: number; trait: string; class: string; best_depth: number; marks: number;
                        facts: string[]; unlocks: string[]; vault: InvItem[]; graveyard: { heir: number; depth: number; cause: string; deeds: string[] }[];
                        trophies: string[]; sets: RuleSet[]; active_set: number; ended: boolean };

export interface Engine {
  newLineage(seed: number): Lineage;
  load(save: string): Lineage;  save(): string;
  vocabulary(): Vocabulary;
  setRules(set: RuleSet): void;
  loadout(itemIds: number[]): void;
  forecast(): Forecast;
  send(): Snapshot;                    // start (or resume) an expedition
  step(turns: number): StepResult;     // advance live view
  runOffline(elapsedS: number): ReturnReport;
  death(runId: number): Death;
  buy(unlock: string): Lineage;
  lineage(): Lineage;
  exportRules(): string;  importRules(text: string): RuleSet;
}
```

WASM exports (`riddle-wasm`): `struct Game` with constructor `new(seed)`, `load(json)`,
`save()`, and one method per interface member taking/returning JSON strings. wasm-bindgen
camelCases method names (`run_offline` → `runOffline`).

## Acceptance gates (`examples/metrics.rs`, 30 seeds, printed PASS/FAIL)

| Gate | Bar |
|---|---|
| DEFAULT dies by ≤ D6 | ≥ 80% of seeds |
| EDITED (the shipped `good.json` set) reaches ≥ D10 | ≥ 50%; gap vs DEFAULT ≥ 15 pts |
| RANDOM loses | 100% |
| PASSIVE loses by ≤ D3 | 100% |
| LEARNED (DEFAULT + all facts) | mean depth ≤ DEFAULT + 2 |
| Unfair deaths (verdict `dice`) | ≤ 5% of deaths across bots |
| Top death cause share | < 35% |
| Deaths tracing to a player row (`gap`) | ≥ 70% |
| Events per 60 turns (renderable kinds) | ≥ 6 |
| Replay hash | identical across two runs of the same seed+rules+elapsed |
| Forecast | `known_to == best_depth + 1` always |
| Offline 8 h | every seed: `learned ≥ 1`, `pending ≥ 1` |

## Track contracts

**Core (Rust).** Everything in "Game rules" and the wire types; `cli` example; `metrics`
example with the gate table; unit tests for: determinism, rule evaluation order and
fall-through, sanity rules, every monster tag behaviour, every item, exits and loot tiers,
facts gating vocabulary, forecast bounding, offline budget and sampling, verdict and patches,
sifter patterns, marks on new bests only, save round-trip. Ship `presets/{fighter,rogue}.json`
(DEFAULT) and `presets/good.json` (EDITED).

**Client (TS).** `engine/fake.ts` implementing `Engine` with canned data so UI builds
before wasm lands; `app.ts` state machine; screens: camp (editor + vault + unlocks +
forecast + send), watch (renderer + callouts + speed), death, report. Copy budgets enforced by
`tools/copy-lint.mjs`. Phone-first layout. Save to localStorage + export/import text. PWA
with versioned precache.

**Renderer (TS, `src/render/`).** Replay viewer consuming `Snapshot` + `Ev[]`: pixel
pipeline per `research/art-tech.md` (env texel 4 device px, 8×8 tiles, 270-wide target,
snapped ortho camera, palette + Bayer blit, two-density sprites, contact shadows, callouts
as bitmap text in the low-res target, telegraph glyphs). Primitive fallbacks for every
sprite id. `render/index.ts` exports `createViewer(canvas): { load(snap), apply(ev[]), setSpeed(n), skipToEvent(), dispose() }`.

**Art.** `art/manifest.json` ids: `hero_fighter hero_rogue` + the 15 monsters + 3 bosses
(keyed watercolour, 1024 px masters → 96 px packed, ART.md register); tiles per biome
(`warrens fens crypt` × `floor wall door stairs water chasm`, 8×8 hand-authored or
CC0-derived, ≤ 8 colours each); overlays gas/fire (8×8 animated 2 frames); `title`. Codex
generation, `pack.py`, `art-qc.py`.

## Verification

`tools/verify.sh`: `cargo test` → `cargo clippy -D warnings` → `wasm-pack build` → `tsc` →
`vite build` → `copy-lint` → `node tools/gates.mjs` (runs `metrics`). Green = cut done.

## Addendum A — Companions (added 2026-09-16, in scope for Cut 1)

Design: `PLAN.md` → "Companions (the collector layer)". Additive to everything above.

**Rules.**
- New item `leash` (misc, found D2+, ~1 per 2 floors, stacks). New verb `tame(tag:T|nearest)`:
  executable only if a `leash` is held and the target foe has `hp < 25%` of max; consumes
  the leash and the turn. Chance = 20% + 10% per known tag of that monster kind (cap 60%);
  bosses and summons cannot be tamed. Success: foe becomes a companion entity
  (`ally: true`, kind unchanged) that follows the hero; `fact` event `tamed:<kind>`;
  chronicle note. Failure: foe gets a free attack.
- Party: hero + companions in `lineage.party` (max `party_slots`, 1 at start, 2 by unlock).
  A companion has `level` (1–5), `tags`, and its own `RuleSet` (`max_rows = 1 + level`).
  Companion verbs are its tags: `ranged`→`shoot`, `gas`→`burst` (self-destruct into gas
  3×3), `thief`→`steal` (takes a foe's next drop), `splitter`→`split` (if hp > 50%), `pack`→
  `flank`, `undead`→`drain`, default `attack`, plus `follow` (chore), `recall` (leaves the
  floor, safe). Companion condition tokens are the same set with `self_hp<N` replacing `hp<N`.
- Hero scope token `party:<kind>` prefixes a row that targets that companion with verb
  `recall` or `send` (companion engages nearest foe). Companion AI: its own rows top-down;
  fallback = stay within 2 tiles of the hero, attack adjacent foes.
- Exits: bank → companions return, `level += 1` (cap 5). return → unchanged. Companion
  death → removed from the run and from `party`; an `Egg { tags, kind, gen }` is appended to
  `lineage.eggs`; `Ev::ally{state:"lost"}`; chronicle note. Hero death → every companion in
  the run becomes an egg likewise.
- Breeding (camp, engine method `breed(a_id, b_id)`): both level ≥ 2, both consumed; egg =
  kind of A, tags = tags(A) ∪ one tag of B not in A, `gen = max(gen)+1`, tag count ≤ 3.
- Eggs hatch after 5 completed expeditions (`egg.hatch_in` decremented per exit); hatched
  → companion at level 1 in `lineage.kennel` (roster; not in party until chosen).
  `hatch(egg_id)` for a lost companion's egg costs 2 marks instead of expeditions.
- Counters, learnable as facts `counter:<a>>b`: ranged>heavy, pack>lone, gas>pack,
  water>fire, undead>poison (immune). Effect: ×1.5 damage (or immunity). Applies to all
  entities.
- Ledger: per monster kind `seen known tamed bred`. Trophy `ledger:<biome>` when every kind
  in the biome is `tamed`.
- Gate PETS: DEFAULT rules + two level-3 bred companions with default rows must die by
  ≤ D8 on ≥ 80% of seeds.

**Wire additions.**
```ts
export type Companion = { id: number; kind: string; name: string; level: number; tags: string[]; gen: number;
                          rules: RuleSet; max_rows: number; hp: number; max_hp: number };
export type Egg = { id: number; kind: string; tags: string[]; gen: number; hatch_in: number; from_loss: boolean };
export type LedgerRow = { kind: string; seen: boolean; known: boolean; tamed: boolean; bred: boolean };
// Lineage gains: party: Companion[]; kennel: Companion[]; eggs: Egg[]; party_slots: number; ledger: LedgerRow[]
// Snapshot.entities: companions appear with ally:true and a `cid` field (companion id)
// Ev additions:
//   | { t; k: "tame"; id: number; kind: string; ok: boolean }
//   | { t; k: "hatch"; kind: string }
// ReturnReport gains: tamed: string[]; hatched: string[]; lost: string[]
// Engine gains:
//   setParty(ids: number[]): Lineage;  setCompanionRules(id: number, set: RuleSet): void;
//   breed(a: number, b: number): Lineage;  hatch(eggId: number): Lineage;  companionVocabulary(id: number): Vocabulary;
```
Vocabulary additions: cond `party_hp<N` (any companion), verb `tame`, scope token
`party:<kind>` exposed as a cond `{k:"party",t:"<kind>"}` for editor simplicity. Unlocks:
`party_slot_2` (4 marks), `tame` verb for both classes (2 marks; hero must have the fact
`item:leash` first).

## Addendum B — Gold and supplies (added 2026-09-16, in scope for Cut 1)

Gold is the common currency; marks stay rare (new bests only, vocabulary only).
- `loot` (gold) kept per exit tier is added to `lineage.gold`.
- Camp **supplies**: `buySupply(kind)` at fixed prices, max 3 supply items per expedition,
  placed into the starting inventory alongside vault items. Catalogue: `leash` 30,
  any *identified* potion kind 40, any *identified* scroll kind 60 (identified = fact
  `item:*=<kind>` exists). Supplies are per-run consumables; they are never kept back.
- `hatch(eggId)` for `from_loss` eggs costs **50 gold** (not marks).
- Wire: `Lineage.gold: number`, `Lineage.supplies: InvItem[]` (this run's picks),
  `Engine.buySupply(kind: string): Lineage`, `Engine.clearSupplies(): Lineage`,
  `Vocabulary`-like `Engine.supplyCatalogue(): { kind: string; price: number; label: string }[]`.
- No other gold sink. Gold never buys stats, rows, or unlocks.

## Addendum C — Class XP (added 2026-09-16, in scope for Cut 1)

Persistent per-class levels. `lineage.classes: { [class]: { level: number; xp: number } }`,
levels 1–10 (v1). XP per expedition = Σ kills × (1 + depth/5) + 5 × max depth reached, kept
by exit tier (100/60/30%). Level thresholds: `xp_to_next(level) = 40 × level²`.
- **Verbs by level** (each becomes a rule token; classes gate them):
  fighter L1 `shield_bash`, L3 `cleave` (hit all adjacent, cooldown 6), L5 `taunt` (foes in
  view target hero 3 turns; companions safe), L7 `second_wind` (heal 30% once per floor),
  L9 `bulwark` (def +3 for 3 turns, cooldown 8).
  rogue L1 `vanish`, L1 `throw`, L3 `backstab` (2× dmg on an unaware or stunned foe), L5
  `smoke` (darkness 2×2, 3 turns), L7 `ambush` (first hit from vanish 3×), L9 `shadowstep`
  (blink 3 tiles behind target).
- **Stats by level**: `max_hp += 2` per level; `atk_min, atk_max += 1` at L3, L6, L9.
- **L10 mastery**: trophy `master:<class>` (2 marks) and a class-unique tactic card
  (fighter `phalanx`: back_corridor + taunt bundle; rogue `hit_and_fade`: backstab + vanish).
- Wire: `Lineage.classes` as above; `ReturnReport.xp: { class: string; gained: number;
  level_ups: number }`; `Ev { k: "level", class, level }` emitted at exit when a level is
  reached. Vocabulary for the active class must include verbs up to its level.
- Gate LEVELLED: DEFAULT rules at class L10 must die by ≤ D9 on ≥ 80% of seeds.
