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
