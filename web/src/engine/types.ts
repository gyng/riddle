// Wire types. Transcribed from docs/CUT1.md ("Wire types") and docs/CUT2.md ("Wire additions"); Rust mirrors
// with serde, snake_case JSON. Do not edit here without editing the contract. Cut 2 fields are optional on the
// client so the UI runs against a Cut 1 core.

export type Cond = { k: string; n?: number; t?: string };          // {k:"hp<",n:40} {k:"foe_tag",t:"pack"}
export type Verb = { v: string; a?: string };                      // {v:"drink",a:"heal"} {v:"attack",a:"tag:caster"}
/** Cut 7 §2 — where a row came from (optional; the core may tag, else the client infers: the shipped rows at boot are
 *  `preset`, `applyPatch` rows `patch`, bought card rows `card`, anything the player adds or edits a token of `player`). */
export type RowOrigin = "preset" | "patch" | "card" | "player";
export type Row  = { conds: Cond[]; verb: Verb; origin?: RowOrigin };
export type RuleSet = { rows: Row[]; name?: string };

export type Vocabulary = { conds: Cond[]; verbs: Verb[]; max_rows: number;   // max_rows: Cut 12 §1, the cap on the player's OWN rows; card rows (verb.v === "tactic") sit outside it, one per owned card
                           combos?: Combo[];                                // Cut 8B §1: the combo table (adjacent-row verb pairs the engine names)
                           locked?: LockedCond[] };                         // Cut 9 §1: gated conds the sheet shows dim with their gate, never selectable
/** Cut 9 §1 — a condition the lineage cannot use yet and the gate as the player reads it (`foe: any`, `◆2`). */
export type LockedCond = { cond: Cond; needs: string };
/** Cut 8B §1 — a combo: verb patterns for two adjacent rows (`shield_bash`, or `drink unknown` for one argument) and the name the
 *  engine gives the pair (`opener`, `hit and fade`, `bait`). Engine data, never player-written. */
export type Combo = { a: string; b: string; name: string };
/** Cut 8B §1 — a combo found in the active set: the two adjacent row indices (0-based) and its name. */
export type ComboHit = { rows: [number, number]; name: string };

export type Tile = "floor"|"wall"|"door"|"stairs_down"|"stairs_up"|"water"|"chasm"
                 | "shrine"|"vault"|"vault_open"|"nest";                 // Cut 5 §4 situations: floor-standing props (renderer tolerates them absent)
export type Overlay = { x: number; y: number; k: "gas"|"fire"; ttl: number };
export type Entity = { id: number; kind: string; name?: string; x: number; y: number;
                       hp: number; max_hp: number; tags: string[]; ally?: boolean; telegraph?: string;
                       cid?: number;                                    // Addendum A: companions carry their companion id
                       remembered?: boolean };                          // Cut 4 §3: pursued but unseen, at its last seen tile (drawn dimmed)
export type FloorItem = { id: number; x: number; y: number; kind: string; known: boolean; label: string };
export type Snapshot = {
  depth: number; biome: string; w: number; h: number; tiles: Tile[]; seen: boolean[]; visible: boolean[];
  overlays: Overlay[]; hero: Entity & { inv: InvItem[]; weapon?: string; armour?: string; class: string; trait: string };
  entities: Entity[]; items: FloorItem[]; alert: number; turn: number; loot: number;
  run: { id: number; heir: number; started_turn: number };
  stake?: Stake;                                                          // Cut 2 §7: what is on the line right now
  vision?: number;                                                        // Cut 3: the hero's sight radius on this floor (Deep 4, else 7; +2 lantern)
  vault_choice?: VaultChoice;                                             // Cut 5 §4: an opened vault waiting for `choose(itemId)` (50-tick grace, then `vault_pref` picks)
  room?: { id: number; hostiles: number };                                // Cut 7 §4: the room the hero is in (0 = corridor); awake hostiles in it (optional)
  rooms?: number;                                                         // Cut 7 §4: rooms on this floor, for `D3 · 4 rooms` (optional)
  floor_twist?: string;                                                   // Cut 12 §4: the floor's one situation, one word (`nest`), for `D4 · 9 rooms · a nest` (optional; absent on D1–D2)
};
/** Cut 5 §4 — the three items of an opened vault; `choose(id)` takes one, the rest vanish. */
export type VaultChoice = { items: InvItem[]; left?: number };   // Cut 14: ticks of the 50-tick grace left at this snapshot (the sheet's shrinking bar)
/** Cut 2 §7 — loot on the hero, brought items (insured = kept on death), the row that would bank/return if any.
 *  Cut 6 §1: `kept` = what that row would bring home now (`$84 · keeps $50`). */
