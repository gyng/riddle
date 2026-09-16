// Wire types. Transcribed from docs/CUT1.md ("Wire types"); Rust mirrors with serde, snake_case JSON.
// Do not edit here without editing the contract.

export type Cond = { k: string; n?: number; t?: string };          // {k:"hp<",n:40} {k:"foe_tag",t:"pack"}
export type Verb = { v: string; a?: string };                      // {v:"drink",a:"heal"} {v:"attack",a:"tag:caster"}
export type Row  = { conds: Cond[]; verb: Verb };
export type RuleSet = { rows: Row[]; name?: string };

export type Vocabulary = { conds: Cond[]; verbs: Verb[]; max_rows: number };  // what the editor may offer

export type Tile = "floor"|"wall"|"door"|"stairs_down"|"stairs_up"|"water"|"chasm";
export type Overlay = { x: number; y: number; k: "gas"|"fire"; ttl: number };
export type Entity = { id: number; kind: string; name?: string; x: number; y: number;
                       hp: number; max_hp: number; tags: string[]; ally?: boolean; telegraph?: string;
                       cid?: number };                                   // Addendum A: companions carry their companion id
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
  | { t: number; k: "callout"; text: string }                               // ≤ 3 words, for the renderer
  | { t: number; k: "tame"; id: number; kind: string; ok: boolean }         // Addendum A
  | { t: number; k: "hatch"; kind: string }                                 // Addendum A
  | { t: number; k: "level"; class: string; level: number }                 // Addendum C
  | { t: number; k: "rank"; rank: number }                                  // Addendum D
  | { t: number; k: "projectile"; src: number; dst: number; path: [number, number][] }; // Addendum E: 1 tile per tick along path

export type StepResult = { events: Ev[]; snapshot: Snapshot; run_over: boolean;
                           exit_pending?: { items: InvItem[]; tier: string } };                                       // Addendum D

export type Forecast = { depths: { depth: number; reach: number }[]; causes: { cause: string; share: number }[];
                         known_to: number };
export type Trace = { turns: { t: number; row: number; verb: Verb; hp: number; foes: number; telegraphs: string[] }[] };
export type Death = { run_id: number; depth: number; cause: string; margin: string; verdict: "gap"|"dice";
                      baseline: number;                                                   // core addition: survival of the unpatched rules, 0..1
                      trace: Trace; patches: { row: Row; insert_at: number; survive: number; forecast_delta: number }[];
                      morgue: string };
// Fractions: Forecast.depths[].reach, causes[].share, Death.baseline, patches[].survive and forecast_delta are 0..1.
export type Highlight = { pattern: string; score: number; t: number; run_id: number; text: string };
export type ReturnReport = {
  elapsed_s: number; runs: number; sampled: boolean;
  learned: string[]; bests: string[]; found: InvItem[]; deaths: { cause: string; n: number }[];
  pending: string[]; reel: Highlight[]; marks_earned: number; worst_death?: Death; worst_death_id?: number; live: Snapshot;
  tamed: string[]; hatched: string[]; lost: string[];                        // Addendum A
  xp: { class: string; gained: number; level_ups: number };                 // Addendum C
  salvaged: { kind: string; n: number; gold: number }[];                    // Addendum D
  renown: { gained: number; rank: number; ranks_up: number };               // Addendum D
};
export type Lineage = { seed: number; heir: number; trait: string; class: string; best_depth: number; marks: number;
                        facts: string[]; unlocks: string[]; vault: InvItem[]; graveyard: { heir: number; depth: number; cause: string; deeds: string[] }[];
                        trophies: string[]; sets: RuleSet[]; active_set: number; ended: boolean;
                        party: Companion[]; kennel: Companion[]; eggs: Egg[]; party_slots: number; ledger: LedgerRow[];  // Addendum A
                        gold: number; supplies: InvItem[]; insured?: number[];                                                              // Addendum B
                        classes: { [cls: string]: { level: number; xp: number } };                                     // Addendum C
                        forge: { [kind: string]: { salvaged: number; craftable: boolean; tier: number } };               // Addendum D
                        renown: number; rank: number; keep_pref: string };                                              // Addendum D

// Addendum A — Companions
export type Companion = { id: number; kind: string; name: string; level: number; tags: string[]; gen: number;
                          rules: RuleSet; max_rows: number; hp: number; max_hp: number };
export type Egg = { id: number; kind: string; tags: string[]; gen: number; hatch_in: number; from_loss: boolean };
export type LedgerRow = { kind: string; seen: boolean; known: boolean; tamed: boolean; bred: boolean };

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
  buy(unlock: string): Lineage;
  lineage(): Lineage;
  exportRules(): string;  importRules(text: string): RuleSet;
  // Addendum A
  setParty(ids: number[]): Lineage;  setCompanionRules(id: number, set: RuleSet): void;
  breed(a: number, b: number): Lineage;  hatch(eggId: number): Lineage;  companionVocabulary(id: number): Vocabulary;
  // Addendum B
  buySupply(kind: string): Lineage;  clearSupplies(): Lineage;  supplyCatalogue(): SupplyEntry[];
  // Addendum D
  keep(ids: number[]): Lineage;
  setKeepPref(pref: string): Lineage;   // core addition (README): keep preference for offline exits
  insure(id: number): Lineage;          // core addition: gold bet that keeps a brought vault item on death
  // core additions (crates/riddle-core/README.md)
  unlocks(): UnlockInfo[];              // the catalogue; `available` = prereq + fact gate + affordable
  setClass(cls: string): Lineage;       // switch class (rogue needs the `rogue` unlock)
  selectSet(i: number): Lineage;        // pick one of the three saved sets; setRules writes the active one
}
export type UnlockInfo = { id: string; cost: number; owned: boolean; available: boolean };

/** The Engine with every method returning a Promise: the wasm engine lives in a Web Worker. */
export type AsyncEngine = { [K in keyof Engine]: Engine[K] extends (...a: infer A) => infer R ? (...a: A) => Promise<R> : never };
export type SupplyEntry = { kind: string; price: number; label: string };                                             // Addendum B