export type Stake = { loot: number; brought: { label: string; insured: boolean }[]; return_row?: number; kept?: number;
                      stalling?: boolean };                                          // Cut 13 §1: the guard has fired this floor — a stall pays nothing (`keeps $0 · stalling`)
/** Cut 6 §1 — the ledger line of an exit: one arithmetic line the player can check, `text` is shown verbatim
 *  (`$84 carried · return keeps 60% → $50 · supplies −$12 → $68`). Fractions: `keep_pct` 0..100. */
export type ExitLine = { carried: number; keep_pct: number; kept: number; spent: number; spent_on: string[]; text: string;
                         trace?: Trace;                                                                     // Cut 9 §5: the exit's last-5 trace (every tier)
                         salvaged?: { kind: string; n: number; gold: number }[];                           // what the exit salvaged before the keep sheet (a return's 40 % cut), per kind in coins
                         run_id?: number };                                                                 // the run, so a report's trace links can open its replay (QA on e0f87e7: the return sheet's had `watch`, the report's did not)
/** Cut 6 §1 — one gold movement in the camp's `gold` sheet: `+$50 returned D5`, `−$40 heal`, `−$8 insure sword`. */
export type GoldLine = { t: number; delta: number; why: string };
/** Cut 6 §5 — a boss whose counter is a known row (`attack boss`, `throw fire, boss`, `read silence`). */
export type Counter = { boss: string; row?: Row | string; text: string };
export type InvItem = { id: number; kind: string; known: boolean; label: string; hint?: "benevolent"|"malevolent";
                        free?: boolean };                                   // Cut 12 §6: a supply the camp gave (the kennel's leash) reads `leash · kennel`; the core always sends it

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
  | { t: number; k: "steal"; id: number; item: string; amount?: number }   // amount: Cut 10 §3, gold stolen (`stolen $16`)
  | { t: number; k: "ally"; id: number; state: "freed"|"lost" }
  | { t: number; k: "descend"; depth: number; biome: string }
  | { t: number; k: "exit"; tier: "bank"|"return"|"death"; loot_kept: number; line?: ExitLine; trace?: Trace }   // line: Cut 6 §1; trace: Cut 9 §5
  | { t: number; k: "note"; text: string }                                  // chronicle line, ≤ 8 words
  | { t: number; k: "callout"; text: string }                               // ≤ 3 words, for the renderer
  | { t: number; k: "tame"; id: number; kind: string; ok: boolean }         // Addendum A
  | { t: number; k: "hatch"; kind: string }                                 // Addendum A
  | { t: number; k: "level"; class: string; level: number }                 // Addendum C
  | { t: number; k: "rank"; rank: number }                                  // Addendum D
  | { t: number; k: "projectile"; src: number; dst: number; path: [number, number][] } // Addendum E: 1 tile per tick along path
  | { t: number; k: "rest"; seconds: number }                               // Cut 2 §1: emitted at exit; the viewer shows `rest Nm`
  | { t: number; k: "bones"; heir: number; items: number }                  // Cut 2 §2: a bones pile (left on death, or recovered by a later heir)
  | { t: number; k: "see"; id: number; e?: Entity }                         // Cut 2 §7: first sight of an entity (renderer flashes on a boss); not emitted by the core yet
  | { t: number; k: "ending"; ticks: number };                              // Cut 7 §4: the last `ticks` before an exit start here (optional; else the client infers exit − 30)

export type StepResult = { events: Ev[]; snapshot: Snapshot; run_over: boolean;
                           exit_pending?: { items: InvItem[]; tier: string; worth?: number[]; auto_keep?: number[] } };                     // Addendum D; `worth`: each item's salvage at this exit, in coins

export type Forecast = { depths: { depth: number; reach: number; cause?: string; pm?: number; try?: ForecastTry; wall?: string }[]; causes: { cause: string; share: number }[];
                         known_to: number;                               // depths[].cause: Cut 4 §8, optional per-depth top cause; pm: Cut 9 §3, the binomial half-width (`D4 71% ±6`); wall: Cut 18 §3, the sealing boss's kind where reach falls to ≤ 5 % below his floor (`D9 0% · warlord wall`)
                         ends?: { bank: number; return: number; death: number; stall?: number; gold: number; pm?: number };
                         refined?: boolean;
                         shadowed_by?: (number | null)[] };                                                    // QA 92eb880: per row of the set (by index), the earlier row (0-based) that takes every moment it could fire — mark it `shadowed by R{n+1}`; null = free; absent = none shadowed   // Cut 13 §5: the refine pass (100 sims); a first paint is marked `…`   // Cut 12 §3: how a send ends (rates 0..1 summing to 1; `stall`: came home by the cap, nothing in the rules) and the mean gold brought home per send
/** Cut 10 §2 — a boss floor whose counter fact is known and whose row is absent from the set: `D9 0% · warlord · try: attack boss`;
 *  tapping the bar inserts `row` at the top (optional on the wire; the client derives it from `Lineage.counters` when absent). */
export type ForecastTry = { row: Row; text: string; boss?: string };
/** Cut 4: `blocked` = the first row whose conds held but whose verb could not execute. Cut 6 §3: `rows` = every row above the
 *  fired one with one reason why it did not fire (`none held`, `no path`, `not in view`, `hp 8% ≥ 30%`). */
export type TraceTurn = { t: number; row: number; verb: Verb; hp: number; foes: number; telegraphs: string[];
                          blocked?: string; rows?: { row: number; why: string; because?: Because }[] };   // because: Cut 11 §1
/** Cut 11 §1 — why a state reason held: the most recent event that put it there (`den took the heal, D3`, ≤ 8 words),
 *  its tick and floor. The client scrubs the run's replay to `t` when it still holds the run's events. */
export type Because = { text: string; t: number; depth: number };
export type Trace = { turns: TraceTurn[];
                      provenance?: Because[] };                                              // Cut 11 §3: every `because` event of the run (exit traces)
/** A candidate row. Death patches insert before `insert_at`; stall patches (core README) may instead `replace` the row at
 *  `insert_at` or `remove` it (`row` echoes the removed row).
 *  Cut 11 §2: `root` names the chain's root the row answers (`den took the heal`); `insert_at: -1` is an unlock pseudo-patch
 *  (`root.text` = `◆2 cond: alert`; the client buys the cond's unlock, then inserts the row at the top); `below_bar` marks a
 *  §4 candidate under the verdict bar (`survives 40% · below bar`), shown dimmed. */
export type Patch = { row: Row; insert_at: number; survive: number; forecast_delta: number; replace?: boolean; remove?: boolean;
                      root?: { text: string }; below_bar?: boolean;
                      forecast_depth?: number; forecast_pm?: number;                         // QA 23ed91f: the camp bar `forecast_delta` moves (death's depth + 1, ≤ known_to) and its 95 % ± — set by `deathDeltas`
                      camp_pending?: boolean;                                                // QA 23ed91f: on `death()`'s patches — `forecast_delta` is the verdict's 12-sim ranking estimate, NOT the camp's; paint reach as pending until `deathDeltas(id)` lands
                      unlock?: string };                                                     // optional: the pseudo-patch's unlock id (else derived from the row's cond)
export type Death = { run_id: number; depth: number; cause: string; margin: string; verdict: "gap"|"dice"|"stall";   // stall: Cut 13 §1, a stalled run's verdict (client-side widening; the Rust side is a String)
                      baseline: number;                                                   // core addition: survival of the unpatched rules, 0..1
                      trace: Trace; patches: Patch[];
                      morgue: string;
                      line?: ExitLine;                                                       // Cut 6 §1: the death's ledger line
                      chain?: Because[];                                                     // Cut 11 §2: the death's chain, root first (the rows' `because`s, deduplicated)
                      rules?: RuleSet;                                                       // the rules the run died under (the accounting's row labels; else the morgue's lines)
                      notes?: string[];                                                      // Cut 13 §4: the run's last two chronicle notes, under the headline (never a `saved him` — QA 92eb880)
                      nothing_beats_base?: boolean };                                        // QA 92eb880: a `dice` death none of whose patches survives more than `baseline` (a 100 % base: the replays win the fight he lost) — say `nothing beats base · base N%`, not `below bar`
/** Core addition: the last ≥ 4 runs all came home with no new depth — the row that ended them, how many, a ≤ 12-word line,
 *  and up to 3 patches with forecast deltas at the stall depth + 1 (`survive` = the patched reach there). A state: the
 *  last slice's wins on merge. */
export type Stall = { row: number; fired: number; text: string; patches: Patch[]; trace?: Trace };   // trace: Cut 9 §5, the last run that row ended
// Fractions: Forecast.depths[].reach, causes[].share, Death.baseline, patches[].survive and forecast_delta are 0..1.
export type Highlight = { pattern: string; score: number; t: number; run_id: number; text: string };
export type ReturnReport = {
  elapsed_s: number; runs: number; sampled: boolean;
  deepest?: number;                                                            // the send's deepest floor (a delta, like the tiles beside it); absent on an old wire
  stalled?: number;                                                            // Cut 13 §1: sends that stalled (among `returned`, keeping nothing); the tiles count them apart
  spent?: { kind: string; n: number; gold: number }[];                         // Cut 13 §3: what the automations bought this absence, per kind (the SPENT section)
  gold?: { home: number; salvage: number; wake: number; spent: number };       // Cut 13 §3: the absence's movements to the coin (home + salvage + wake − spent = the header's delta)
  learned: string[]; bests: string[]; found: InvItem[]; deaths: { cause: string; n: number }[];
  pending: string[]; reel: Highlight[]; marks_earned: number; worst_death?: Death; worst_death_id?: number; live?: Snapshot;
  tamed: string[]; hatched: string[]; lost: string[];                        // Addendum A
  xp: { class: string; gained: number; level_ups: number };                 // Addendum C
  salvaged: { kind: string; n: number; gold: number }[];                    // Addendum D
  renown: { gained: number; rank: number; ranks_up: number };               // Addendum D
  rested_s?: number; banked?: number; returned?: number; bones_found?: string[]; // Cut 2 §1–2
  stall?: Stall;                                                              // core addition: stall verdict
  exits?: ExitLine[];                                                         // Cut 6 §1: one ledger line per exit in the batch
  picked?: number[];                                                          // Cut 16 §1: depths picked clean (≥ 3 banks/returns, shallower than the best), ascending — `D3 · picked clean`
};
export type Lineage = { seed: number; heir: number; trait: string; trait_offer?: string[]; class: string; best_depth: number; marks: number;   // Cut 13 §2: `trait_offer` — two traits a new heir may wake with; `setTrait(name)` picks
                        facts: string[]; unlocks: string[]; vault: InvItem[];
                        graveyard: { heir: number; depth: number; cause: string; deeds: string[]; death_id?: number }[];   // death_id: Cut 9 §7, a kept death (`death(id)` answers)
                        trophies: string[]; sets: RuleSet[]; active_set: number; ended: boolean;
                        party: Companion[]; kennel: Companion[]; eggs: Egg[]; party_slots: number; ledger: LedgerRow[];  // Addendum A
                        gold: number; supplies: InvItem[]; insured?: number[];                                                              // Addendum B
                        classes: { [cls: string]: { level: number; xp: number } };                                     // Addendum C
                        forge: { [kind: string]: { salvaged: number; craftable: boolean; tier: number;
                                                   next?: { need: number; label: string } } };                         // Addendum D; next: Cut 9 §10, the ladder's next rung (`3/5 → craftable`)
                        renown: number; rank: number; keep_pref: string;                                               // Addendum D
                        keep_auto?: string[];                                                                          // QA 23ed91f: what an unwatched exit keeps, in order, after the brought vault items (`["armour"]`, `["armour","weapon"]` with quartermaster, `[]` for none); a keep replaces only a weaker vault item of its own category
                        rest_left_s?: number; bones?: BonesPile[];                                                      // Cut 2 §1–2
                        ascension?: Ascension;                                                                         // Cut 3
                        chronicle?: string[];                                                                          // Cut 5 §2: one line per ended heir, oldest first (cap 40)
                        vault_pref?: string;                                                                           // Cut 5 §4: what an unwatched vault choice takes (`weapon | armour | potion | scroll`)
                        ascended?: string[];                                                                          // Cut 5: variants the lineage has finished the dungeon with
                        gold_ledger?: GoldLine[];                                                                      // Cut 6 §1: the last 20 gold movements, oldest first (`ledger` is the bestiary)
                        counters?: Counter[];                                                                         // Cut 6 §5: bosses whose counter row is known
                        combos?: ComboHit[];                                                                          // Cut 8B §1: the active set's combos, in row order (recomputed on setRules)
                        picked?: number[];                                                                            // Cut 16 §1: depths picked clean now (as ReturnReport.picked)
                        class_offer?: ClassChip[];                                                                    // Cut 16 §2: the wake's class chips — owned classes, current first; present while trait_offer is and ≥ 2 are owned; `setClass(name)` picks (sticks until changed)
                        shadowed_by?: (number | null)[] };                                                            // QA 92eb880: the active set's shadowed rows, as Forecast.shadowed_by (absent when none)
/** Cut 16 §2: a class chip at the wake (`rogue · vanish`). `signature` is a verb id (`shield_bash | vanish | mark | slow`);
 *  `level` the class's level; `opens` the level the signature opens at (`mark L7` while level < opens).
 *  §4 (no new wire): the Warlord's break is a callout `warlord breaks` + a note `The Warlord breaks.` (visible only), once, at ≤ 50 % hp.
 *  §3 (no new wire): D5–8 are biome `burrows` (Snapshot.biome, descend.biome, fact `biome:burrows`). */
export type ClassChip = { class: string; signature: string; level: number; opens: number };
/** Cut 3: times the lineage ascended and the variant it plays under (`""` at level 0). */
export type Ascension = { level: number; variant: string };
export const VARIANTS = ["no_rest", "short_list", "bones_only", "hunted"] as const;
/** Cut 2 §2 — a dead heir's kit on the floor (per lineage, max 3; oldest expires). */
export type BonesPile = { depth: number; heir: number; items: number };

// Addendum A — Companions
export type Companion = { id: number; kind: string; name: string; level: number; tags: string[]; gen: number;
                          rules: RuleSet; max_rows: number; hp: number; max_hp: number };
export type Egg = { id: number; kind: string; tags: string[]; gen: number; hatch_in: number; from_loss: boolean };
export type LedgerRow = { kind: string; seen: boolean; known: boolean; tamed: boolean; bred: boolean; studied?: boolean;   // studied: Cut 2 §5
                          counter?: { row: Row; text: string } };                                                         // Cut 7 §1: a boss's known counter as a row (a chip on the bestiary card)

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
  runOfflineQuick(elapsedS: number): ReturnReport;   // no worst-death verdict (~3 s saved per slice)
  death(runId: number): Death;
  /** QA 23ed91f: the death's shown patches (same order as `death(id).patches`) with the camp's own reach: `forecast_delta` is
   *  the camp forecast's bar move at `forecast_depth` once the patch is applied (same insert, same lineage state), `forecast_pm`
   *  that bar's ±, `camp_pending` cleared. Slow (four 50-sim camp panels, seconds in wasm): call after painting `death(id)`.
   *  Memoised; the camp's next `forecast()` for the base or the tapped patch's set is then a cache hit. Optional on old builds. */
  deathDeltas?(runId: number): Patch[];
  buy(unlock: string): Lineage;
  buyUnlockGold?(unlock: string): Lineage;   // Cut 15 §2: the same gates as `buy`, paid in gold at `UnlockInfo.gold` (marks untouched, ledger `unlock <id>`, the next gold price climbs); absent on an old build
  lineage(): Lineage;
  exportRules(): string;  importRules(text: string): RuleSet;
  // Addendum A
  setParty(ids: number[]): Lineage;  setCompanionRules(id: number, set: RuleSet): void;
  breed(a: number, b: number): Lineage;  hatch(eggId: number): Lineage;  companionVocabulary(id: number): Vocabulary;
  // Addendum B
  buySupply(kind: string): Lineage;  clearSupplies(): Lineage;  supplyCatalogue(): SupplyEntry[];
  dropSupply?(id: number): Lineage;     // Cut 12 §6 (client-proposed, core item): the supplies `×` removes one line; absent → the client clears and rebuys the rest
  setTrait?(name: string): Lineage;     // Cut 13 §2: pick one of `Lineage.trait_offer` for the new heir (optional; an older core has no offer)
  // Addendum D
  keep(ids: number[]): Lineage;
  /** QA 23ed91f (L: `auto: keep weapon+armour` owned, vault full → the skipped sheet's `keep([])` salvaged mail · axe · sword):
   *  resolve the pending exit by the keep preference and owned automations — `exit_pending.auto_keep` (brought vault items
   *  first; a keep replaces only a weaker vault item of its own category). The skipped sheet's call; `keep([])` keeps nothing.
   *  Optional on old builds. */
  autoKeep?(): Lineage;
  setKeepPref(pref: string): Lineage;   // core addition (README): keep preference for offline exits
  insure(id: number): Lineage;          // core addition: gold bet that keeps a brought vault item on death
  // core additions (crates/riddle-core/README.md)
  unlocks(): UnlockInfo[];              // the catalogue; `available` = prereq + fact gate + affordable
  unlockDeltas(): UnlockInfo[];         // Cut 4: same catalogue with forecast `delta` computed (0.3–2 s); call after paint
  setClass(cls: string): Lineage;       // switch class (rogue needs the `rogue` unlock)
  selectSet(i: number): Lineage;        // pick one of the three saved sets; setRules writes the active one
  ascend(variant: string): Lineage;     // Cut 3: after the ending, a new lineage under a variant (keeps classes, kennel, vault, facts, rules)
  // Cut 5
  bail(): void;                         // §5: a `return` fires on the hero's next action as a chore; the rules are untouched
  choose(itemId: number): Snapshot;     // §4: take one item of the opened vault (`Snapshot.vault_choice.items[].id`)
  setVaultPref(pref: string): Lineage;  // §4: `weapon | armour | potion | scroll` — what an unanswered vault choice takes
  // Cut 6
  forecastRefine?(): Forecast;          // §9: the same forecast at 100 sims (optional; the client calls it 2 s after a quiet paint)
}
export type UnlockInfo = { id: string; cost: number; owned: boolean; available: boolean; needs?: string;   // needs: Cut 2 §3, the gate still missing (absent once met)
                           delta?: number;                                                                 // Cut 4 §9: forecast reach delta of buying (0..1), tactic cards
                           rows?: Row[];                                                                   // Cut 6 §6: a card's rows / an automation's effect as a row
                           insert_at?: number;                                                             // Cut 12 §1: where a bought card's row goes; Cut 18 §5: with deltas, its best measured place (the old place before the engagement row, the top, before the first own row) — its `delta` is measured there
                           pm?: number;                                                                     // Cut 13 §5: the half-width of `delta`; within it the client reads `reach ~0`
                           situation?: string;                                                             // Cut 18 §5: a tactic card's foe tag (`kite_archers` → `ranged`, `gas_step` → `gas`), for `vs archers` beside `reach ~0`; absent on other unlocks
                           gold?: number };                                                                 // Cut 15 §2: today's gold price (`150 × cost × (4 + gold buys) / 4`); 0 when owned or free (not gold-buyable). A card short only of marks (`needs` = `◆N more`) buys with gold when the lineage has it

/** The Engine with every method returning a Promise: the wasm engine lives in a Web Worker. */
export type AsyncEngine = { [K in keyof Engine]: NonNullable<Engine[K]> extends (...a: infer A) => infer R ? (...a: A) => Promise<R> : never };
export type SupplyEntry = { kind: string; price: number; label: string;
                            needs?: string };                                                                         // Addendum B; needs: Cut 10 §3, why a supply is greyed (`◆ identify`), optional
