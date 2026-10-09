// Wire types. Transcribed from docs/CUT1.md ("Wire types") and docs/CUT2.md ("Wire additions"); Rust mirrors
// with serde, snake_case JSON. Do not edit here without editing the contract. Cut 2 fields are optional on the
// client so the UI runs against a Cut 1 core.

export type Cond = { k: string; n?: number; t?: string };          // {k:"hp<",n:40} {k:"foe_tag",t:"pack"}
export type Verb = { v: string; a?: string };                      // {v:"drink",a:"heal"} {v:"attack",a:"tag:caster"}
/** Cut 7 §2 — where a row came from (optional; the core may tag, else the client infers: the shipped rows at boot are
 *  `preset`, `applyPatch` rows `patch`, bought card rows `card`, anything the player adds or edits a token of `player`). */
export type RowOrigin = "preset" | "patch" | "card" | "player"
  | `${"stance" | "tactic" | "temper" | "drill" | "class"}:${string}`;   // Compiled rows sit outside the player row cap.
export type Row  = { conds: Cond[]; verb: Verb; origin?: RowOrigin };
export type RuleSet = { rows: Row[]; name?: string;
                        /** Cut 26 §2 (core): the set's route — the fork depths (5 · 9 · 14 · 19 · 24) whose FAR stair the hero takes; absent/[] = the
                         *  near stair everywhere (the base order). Non-overlapping: a far stair at D5 excludes D9 (the deferred biome is taken there).
                         *  Written with `setRules` (the whole set, rows untouched), exported/imported and restored with sets 1–3 like the rows. The
                         *  core refuses a fork the hero has not seen (`D9 fork unseen`). */
                        route?: number[] };

export type Vocabulary = { conds: Cond[]; verbs: Verb[]; max_rows: number;   // max_rows: Cut 12 §1, the cap on the player's OWN rows; card rows (verb.v === "tactic") sit outside it, one per owned card
                           combos?: Combo[];                                // Cut 8B §1: the combo table (adjacent-row verb pairs the engine names)
                           locked?: LockedCond[];                           // Cut 9 §1: gated conds the sheet shows dim with their gate, never selectable
                           depth_max?: number;                              // Cut 21 §3: the deepest `depth ≥` the picker offers (the lineage best + 2, at least 8)
                           why_gloss?: { [reason: string]: string } };      // Cut 23 §3 (core): every block/why reason the core emits (`no use`, `no target`, `row guard`) → its reason on tap, ≤ 3 words (`nothing to learn`); keys are reason prefixes (longest match wins)
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
/** Cut31 B: immutable birth modifiers; saved separately from learned kind facts. */
export type ModifierInfo = { id:string; mask:number; name:string; effect:string; counter:string };
export type EncounterModifiers = { tier:number; affixes:number; elite?:"shielded"|"frenzied"|"leeching"; tight_mirror?:boolean; affix?:string };   // Cut 116 §1 (core): `affix` — the heir's band-boss affix (`armoured`), at any tier
export type Entity = { modifiers?:EncounterModifiers; id: number; kind: string; name?: string; x: number; y: number;
                       hp: number; max_hp: number; tags: string[]; ally?: boolean; telegraph?: string;
                       cid?: number;                                    // Addendum A: companions carry their companion id
                       remembered?: boolean };                          // Cut 4 §3: pursued but unseen, at its last seen tile (drawn dimmed)
export type FloorItem = { id: number; x: number; y: number; kind: string; known: boolean; label: string };
export type GunSnap = { item:number; kind:string; loaded:number; capacity:number; range:number;
  damage:[number,number]; armour_piercing:number; reload_ticks:number; reload_left:number; reload_until?:number; aiming?:boolean };
/** Take control: one hand-chosen action — a step (a foe on the tile is attacked), a verb as rows write it, or a wait. */
export type ManualAct = { k: "step"; dx: number; dy: number } | { k: "verb"; verb: Verb } | { k: "wait" }
  | { k: "attack_until"; hp: number };   // blind b58b431: one order — the nearest foe, then the same foe, until it falls or hp < `hp` %
export type Snapshot = {
  difficulty?:number; modifier_catalogue?:ModifierInfo[];
  depth: number; biome: string; w: number; h: number; tiles: Tile[]; seen: boolean[]; visible: boolean[];
  overlays: Overlay[]; hero: Entity & { inv: InvItem[]; weapon?: string; armour?: string; class: string; trait: string; specialization?: "sentinel" | "hexbinder"; gun?: GunSnap };
  entities: Entity[]; items: FloorItem[]; alert: number; turn: number; loot: number;
  run: { id: number; heir: number; started_turn: number; start?: number;   // start: QA a946e04 (core, optional) — the floor the run started on (a toll the purse could not pay starts it on D1)
         passage?: number };                                              // Cut 27 §1 (core): the passage paid at a waystone start (`+$84 passage`, coins, into the purse at the send — ledger `passage D9`; a `passage +$84` callout opens the run); absent from D1 / when the set does not clear the floors above ≥ 95 %   // start: QA a946e04 (core, optional) — the floor the run started on (a toll the purse could not pay starts it on D1)
  stake?: Stake;                                                          // Cut 2 §7: what is on the line right now
  vision?: number;                                                        // Cut 3: the hero's sight radius on this floor (Deep 4, else 7; +2 lantern)
  vault_choice?: VaultChoice;                                             // Cut 5 §4: an opened vault waiting for `choose(itemId)` (50-tick grace, then `vault_pref` picks)
  room?: { id: number; hostiles: number };                                // Cut 7 §4: the room the hero is in (0 = corridor); awake hostiles in it (optional)
  rooms?: number;                                                         // Cut 7 §4: rooms on this floor, for `D3 · 4 rooms` (optional)
  floor_twist?: string;                                                   // Cut 12 §4: the floor's one situation, one word (`nest`), for `D4 · 9 rooms · a nest` (optional; absent on D1–D2)
  fork?: SnapFork;                                                        // Cut 26 §2 (core): this floor's down stairs are a fork's (D4 on the base order) — draw a second stair at (x, y); the real stairs are `taken`'s
  meters?: SnapMeters;
  manual?: boolean; awaiting?: boolean; order?: string;   // order (blind b58b431, core): how the last hand order resolved — `moved` · `can't · wall` · `paralysed · 3` · `foe down`
                                    // take control: the player has the hero · the world waits for his action                                                    // Cut 29 §3 (core): the watch's compact meter — the run so far, the fight in progress (or the last), `fighting` while one is; absent before the first tick
};
/** Cut 26 §2 (core) — the fork at this floor's down stairs: `depth` the band's first floor (5 · 9 · 14 · 19 · 24), `taken` the biome the
 *  run's route takes (the floor's own `stairs_down`), `other` the stair not taken, drawn at (`x`, `y`) beside it. The hero never takes
 *  `other`; the callout is `TWO STAIRS` (an ordinary `callout` event, once per run and fork). */
export type SnapFork = { depth: number; taken: string; other: string; x: number; y: number };
/** Cut 26 §2 (core) — a seen fork on the rule set's chip line (the core ships the D5 fork only — `descent::OPEN_FORKS`, Cut 26 §3's
 *  fallback — so `Lineage.forks` holds at most `[5]` today; build for several) (`⑂ D5 fens · D14 crypt`): `near` the band's own biome, `far` the next one
 *  early, `taken` what the active set's route takes. `open: false` — the fork above took its far stair, so this band is the deferred
 *  biome's (`taken`), no choice (dim chip). A route edit is `setRules({...set, route})`; `forkForecast(depth)` prices both stairs. */
export type ForkChip = { depth: number; near: string; far: string; taken: string; open?: boolean };
/** Cut 26 §2 (core) — a lit waystone as a (lane, depth) pair: `lane` the biome there, `route` the far stairs above it it was lit on
 *  (absent = the base order), `current` lit for the active set's route (a send can start there: `Lineage.waystones`). */
export type LaneStone = { depth: number; lane: string; route?: number[]; current?: boolean };
/** Cut 26 §2 (core) — one stair of a fork on the fork chip's tablet (`forkForecast(fork)`), like `StartOption`: the active set with its
 *  route taking `biome` at `fork` (`far`), on the camp's seeds (first pass, paired with the current route's panel). `depth` = the band's
 *  last floor (the boss's) — the chip reads `fens D8 61% · burrows D8 34%`; `delta` = `bank_delta` when either banks, else `reach_delta`;
 *  `route` = the set's route with this stair (setRules it to take it). Fractions 0..1. */
export type ForkOption = { fork: number; biome: string; far: boolean; current: boolean; route: number[]; depth: number;
                           reach: number; reach_delta: number; bank: number; bank_delta: number; gold: number; gold_delta: number;
                           death?: number; death_delta?: number; delta: number; pm: number; refined?: boolean; low?: number };
/** Cut 5 §4 — the three items of an opened vault; `choose(id)` takes one, the rest vanish. */
export type VaultChoice = { items: InvItem[]; left?: number;   // Cut 14: ticks of the 50-tick grace left at this snapshot (the sheet's shrinking bar)
                            pick?: number };                   // Cut 19 §1: the item id `vault_pref` takes when `left` runs out (the beat's `took mail`); `choose(id)` overrides while left > 0
/** Cut 19 §1: one cage preference measured for the active set (`cageForecast()`): the camp panel with `vault_pref` = `pref`, paired
 *  with the current preference's. `depth`: the bar `reach` is read at (the set's bank row's depth, else the lineage best); `delta`:
 *  the picker's headline — `bank_delta` when either panel banks, else `reach_delta`; `pm`: the ± of `reach` (0..1 fractions). */
export type CageOption = { pref: string; current: boolean; depth: number; reach: number; reach_delta: number; bank: number; bank_delta: number;
                           gold: number; gold_delta: number; delta: number; pm: number;
                           refined?: boolean };                // QA 778fa1b (core): measured on the refined panels (the forecast's own sims once refined)
/** Cut 21 §1: one start measured for the active set (`startForecast()`): D1 and each lit waystone, like `CageOption` — the camp
 *  panel with the run starting on `start`, paired with the current start's. `toll`: what the send pays for it (`$10 × depth`, 0 at D1);
 *  `depth`: the floor `reach` is read at; `delta`: the headline (`bank_delta` when either panel banks, else `reach_delta`). */
export type StartOption = { start: number; current: boolean; toll?: number; biome?: string; depth: number; reach: number; reach_delta: number;
                            bank: number; bank_delta: number; gold: number; gold_delta: number; delta: number; pm: number;
                            short?: boolean;                    // core: the purse cannot pay the toll now — that send starts on D1 (the numbers are D1's)
                            refined?: boolean;                  // QA 778fa1b (core): measured on the refined panels (the forecast's own sims once refined)
                            pass?: boolean;                     // QA a946e04 (core): tonight's pass for this start is paid — the next send from it pays nothing (`toll` is the next night's); `net` is the next send's
                            net?: number; net_delta?: number;   // core: `gold − toll` per send, and its move against the current start
                            death?: number;                     // Cut 22 §4: the death share of a send from this start (0..1), shown beside the bank move
                            death_delta?: number;               // core: its move against the current start
                            passage?: number;                   // Cut 27 §1 (core): the passage a send from here is paid (`+$84 passage`, into the purse at the send; `gold`/`net` count it; 0 from D1 or when the set does not clear the floors above ≥ 95 %)
                            low?: number };                     // Cut 23 §2 (core): as Forecast.low — a `death` of 0 prints `<{low}%`
/** Cut 2 §7 — loot on the hero, brought items (insured = kept on death), the row that would bank/return if any.
 *  Cut 6 §1: `kept` = what that row would bring home now (`$84 · keeps $50`). */
export type Stake = { loot: number; brought: { label: string; insured: boolean }[]; return_row?: number; kept?: number;
                      stalling?: boolean;                                            // Cut 13 §1: the guard has fired this floor — a stall pays nothing (`keeps $0 · stalling`)
                      returning?: boolean;                                           // QA 1a2a4a9: a return/bank row acted — the walk home replaces the chores until the exit (`returning`)
                      swap_left?: string;                                            // QA 0c6e126 (qaY; core): what the last costly swap left on the floor — the strip's fall reads `−$5 left axe`
                      death_keep?: number;                                           // Cut 20 §4 (below)
                      swapped?: number };                                            // Omitted when zero (also absent on older wires). QA 912e135: the carry the run's pack swaps took so far — the strip names a fall `swap` by its rise (`Run.swapped`; the exit line's `swapped` at the end)
                                                                                     // Cut 20 §4: what a death now would keep (the death tier's share) — `carry $78 · bank keeps $78 · death $0`
/** Cut 6 §1 — the ledger line of an exit: one arithmetic line the player can check, `text` is shown verbatim
 *  (`$84 carried · return keeps 60% → $50 · supplies −$12 → $68`). Fractions: `keep_pct` 0..100. */
export type ExitLine = { legacy_earned?:number; packages?:string[]; bloodline_id?:number; carried: number; keep_pct: number; kept: number; spent: number; spent_on: string[]; text: string;
                         secured?: number;                                                                  // Cut 30.5 (core; the owner: a new record is a checkpoint, never an exit): of `carried`, the gold the run's checkpoints secured — kept whole at any exit (a death keeps it alone); `keep_pct` is the share of the rest (`banked $120 · $80 secured + 100% of $40`)
                         trace?: Trace;                                                                     // Cut 9 §5: the exit's last-5 trace (every tier)
                         salvaged?: { kind: string; n: number; gold: number }[];                           // what the exit salvaged before the keep sheet (a return's 40 % cut), per kind in coins; `kind` is a display name — an unidentified kind reads as its flavour (`brittle scroll?`, QA 1a2a4a9)
                         run_id?: number;                                                                   // the run, so a report's trace links can open its replay (QA on e0f87e7: the return sheet's had `watch`, the report's did not)
                         xp?: number; level_ups?: number;                                                   // QA 92eb880: the XP this run earned (the part that crossed a level included) and the levels crossed — the watched report's `xp` line, never a client-side ladder (web's 40·L² was not the core's; `fighter +0 · L4 ↑1`)
                         stolen?: string[];                                                                 // QA e75ec29 (qaR): what thieves took this run and it never got back (`· stolen heal`; flavour-named while unidentified)
                         purse_full?: boolean;
                         cause?: string;                                                                    // QA 0c6e126 (qaY; core): a death's killer as it reads after `died to` (`a goblin archer`) — the report's line leads with it
                         reason?: string;                                                                   // c30-legible (core): why the run ended, ≤ 3 words (`hurt · banked`, `hurt · went home`, `slain · jackal`)
                         swap_left?: { kind: string; n: number }[];                                        // QA 0c6e126 (qaY; core): what the costly swaps left on the floor, per label — `−$5 swapped` names it (`−$5 left axe`)
                         swapped?: number;                                                                  // QA 778fa1b (core): the carried gold this run's pack swaps took off (a find taken in the place of a dearer carried item — the strip's `−$37 swapped`, summed); `carried` is after it
                         wake?: number;                                                                     // QA 778fa1b (core): the heir purse's top-up this death paid (the text's `+$N wake`); `purse_full` now only when the purse was under $80 (just over the $40 line) — a richer death has no purse word
                         stolen_gold?: number;                                                              // QA a946e04 (core, optional): the carried gold thieves took this run (`$36`, the STOLEN list's gold line); else the client sums the `steal` events' amounts
                         start?: number; start_short?: number | boolean;                                    // QA a946e04 (core, optional): the floor the run started on, and (core: a number) the waystone it wanted and did not start on — the toll short or unlit (`from D1 · toll short`); absent when it started where chosen
                         found?: { kind: string; n: number; fate: "kept" | "salvaged" | "shelved" | "used" | "left" | "stolen" | "bones" | "sheet" | "lost" }[];   // QA 778fa1b (qaV): where each find ended, per kind and place (`sheet`: on the keep sheet — the core settles it at `keep`; `lost` never); Σ n == found_n
                         found_n?: number;                                                                  // QA 778fa1b (qaV): the units the run found (a leash stack counts each)
                         toll?: number;                                                                     // QA a946e04 (core): the toll this run's send paid (0/absent from D1 or on the night's pass) — the report's gold line counts it (qaT: `+$71 banked · −$40 spent` beside `$51 → $32`)
                         bones?: { kind: string; n: number }[];                                            // QA 912e135 (core): a death's whole pile per kind, `n` items (a stack is one) — Σ n is the text's `bones: N items`
                         shelved?: { kind: string; n: number }[];
                         driven?: DrivenOff;
                         meters?: MeterWire;                                                                // Cut 29 §3 (core): the run metered (dps dealt/taken by side, hps by source, time split, row shares, supplies, gold/min, hits); absent on an old wire
                         news?: News[];
                         end?: "bank" | "return" | "death";                                                 // run-clear (core): the end's kind (the exit event's tier)
                         reached?: number; new_best?: boolean;                                              // run-clear (core): the deepest floor the run reached; it went past the lineage's record
                         finds?: InvItem[] };                                                               // run-clear (core): what the run found and brought home (a death: left in its bones), rarest first, ≤ 6                                                                  // Cut 24 §2 (core): what was new this run, most telling first, ≤ 3 (`first: Warlord slain` · `record: D10` · `avenged Ulak` · `new find: mail` · `first: the captive` · `driven off: Warlord` · `learned 3`); a run with nothing new has one `differ` line (`deeper: D9, last D8` · `banked, last returned` · `+$23 on last`) — the report leads with these, before the counts                                                              // Cut 24 §1 (core): a boss whose HP did not move for 60 of the hero's actions drove him off — a return-tier exit (keeps 60%), verdict `no counter`; the text reads `returned $N · … · no counter`                                        // Cut 21 §2: found supplies of a kind the shelf sells, put on the shelf at this exit (not salvaged) — `found heal → shelf`                                                            // QA e75ec29 (qaR): a death whose heir purse was already at the top-up line ($40) — no `+$N wake`; the line reads `purse full`
/** Cut 24 §1 (core) — the `no counter` exit: the boss kind + short title (`goblin_warlord`, `Warlord`), the floor, the verdict word
 *  (`no counter`), the defence that shrugged every blow (`shield wall`, ≤ 3 words), the counter in words (`attack boss`) and as a row
 *  the editor can insert (like a patch's). Render `Warlord · no counter · shield wall · try: attack boss`. */
/** Cut 24 §2 (core) — one line of `ExitLine.news`: `k` first | record | named | find | situation | driven | learned | differ; `text` ≤ 6 words, lower case. */
export type News = { k: "first" | "record" | "named" | "find" | "situation" | "driven" | "learned" | "differ" | "oath"; text: string };   // Cut 28 §1 (core): `oath` — `oath kept: D10 · no drink`
export type DrivenOff = { boss: string; title: string; depth: number; verdict: string; defence: string; counter: string; row: Row;
                          held?: number; over?: number;   // Cut 27 §5 (core): `verdict: "order"` — the counter row is in the set at `held` (0-based) under `over`, which acted first: show `R{held+1} under R{over+1}`; the fix is a move (row `held` above `over`), not a new row
                          run_id?: number; hp?: number; max_hp?: number; lost?: number };   // Cut 26 §6 (core; AP: a drive-off at 25/36 hp, no verdict screen): the run (open its verdict from the report), the hero's hp at the drive-off, the carry it lost
/** Cut 6 §1 — one gold movement in the camp's `gold` sheet: `+$50 returned D5`, `−$40 heal`, `−$8 insure sword`. */
export type GoldLine = { bloodline_id?:number; t: number; delta: number; why: string; n?: number; lost?: number };   // QA 912e135 (core): `lost` — on an exit's line, the carried gold the exit did not keep (`$0 died D6 · $157 lost`)
                                                                                                   // QA on 778fa1b (qaV): `n` — the supplies the line bought or refunded (`repeat heal` · n 4 · −$104); absent on other lines
/** Cut 6 §5 — a boss whose counter is a known row (`attack boss`, `throw fire, boss`, `read silence`). */
export type Counter = { boss: string; row?: Row | string; text: string };
export type InvItem = { id: number; kind: string; known: boolean; label: string; hint?: "benevolent"|"malevolent";
                        free?: boolean;                                    // Cut 12 §6: a supply the camp gave (the kennel's leash) reads `leash · kennel`; the core always sends it
                        found?: boolean;                                   // Cut 21 §2: a shelf line an exit put there (found in the dungeon, packed free) — `heal · found`
                        enchanted?: number;                                // QA 524827b (core): the `+N` enchant scrolls read on it added (KEPT `axe +7 → vault · enchanted ×6`)
                        rarity?: Rarity };                                 // run-clear (core `item::rarity`): read off its kind's depth band and its +N; absent = common
/** Run-clear (core `item::Rarity`): an item's rarity, rising — common · uncommon · rare · epic · legendary. */
export type Rarity = "common" | "uncommon" | "rare" | "epic" | "legendary";

export type Ev =
  | { t:number; k:"gun"; state:GunSnap|null }
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
  | { t: number; k: "callout"; text: string; why?: string }                               // ≤ 3 words, for the renderer
                                                                            // Cut 23 §3 (core): `why` — the reason on tap, ≤ 3 words: a `✗` callout's block gloss (`read ✗ no use` → `nothing to learn`), a telegraph shout's foe trait (`bloat swells` → `gas burst next`)
  | { t: number; k: "tame"; id: number; kind: string; ok: boolean }         // Addendum A
  | { t: number; k: "hatch"; kind: string }                                 // Addendum A
  | { t: number; k: "level"; class: string; level: number }                 // Addendum C
  | { t: number; k: "rank"; rank: number }                                  // Addendum D
  | { t: number; k: "projectile"; src: number; dst: number; path: [number, number][] } // Addendum E: 1 tile per tick along path
  | { t: number; k: "rest"; seconds: number }                               // Cut 2 §1: emitted at exit; the viewer shows `rest Nm`
  | { t: number; k: "bones"; heir: number; items: number }                  // Cut 2 §2: a bones pile (left on death, or recovered by a later heir)
  | { t: number; k: "see"; id: number; e?: Entity }                         // Cut 2 §7: first sight of an entity (renderer flashes on a boss); not emitted by the core yet
  | { t: number; k: "ending"; ticks: number }                               // Cut 7 §4: the last `ticks` before an exit start here (optional; else the client infers exit − 30)
  | { t: number; k: "max_hp"; id: number; max: number; delta: number; cause: string }   // QA 1a2a4a9: max HP moved (`hunger`: −1 a bite on an unlit hunger floor; the callout reads `hunger −1 max`)
  // Cut 25 §3 (core): a drain stretch starts — hp or max hp falling with no foe in view; `cause` one word (`starving`, `poisoned`,
  // `drained`), once per stretch. Until a foe is in view, the stairs or the run's end, the hero's `hurt` / `max_hp` with no foe in view are
  // the drain's: dead time — play them at the travel rate and show the cause once.
  | { t: number; k: "drain"; cause: string }
  /** Cut 28b (core): the sworn oath's fate, said once in a send as it happens — kept (`cause` the kept row's verb, or empty), broken (`cause` the tool
   *  it forbade: `return` · `rest`, or `stalled` · `driven`), or missed at the end (not kept, `cause` empty). `row` the row that did it (−1 none). */
  | { t: number; k: "oath"; kept: boolean; row: number; cause: string }
  /** Cut 29 §3 (core): hp the hero (`id` 0) or a pet regained this tick, by source (`potion` · `rest` · `regen` · `skill` · `pet`) — the meters'
   *  read, not a beat (never renderable; the watch may ignore it). */
  | { t: number; k: "heal"; id: number; amount: number; src: string }
  | { t:number; k:"recover"; id:number; amount:number; hp:number; src:"leeching" };

export type StepResult = { events: Ev[]; snapshot: Snapshot; run_over: boolean;
                           /** Cut 28 §3 (core): the step's calm stretches — `[from, to]` run ticks (inclusive, `Ev.t`) with no decision and no threat: no foe
                            *  awake in view, no hurt, no row of the set acting (a chore / a `pick up` / `rest` row is calm), no beat (a descend, a fact, a
                            *  telegraph, a theft, a death, a callout, a level, a tame, a new find). Play them at the travel rate in every mode, 1× included
                            *  (1× keeps fights and beats at 1×). Absent when the step had none. */
                           calm?: [number, number][];
                           exit_pending?: { items: InvItem[]; tier: string; worth?: number[]; auto_keep?: number[];
                                            decide?: boolean;   // Cut 29 §4 (core): the sheet is a decision — a find beats something in the full vault (or finds outnumber its free slots); false → settle it (`autoKeep()`) without a sheet and show `note`
                                            note?: string } };  // Cut 29 §4 (core): the settled exit's one line (`kept leather +1`); absent when it keeps nothing new                     // Addendum D; `worth`: each item's salvage at this exit, in coins

/** Cut 20 §5: `bounty` — tonight's bounty floor (the lineage best + 2: gold ×2 and a guaranteed item), the shaft's `D12 ×2`. */
export type ForecastDepth = { depth: number; reach: number; cause?: string; pm?: number; try?: ForecastTry; wall?: string; bounty?: boolean | number;
                              biome?: string;    // Cut 26 §2 (core): on a set with a route, the biome this floor sits in on it (`fens` at D5 for route [5]); absent on the base order
                              clear?: number;    // Cut 27 §1 (core): the share of the sims on this floor that got through it (0..1); absent where no sim stood on it
                              affix?: string;    // Cut 116 §1 (core): beside `boss`, the live heir's affix on him (`armoured`)
                              boss?: string };   // Cut 24 §5 (core): on the floor a boss stands on (met there: the Warlord D8), his kind — name him on this row; the next row's `try` / `wall` are his
export type Forecast = { depths: ForecastDepth[]; causes: { cause: string; share: number }[];
                         known_to: number;                               // depths[].cause: Cut 4 §8, optional per-depth top cause; pm: Cut 9 §3, the binomial half-width (`D4 71% ±6`); wall: Cut 18 §3, the sealing boss's kind where reach falls to ≤ 5 % below his floor (`D9 0% · warlord wall`)
                         ends?: { bank: number; return: number; death: number; stall?: number; gold: number; pm?: number;
                                  passage?: number };                     // Cut 29 §6 (core; AX: `$81` banked under `~$260`): of `gold`, the waystone passage paid at the send — the exit's `BANKED $N` is `gold − passage` (show `~$125/run +$135 passage`)
                         refined?: boolean;                              // QA 778fa1b: always present from the core (false = the first pass, true = the refine); absent only on an older core
                         start?: number;                                 // Cut 21 §1: the floor the sims started on (`Lineage.start` when lit and payable, else 1); rows above it read reach 1.0
                         shadowed_by?: (number | null)[];                                                    // QA 92eb880: per row of the set (by index), the earlier row (0-based) that takes every moment it could fire — mark it `shadowed by R{n+1}`; null = free; absent = none shadowed   // Cut 13 §5: the refine pass (100 sims); a first paint is marked `…`   // Cut 12 §3: how a send ends (rates 0..1 summing to 1; `stall`: came home by the cap, nothing in the rules) and the mean gold brought home per send
                         sims?: number;                                  // Cut 23 §2 (core): the sims the shares were drawn from
                         low?: number;                                   // Cut 23 §2 (core): the smallest share one sim makes, in whole percent (⌈100 / sims⌉): a share sampled at 0 prints `<{low}%` (`death <2%`), never `0%`; a reach of 1.0 below `start` is not sampled (prints as is)
                         fold_to?: number;                               // Cut 27 §1 (core): the last floor the watch folds for this set (every floor from `start` to it clears ≥ 95 %: `ForecastDepth.clear`); absent = nothing folds
                         oath?: OathShare;                               // Cut 28 §1 (core): the sworn oath priced on this panel (`oath · D10 no drink · 34%`); absent with none sworn
                         vs?: ForecastVs };                              // Cut 22 §3: the paired move against the last painted set (absent without an edit, or on a core that answers `forecastVs(prev)` instead)
/** Cut 22 §3 — an edit's paired move: this set's panel minus the previous set's, on the same seeds (so far tighter than either
 *  absolute bar). `delta`: the move (a 0..1 fraction, signed); `pm`: its own paired half-width — a move inside it reads `≈`. The
 *  ends (`bank`, `death`, …) may come as a bare delta or as `{delta, pm}`. */
export type VsMove = { delta: number; pm?: number; base?: number };   // QA 912e135 (core): `base` — the sent set's own share on the same seeds (today's kit); base + delta is the active panel's
export type ForecastVs = { oath?: VsMove;   // Cut 28 §1 (core): the sworn oath's share moved by the edit (same paired seeds); absent with none sworn
                           depths: ({ depth: number; abs_pm?: number } & VsMove)[]; bank?: number | VsMove; death?: number | VsMove; return?: number | VsMove; gold?: number | VsMove;
                          stall?: VsMove;   // QA 912e135 (core): the stall share's move — a death traded for a stall reads (`stall 0→11%`)
                           sims?: number; refined?: boolean };   // QA 778fa1b (core): `refined` — the panels paired are the refined ones; a vs read before the refine is false, ask again after it   // core: `abs_pm` = the active bar's own ± at that depth; `sims` = the paired seeds both panels ran
/** Cut 27 §1 (core) — a state change on a folded floor, a chip on the fold line. `kind`: `theft` · `find` · `gold` · `use` · `fact` ·
 *  `dip` (the floor's lowest hp when ≤ 30 % of max) · `max_hp` · `pet` · `boss` · `bones` · `level` · `hatch`; `text` ≤ 3 words. */
export type FoldBeat = { depth: number; t: number; kind: string; text: string };
/** Cut 27 §1 (core) — one folded floor: its snapshot at its first tick (later entities/items and seen tiles folded in — as
 *  `runlog.floorSnapshot`), its events (a tap on the fold line plays them: `load(snapshot)`, `apply(events)`), its clear, gold and beats. */
export type FoldFloor = { depth: number; clear: number; gold: number; snapshot: Snapshot; events: Ev[]; beats: FoldBeat[] };
/** Cut 27 §1 (core) — `fold()` right after `send()`: the live run played through the floors the set clears ≥ 95 % —
 *  `D{from}–{to} · {clear%} · +${gold} · chips…` (`chips`: the line's words by kind, ≤ 3 words each, e.g. `stolen heal`, `2 finds`,
 *  `hp 9/40`; `beats`: every state change, each on its floor). `step` is the fold as one `step()`: all its events, the snapshot on the
 *  first unfolded floor (the watch opens there), `run_over`/`exit_pending` if the run ended inside it. Nothing folded: `to < from`,
 *  no tick ran, `floors` empty, `step.snapshot` the current one. */
export type FoldLine = { from: number; to: number; clear: number; gold: number; beats: FoldBeat[]; chips: string[]; floors: FoldFloor[]; step: StepResult;
                         /** Cut 28 §4 (core; AV: "`send skips rest` sent a 9/40 heir" — it was the fold, handing off hurt): the hero's hp and max hp where the
                          *  fold hands the run to the watch; `low` when hp ≤ half of max — say it on the fold line (`hp 9/40`; the chip is in `chips` too). */
                         hp?: number; max_hp?: number; low?: boolean };
/** Cut 27 §2 (core) — how a divergence branch's whole run ended (the panel's own result for that seed). `tier`: bank · return · death · stall. */
export type DivergenceEnd = { tier: string; depth: number; cause?: string; gold: number;
                              oath?: boolean };   // Cut 28 §1 (core): with an oath sworn, whether this branch's whole run kept it
/** Cut 27 §2 (core) — one branch's seconds: `snapshot` a few ticks before the divergence (the same state in both branches unless the
 *  sets packed differently), `events` from it to ~60 ticks past the divergence tick (earlier at a descend or the run's end), `end_snapshot`
 *  its last frame. `row`: the row (0-based, in its own set) that fired at the divergence tick (absent: none — a chore or a step);
 *  `text` its words (`R5 bank`, `—`). */
export type DivergenceBranch = { row?: number; text: string; snapshot: Snapshot; events: Ev[]; end_snapshot: Snapshot };
/** Cut 27 §2 (core) — a row's mean fires per send in each set on the paired panel (`R3 fires 4× more`); indices 0-based, absent where
 *  the set lacks the row. */
export type RowFires = { sent_row?: number; new_row?: number; text: string; sent: number; new: number };
/** Cut 27 §2 (core) — the edit as a scene (`divergence(prev)`): on paired panel seed `seed` the first `tick` where the active set acts
 *  differently from `prev` (the sent set), on floor `depth`; `sent_row`/`new_row` what fired there; the branches' next seconds
 *  (`sent`, `new`: play them side by side / before-after on the renderer) and each whole run's end (`sent_end`/`new_end`: `lives · D9`
 *  vs `dies · D7`). `moved`: the paired move's headline |Δ| (0..1, max over the shaft and the ends); `inside`: it sat within its ±
 *  (show `≈ ±6 · R3 fires 4× more` from `fires`). Null from the engine: the sets play alike on every seed tried. */
export type Divergence = { seed: number; tick: number; depth: number; sent_row?: number; new_row?: number; sent_end: DivergenceEnd; new_end: DivergenceEnd;
                           sent: DivergenceBranch; new: DivergenceBranch; moved: number; inside: boolean; fires?: RowFires[]; sims: number };
/** Cut 10 §2 — a boss floor whose counter fact is known and whose row is absent from the set: `D9 0% · warlord · try: attack boss`;
 *  tapping the bar inserts `row` at the top (optional on the wire; the client derives it from `Lineage.counters` when absent). */
export type ForecastTry = { row: Row; text: string; boss?: string; met?: number };   // Cut 24 §5 (core): `met` — the floor the boss is met on (this row's depth − 1)
/** Cut 4: `blocked` = the first row whose conds held but whose verb could not execute. Cut 6 §3: `rows` = every row above the
 *  fired one with one reason why it did not fire (`none held`, `no path`, `not in view`, `hp 8% ≥ 30%`). */
/** QA 92eb880: `foes` = the player's count (every hostile seen from this action to the next, running thieves and the killer
 *  included); `rule_foes` = what `foes>=` counted (optional on old saves). */
export type TraceTurn = { t: number; row: number; verb: Verb; hp: number; foes: number; rule_foes?: number; telegraphs: string[];
                          max_hp?: number;   // Cut 28 §2 (core): the hero's max hp at this action (`6/9` — a drain or a curse shows); always sent by a Cut 28 core
                          blocked?: string; rows?: { row: number; why: string; because?: Because }[];
                          blows?: { t: number; by: string; dmg: number; hp: number }[] };   // QA 524827b (core; qaAB): the blows since the previous action, before this one — the table's rows between two actions   // because: Cut 11 §1; why `foes fleeing` / `foes appeared after` where `foes not ≥N` met a foes column ≥ N (QA 1a2a4a9)
/** Cut 11 §1 — why a state reason held: the most recent event that put it there (`den took the heal, D3`, ≤ 8 words),
 *  its tick and floor. The client scrubs the run's replay to `t` when it still holds the run's events. */
export type Because = { text: string; t: number; depth: number };
/** Cut 28 §2 (core) — a max-hp step inside a trace's window (`hunger −1 max`): tick, max after, delta, cause (`hunger`, `drain`, `curse`). */
export type MaxStep = { t: number; max: number; delta: number; cause: string };
export type Trace = { turns: TraceTurn[];
                      max_steps?: MaxStep[];   // Cut 28 §2 (core): the hero's max-hp steps from the trace's first turn to its end, oldest first; absent when the max never moved there
                      provenance?: Because[];                                                // Cut 11 §3: every `because` event of the run (exit traces)
                      blow?: { t: number; by: string; dmg: number; hp: number };
                      blows?: { t: number; by: string; dmg: number; hp: number }[];
                      hp_lost?: { by: string; dmg: number }[];
                      hp_healed?: number };   // QA 308f045 (core; qaAC: `since full hp` summing past his max): the hp healed over the same stretch                             // QA 524827b (core; qaAA: "where 36 hp went takes a replay"): a death's hp lost since full, per cause, most first        // Cut 25 §3 (core; AN: `14 → 0` on one `goblin −2` row): every blow after the last action, oldest first, hp after each; the last is `blow`; absent when `blow` was the only one — the table shows one row per blow           // QA 0c6e126 (qaY): a death's killing blow, the table's last row                                              // Cut 11 §3: every `because` event of the run (exit traces)
/** A candidate row. Death patches insert before `insert_at`; stall patches (core README) may instead `replace` the row at
 *  `insert_at` or `remove` it (`row` echoes the removed row).
 *  Cut 11 §2: `root` names the chain's root the row answers (`den took the heal`); `insert_at: -1` is an unlock pseudo-patch
 *  (`root.text` = `◆2 cond: alert`; the client buys the cond's unlock, then inserts the row at the top); `below_bar` marks a
 *  §4 candidate under the verdict bar (`survives 40% · below bar`), shown dimmed. */
export type Patch = { row: Row; insert_at: number; survive: number; forecast_delta: number; replace?: boolean; remove?: boolean;
                      gem?: boolean;       // Cut 27 §4 (core): after `deathDeltas` lands (death and stall screens) — the gem: the best whole-run patch that does not harm, always patches[0]; none flagged → no gem (`edit`)
                      restores?: number;   // Cut 27 §5 (core): a row the player removed since an earlier send, restored at its old index (0-based, = insert_at): label `restore R{n+1}`
                      root?: { text: string }; below_bar?: boolean;
                      forecast_depth?: number; forecast_pm?: number;                         // QA 23ed91f: the camp bar `forecast_delta` moves (death's depth + 1, ≤ known_to) and its 95 % ± — set by `deathDeltas`
                      camp_pending?: boolean;                                                // QA 23ed91f: on `death()`'s patches — `forecast_delta` is the verdict's 12-sim ranking estimate, NOT the camp's; paint reach as pending until `deathDeltas(id)` lands
                      exits?: boolean;                                                       // QA 778fa1b (core): the row ends the run (`return`/`bank` inserted or narrowed; a cut of one is not) — its cost is floors: name it (`return early`) beside its reach. A costly exit (reach ≤ −10) never leads beside a patch beating the base by 15
                      drops?: number;                                                        // Cut 19 §4: an insert onto a full set drops this own row (set index; the dead run's least-fired, ties the lowest) — `+ drop R5`, the drop sheet opens on it
                      buys?: { kind: string; label: string; price: number };                 // QA 0c6e126 (qaY; core): the row's named item the next heir will not carry — offered with its purchase (the tap buys it, then applies); its reach measured with it bought
                      moves_from?: number;                                                   // Cut 25 §2 (core): a move — the set's own row at `moves_from` goes above the row at `insert_at` (`row` echoes it; nothing added or cut): `move R5 above R2`
                      whole?: PatchWhole;                                                    // QA 524827b (core): the whole-run move on the camp's panel once applied (paired, with `deathDeltas`); absent while `camp_pending`
                      no_gain?: boolean;                                                     // Cut 28 §2 (core): the patch survives no more replays than the unpatched rules (`survives 0/12 · no gain`) — never print a green move beside it
                      unlock?: string };                                                     // optional: the pseudo-patch's unlock id (else derived from the row's cond)
/** QA 524827b (qaAA: `drink unknown · 12/12` led, then the camp's killers read fire 28 % · poison 26 %): a patch's whole-run move — reach
 *  at `forecast_depth` and the death share, paired over the camp's sims, each with its 95 % ±; `harms` (worse beyond a ±: never the lead,
 *  never the gem's default); `risk` the self-dealt harm the patch raises (`fire`). Fractions 0..1. */
export type PatchWhole = { reach: number; reach_pm: number; death: number; death_pm: number; harms?: boolean; risk?: string;
                           reach_from?: number; reach_to?: number;
                           depth?: number;        // QA 308f045 (core; qaAC: `reach D9 ≈ ±1`, then the camp's `vs sent · D6 −21`): the floor the reach is read at — the camp's `vs sent` head (the frontier when it moves, else the floor that moves most); `Patch.forecast_depth` is the same
                           death_from?: number };  // QA 308f045 (core; qaAC: `death −100 ±1`): the death share before the patch (0..1) — print `death 100→0%`   // Cut 26 §6 (core; AP: `reach D5 −76`): the bar's reach before/after the patch (0..1) — print the move as from→to (`reach D5 90→14%`), never a signed delta                                                     // optional: the pseudo-patch's unlock id (else derived from the row's cond)
/** Cut 30 §2 (core) — a death's one cheapest lever before the pen opens: `kind` spend · package · wait, `text` ≤ 3 words (`sword +2`, `Hunter`, `scarred ×3`). */
export type Lever = { kind: string; text: string; id?: string; variant?: number };   // Cut 115 §4: a `tactic` lever's package and variant (`takeFix`)
/** Cut 115 §1 (core): the build the picks make — its name (`Bulwark`, `Guarded skirmisher`), the synergy its pair forms and the effect, the picks. */
export type BuildWire = { name: string; synergy?: string; effect?: string; picks: string[] };
/** Cut 115 §1 (core): rule fires by who chose the row. */
export type CreditShare = { credit: "picked" | "taught" | "default" | "chores" | string; fires: number; share: number };
/** Cut 115 §3 (core): a move read at one wall (paired sends from its waystone). */
export type WallRead = { depth: number; boss: string; n: number; better: number; worse: number };
export type Death = { difficulty?:number; modifier_catalogue?:ModifierInfo[]; modifiers?:EncounterModifiers; hero?: { name:string; bloodline_id:number; heir:number; class:string }; package?: string; lever?: Lever; pick?: Lever; credit?: string;   // Cut 115: `pick` the tactic fix (`try: gas step · burn`), `credit` who chose the deciding row; Cut 30 §2 (core): the package row that acted last (`Steady · HP<20% → return`); the cheapest lever (absent once the pen is open)
  run_id: number; depth: number; cause: string; margin: string; verdict: "gap"|"dice"|"stall"|"row"|"order"|"route";   // route: Cut 26 (core) — the far stair the set's route took killed him (`route_cause`)
                      fight?: MeterWire;                                                    // Cut 29 §3 (core): the fight he died in, metered — the death screen's breakdown; absent for a stall
                      lean?: "dice" | "gap";                                                // blind b58b431 (core): "gap" — a `dice` most of whose unpatched replays die too (≤ 6/12): stamp it a gap, never luck. Cut 26 §6 (core; AO: `GAP` beside `unpatched 10/12`): a gap/row/order most of whose unpatched replays survive (> 6/12) — stamp it beside the counts (`GAP · dice-leaning`)
                      route_cause?: RouteCause;                                             // Cut 26 risks (core): on `route`, the fork, the stair taken and the other; its lead patch is a route edit (`route_cause.route`)   // stall: Cut 13 §1, a stalled run's verdict; row: Cut 19 §4, a row the player wrote was the dying action and cutting it survives (the Rust side is a String)
                      order_over?: number;                                                  // Cut 25 §2 (core): on `order`, the row (0-based) that won every tick `cause_row` would have acted on — `R5 under R2`; the lead patch moves R5 above it
                      cause_row?: number;                                                   // Cut 19 §4: on `row`, the set's row (0-based) that killed him (`R2`); patches[0] cuts it (`remove`, or `replace` narrowed)
                      baseline: number;                                                   // core addition: survival of the unpatched rules, 0..1
                      replays?: number;                                                   // QA 0c6e126 (qaY; core): the reseeded replays `survive`/`baseline` are shares of (12) — printed as counts, `7/12`
                      trace: Trace; patches: Patch[];
                      morgue: string;
                      line?: ExitLine;                                                       // Cut 6 §1: the death's ledger line
                      chain?: Because[];                                                     // Cut 11 §2: the death's chain, root first (the rows' `because`s, deduplicated)
                      rules?: RuleSet;                                                       // the rules the run died under (the accounting's row labels; else the morgue's lines)
                      notes?: string[];                                                      // Cut 13 §4: the run's last two chronicle notes, under the headline (never a `saved him` — QA 92eb880)
                      luck?: DeathLuck;                                                      // Cut 28 §2 (core): on a death most replays survive (a `dice`, or `lean: "dice"`) — the rare event that killed him and its odds; lead with it (`goblin −6 at 6 hp · 1 in 6`)
                      boss?: string;                                                         // blind 3ab97ea (core): the boss this death was fought under — his wall, never luck nor dice-leaning
                      nothing_beats_base?: boolean };                                        // QA 92eb880: a `dice` death none of whose patches survives more than `baseline` (a 100 % base: the replays win the fight he lost) — say `nothing beats base · base N%`, not `below bar`. Cut 26 §6 (control rater AQ: `nothing beats unpatched 12/12` over a patch reading `survives 12/12`): the patches of such a death are what was TRIED, never help — do not print their `survives N/12` as a headline (e.g. `tried · same as base`)
/** Cut 26 (core) — a `route` verdict: at the fork `fork` the set's route took `taken`; the near/other stair's replays survive `survive`
 *  (≥ 50 %, beating `baseline` by ≥ 15 pts) — `D5 fens · route`, the fix `take burrows` = `setRules({...set, route})`. */
export type RouteCause = { fork: number; taken: string; other: string; route: number[]; survive: number; base?: number };   // base: the route taken, past the floor (paired)
/** Core addition: the last ≥ 4 runs all came home with no new depth — the row that ended them, how many, a ≤ 12-word line,
 *  and up to 3 patches with forecast deltas at the stall depth + 1 (`survive` = the patched reach there). A state: the
 *  last slice's wins on merge. */
export type Stall = { row: number; fired: number; text: string; patches: Patch[]; trace?: Trace };   // trace: Cut 9 §5, the last run that row ended
// Fractions: Forecast.depths[].reach, causes[].share, Death.baseline, patches[].survive and forecast_delta are 0..1.
export type Highlight = { pattern: string; score: number; t: number; run_id: number; text: string;
  /** Cut 25 §5: client-side, set by `mergeReel` — the times this line's shape (`reelShape`) came up across the absence's slices
   *  (the report prints `×n` when > 1). Absent on a core line. */
  n?: number };
/** Cut 28 §2 (core) — a luck-leaning death's event: `text` ≤ 6 words (`two blows at 6 hp`, `a max hit at 4 hp`, `goblin −6 at 6 hp`), `t` its tick,
 *  `odds` the share of the reseeded replays that died too (0..1) and `one_in` = round(1 / odds) (`1 in 6`; 12+ when no replay died). */
export type DeathLuck = { text: string; t: number; odds: number; one_in: number };
/** Cut 28 §2 (core) — a report's first screen, decisions first: `k` oath · plateau · counter · record · death · driven · bounty · pending;
 *  `text` ≤ 6 words (`oath kept: D10 · no drink`, `plateau: none past D13`, `mother: fire learned`, `record D14`, `D9 death · order`). */
export type ReportLead = { k: string; text: string };
/** Cut 28 §1 (core) — the sworn oath over an absence: `runs` the sends while it was sworn, `kept` those that met it, `done` it was kept (the reward
 *  granted, the price spent into it), `reward` what it gave. */
export type OathReport = { id: string; chips: string[]; text: string; runs: number; kept: number; done: boolean; reward?: OathReward; price?: number;
  broken?: number; cause?: string };   // Cut 28b (core): the sends that broke it and the cause most share (`R2 return`)
/** Cut 30 §4 (core) — one thing that grew on a track over an absence (`character` · `L7`, `items` · `+$2400`, `scale` · `best D14`,
 *  `town` · `a building`, `<track>` · `opened <stage>`); the report leads with them. */
export type GrewLine = { track: string; what: string };
/** Cut 30 §4 (core) — a track on the tracks panel (`character` · `items` · `scale` · `town`): its stage (≤ 2 words), stages reached,
 *  the next stage and its trigger (`next · kennel · first tame`), progress toward a numeric trigger (0..1). */
export type Track = { id: string; stage: string; stages: number; next?: string; trigger?: string; progress?: number };
/** Cut 30 §3 (core) — a building on the town scene (`blacksmith` · `storehouse` · `kennel` · `bank`), its look 1–3, the day built. */
export type Building = { id: string; level: number; day: number; trigger?: string };   // trigger: c30-legible (core) — what raised it (`first gold home`)
/** Cut 30 §5 (core) — the quest on the board: one plain goal (≤ 5 words: `reach D10 · no return`), the reward's picture
 *  (`title` · `row` · `slot` · `card` — `art/ui/oath/`), progress 0..1, kept, a free swap left today (`swapQuest()`). No stake. */
export type Quest = { goal: string; reward: string; progress: number; done: boolean; swap: boolean };
/** Cut 30 §3 (core) — the town: its buildings (built in order), the next plot staked and its trigger, the bank (`bankDeposit` /
 *  `bankWithdraw`; ~2 % a night, capped at `bank_cap`), the interest it paid in all, the quest board (from the Warlord slain). */
export type Town = { home?: boolean; auto_collect?: boolean; buildings: Building[]; next?: string; next_trigger?: string; next_ready?: boolean; bank: number; bank_cap: number; interest: number; quest?: Quest; quests_done?: number;
  workers?: WorkerPost[] };   // Cut 30.5 (core): the workers at their posts (hired; the lit node's greyed with its price)
/** Cut 30 §2 (core) — a package (`Guarded L3`): `kind` stance · tactic · temperament; `level` 1–5 from `runs` (the runs one of its rows
 *  fired in; `next_at` the runs the next level wants); `slot` when equipped; `owned` once its stage came (`trigger` until then);
 *  `level_price` the marks a level spend costs (`spendLevel`). */
export type Package = { id: string; name: string; description?: string; kind: string; level: number; runs: number; next_at?: number; slot?: number; owned: boolean; trigger?: string; level_price?: number; variants?: string[]; variant?: number;
  /** Blind 77030eb (core `packages::level_adds`): the rows the next level brings or changes to; absent: none (or an older core). */
  level_adds?: Row[] };
/** Cut 30 §1 (core) — a drilled counter (`drill · attack boss`, named, announced once as `DRILLED · Warlord`), revocable (`revokeDrill`);
 *  `scar` the boss's scar now in % (`scarred ×3` = 15). */
export type Drill = { boss: string; rows: Row[]; revoked: boolean; scar: number };
/** Cut 30 §2 (core) — a row of the compiled set: its package label (`Steady`, `drill · Warlord`, `boss focus`; empty for a pen row) and
 *  the row above that always wins it (`Guarded wins`), by index. */
export type RowSource = { label: string; shadowed_by?: number };
/** Cut 30 §2 (core) — the lineage's packages: every package (owned or its trigger), the stance (never empty; `custom` on an old save), the
 *  tactics in their slots, the temperament (from heir 3; `offer` the wake's three cards, card 1 worn until `pickTemperament`), the drills,
 *  the scars (boss → %), the pen (open at the Mother met or a 3-day stall; the editor's rows sit above the packages), and per compiled
 *  row its source. `literal` a harness's lineage (no packages). */
export type Packages = { all: Package[]; stance: string; tactics?: string[]; tactic_slots?: number; temperament?: string; temperament_open?: boolean;
  offer?: string[]; drills?: Drill[]; scars?: [string, number][]; pen_open: boolean; pen_needs?: string[]; rows?: RowSource[]; literal?: boolean; build?: BuildWire };
/** Cut 30 §2 (core) — a package move priced on the paired panel (`packageOptions(sims)`, best first): `action` equip · level, the shares
 *  of the sends that pass the record / reach it / bank / die, and each move against the set as it stands (`Guarded · death −8`). */
export type PkgOption = { id: string; action: string; slot?: number; price?: number; past: number; bank: number; death: number; reach?: number;
  d_past: number; d_bank: number; d_death: number; d_reach?: number;
  /** Blind c4705f9: the paired read — the sends both panels ran on the same seeds, and of them those the move ended better / worse
   *  (deeper, or as deep with a better exit). Absent on an older core. */
  n?: number; better?: number; worse?: number; walls?: WallRead[] };
/** Cut 30.5 (core; docs/CUT30_5.md, docs/AUTOMATION_TREE.md §2) — a node of the works tree. A **worker** retires a chore done by hand a few
 *  times (`count`/`need`, or `fallback_h` of lineage age once its chore exists); a **stage** is one of the four tracks' stages (the tracks
 *  panel's rows, now the tree's branches). ids — workers: quartermaster · porter · scout · armourer · apprentice · keeper · clerk · drillmaster ·
 *  kennel_hand · herald · guide (in the tree's order); stages: `<track>:<stage>` (`town:bank`, `character:second stance`).
 *  `state` — a worker: `done` (hired) · `lit` (the one node to buy: trigger met, its turn; ≤ 1 at a time) · `ready` (trigger met, waits its
 *  turn behind the lit one) · `open` (its chore exists; counting) · `shut` (its chore not yet: `trigger` names the gate); a stage: `done` ·
 *  `next` (the track's next) · `later`. Draw done dimmed, the lit one full with its price, the next two as silhouettes; never the whole tree. */
export type WorkNode = {
  id: string; kind: "worker" | "stage"; branch: string;   // branch: trunk · character · items · scale · town
  name: string;                                         // ≤ 2 words (`porter`, `kennel-hand`, `second stance`)
  state: string;
  chore?: string;                                       // worker: the act by hand — chest · send · wear · forge · keep · deposit · level · field · swap · start (none: the quartermaster, given)
  count?: number; need?: number;                        // worker: chores done by hand / the trigger (`2/3`); a hired worker keeps its last count
  price?: number; affordable?: boolean;                 // worker: the hire's gold (0 = free; forge units, fixed with the forge's), paid from the purse then the chest
  fallback_h?: number;                                  // worker: the lineage age (h) that lights it without the count, once its chore exists
  trigger?: string;                                     // ≤ 3 words: a stage's trigger, a shut worker's gate (`bank built`)
  tip?: string;                                         // worker: ≤ 4 words, the works sheet's fragment (`hauls home · while away`)
  beat?: string;                                        // worker: the hire's beat, ≤ 2 words, caps (`AUTO HAUL`)
  post?: string;                                        // worker: where it stands — crate · mouth · storehouse · blacksmith · bank · tent · kennel · board
  paused?: boolean;                                     // worker: hired and switched off (`setWorker(id, false)`): its chore is by hand again
  rank?: number;                                        // week 2 (the owner: later worker upgrades): a hired worker's rank 1–3 — its look (and its post's)
  rank_price?: number; rank_wait_d?: number;            // week 2: the next rank's gold and the days of service it still waits (0: on offer, `Works.lit_rank`); absent at IV
  bonus?: string; rank_adds?: string;                   // week 2: the rank's small real edge now (`bonus`) and what the next rank adds (`rank_adds`), ≤ 4 words (`+4% hauls`, `−10% steps`, `25‰ interest`, `−5% rest`, `half toll`, `levels −◆1`, `insures its finds`); keeper, kennel-hand and herald ranks are looks (none)
};
/** Cut 30.5 (core) — the `next` pill: the single next goal. `kind` buy (the lit node, affordable) · chest (a haul waits, before the porter) ·
 *  send (the hero waits for a SEND, before the scout) · gold (the lit node, short: `have`/`need` in gold) · count (a node's chores by hand:
 *  `have`/`need`) · system (the next system's wait: `need` hours) · none. `text` ≤ 3 words + the number (`open chest`, `porter · 2/3`,
 *  `apprentice · $96/$120`, `bank · in 3h`); tapping it opens the tree on `node`. */
export type NextPill = { kind: string; node?: string; text: string; have?: number; need?: number };
/** Cut 30.5 (core) — the works tree (`Lineage.tree`; the tracks panel's successor). `chest` the haul waiting uncollected (gold not in
 *  `Lineage.gold`, which is the purse: collected gold, what a tap can spend) — `openChest()`; it never caps or decays, and the engine's own
 *  needs (the restock, insurance, a waystone's toll) draw on it after the purse. `waits` the hero is home and waits for a SEND (before the
 *  scout: a send is one run; the gem/mouth sends him); `sent` a send by hand is under way; `auto_send` the scout is hired (offline uncapped).
 *  `ledger` = purse + chest + bank: every gold movement summed (the conservation audit). */
export type Works = { nodes: WorkNode[]; lit?: string; lit_rank?: string; next?: NextPill; chest: number; waits: boolean; sent: boolean; auto_send: boolean; ledger: number };
/** Cut 30.5 (core) — a worker at its post on the town scene (`Town.workers`): hired ones, and the lit node's worker greyed (`lit`, `price`). */
export type WorkerPost = { id: string; post: string; lit?: boolean; price?: number; paused?: boolean; rank?: number };   // rank 1–3 (week 2): the worker's look
/** Cut 30.5 (core) — what a worker did over an absence (`ReturnReport.workers`): `apprentice` · `+2 steps` (n 2); `first` the first time it
 *  ever acted (name it once: `apprentice · +1 step`; later fold into the report's lines). */
/** blind c4705f9 (A, B: `purse −$12562` unexplained): `items` the steps a buying worker reached (`sword +3`), `spent` the purse it paid. */
export type WorkerAct = { id: string; what: string; n: number; first: boolean; items?: string[]; spent?: number };
/** RUNS_UI (core; docs/RUNS_UI.md) — one run in the runs log (`Lineage.runs`, oldest first, cap 60). `via`: how it ran — `away` (an absence's
 *  batch, `runOffline*`), `town` (unwatched while the app was open: `advance`), `watched` (live in the watch). `absence`: the absence an `away`
 *  run belongs to (the log folds by it). `clock_s`: the lineage clock at its end (the stamp: `Lineage.clock_s` − it). `gold` what came home,
 *  `found` the units found, `kept` what went to the vault, `turns` its ticks (10 a second), `best` a new best depth, `death_id` a death whose
 *  verdict the core still holds (`death(id)`). A record with `sampled` is no run: the absence's runs extrapolated past its stall (`+N`). */
export type RunRec = { id: number; heir: number; via: "away" | "town" | "watched"; absence?: number; clock_s: number;
  start: number; depth: number; tier: "bank" | "return" | "death"; reason?: string; gold: number; found: number; kept?: string[];
  turns: number; best?: boolean; death_id?: number; sampled?: number;
  finds?: InvItem[];     // RUNS_UI × run-clear (core): the run's finds, rarest first, ≤ 6 — the entry's rarity mark and its card
  secured?: number };    // RUNS_UI × Cut 30.5 (core): of `gold`, the checkpoints' (kept whole at any exit) — `$212 · $80 safe`
/** RUNS_UI (core) — the run under way right now (`Lineage.live`; absent at home): the hero's floor, hp, and the run's tick. */
export type LiveRun = { activity?: "combat" | "returning" | "exploring"; run_id: number; heir: number; depth: number; start: number; hp: number; max_hp: number; turn: number };
/** RUNS_UI (core) — `advance(ms)`: the open app's clock run on the lineage (rest, then the next run, unwatched; a run in flight stays in
 *  flight at the budget's end). `ended` the runs it finished (refresh the lineage then), `live` the run under way after it. */
export type Advance = { ended: number[]; live?: LiveRun };
/** RUNS_UI (core) — a past run re-simulated from its send (`replay(id)`): each floor's first snapshot (later entities, items and seen tiles
 *  folded in, as the watch's run log does) and its events; `hash` the FNV-1a of its events (an exit's line and trace left out), `ticks` its
 *  length. Identical to the run as it was played (same seed, state and inputs). */
export type ReplayFloor = { snapshot: Snapshot; events: Ev[] };
export type Replay = { run_id: number; floors: ReplayFloor[]; hash: string; ticks: number };
/** RUNS_UI (client; Cut 31's heroes on the wire later) — one lane: a hero's run state. `state` live · rests · waits. */
export type BloodlineLegacy = { points:number; spent:number; upgrades:Record<string,number> };
export type BossKnowledge = { boss: string; facts: string[]; ledger?: LedgerRow | null; wall?: BossWall | null };
export type DescentProgress = { tier:number; unlocked:number; cleared:number|null };
export type DescentOffer = { tier:number; hp_bonus_percent:number; attack_bonus_percent:number; stat_cap:number; affixes:ModifierInfo[]; elites:ModifierInfo[]; elite_rate_denominator:number; boss:ModifierInfo|null };
export type ClassStyleId = "sentinel" | "hexbinder";
export type ClassStyleOffer = {id:ClassStyleId;name:string;parent:string;level:number;xp:number;next:number;required_level:number;required_depth:number;deepest:number;selected:boolean;available:boolean;blocked:string|null;cooldown_ticks:number;duration_ticks:number;effect:string;tactic:Row};
export type ClassStyles = {selected:ClassStyleId|null;offers:ClassStyleOffer[];remove_available:boolean;remove_blocked:string|null;automatic_row:boolean;player_overrides:boolean};
export type HeroSlot = { specialization?:ClassStyleId; look?:string; hero_name?:string; build?:string; id:number; name:string; heir:number; class:string; level:number; xp:number; next?:number; state:"live"|"rests"|"waits"; live?:Lineage["live"]; rest_s:number; legacy:BloodlineLegacy; notice:boolean; chronicle?:string[] };
export type HeroLane = { id: string; name: string; state: "live" | "rests" | "waits"; depth?: number; hp?: number; max_hp?: number;
  rest_s?: number; run_id?: number; auto: boolean; need?: number; have?: number; kind?: "hero" | "expedition" };
export type ReturnReport = {
  pick?: ReturnPick;                                                           // Cut 113 §3 (core): the return's pick as it waits at camp (`Lineage.return_pick`)
  legacy_earned?: number;
  slice_pending?: boolean; // acknowledgement only; final slice reports the whole absence once
  bloodlines?: {legacy_earned?:number;id:number;name:string;runs:number;deepest:number;gold:number;packages?:string[];bests?:string[];boss_knowledge?:BossKnowledge[];xp?:ReturnReport["xp"][]}[];
  workers?: WorkerAct[];                                                       // Cut 30.5 (core): the workers' acts this absence (porter's hauls, apprentice's steps, clerk's deposits, …)
  chest?: number;                                                              // Cut 30.5 (core): the haul gold this absence left in the chest (before the porter; the chest's badge)
  grew?: GrewLine[];                                                          // Cut 30 §4 (core): what grew on each track over the absence — the report leads with it
  packages?: string[];                                                         // Cut 30 §1–2 (core): the packages' beats (`STEADY L3`, `DRILLED · Warlord`, `+Guarded`, `the pen`, `built bank`, `QUEST DONE · reach D10`)
  lead?: ReportLead[];                                                         // Cut 28 §2 (core): the report's first screen, ≤ 4, decisions first; salvage, bones and spent fold under `details`
  oath?: OathReport;                                                           // Cut 28 §1 (core): the sworn oath's absence; absent with none sworn
  elapsed_s: number; runs: number; sampled: boolean;
  deepest?: number;                                                            // the send's deepest floor (a delta, like the tiles beside it); absent on an old wire
  stalled?: number;                                                            // Cut 13 §1: sends that stalled (among `returned`, keeping nothing); the tiles count them apart
  spent?: { kind: string; n: number; gold: number }[];                         // Cut 13 §3: what the automations bought this absence, per kind (the SPENT section)
  heirs?: number[];                                                                           // QA 912e135 (core): the first and last heir who ran these runs (`♟2–17` under RUNS)
  gold?: { home: number; salvage: number; wake: number; spent: number; wake_cap?: number; wake_n?: number; lost?: number; unkept?: number; passage?: number; net?: number };   // blind ad71e72 (core): `passage` the waystone passages paid at the sends; `net` the purse's actual change over the absence (forge steps, hires and bank moves included)
    // QA 912e135 (core): `lost` — the carry the exits did not keep; QA 524827b (qaAA): `unkept` — the part of it exits that kept something left (a return's 40 %: `not kept`)
        // Cut 13 §3: the absence's movements to the coin (home + salvage + wake − spent = the header's delta)
  learned: string[]; bests: string[]; found: InvItem[]; deaths: { cause: string; n: number }[];
  pending: string[]; reel: Highlight[]; marks_earned: number; worst_death?: Death; worst_death_id?: number; live?: Snapshot;
  tamed: string[]; hatched: string[]; lost: string[];                        // Addendum A
  xp: { class: string; gained: number; level_ups: number };                 // Addendum C
  new_finds?: string[];                                                     // QA 0c6e126 (qaY; core): the kinds an absence found for the first time, every one (the header's `new find` lines) — FOUND; union across slices
  kept?: string[];                                                          // Cut 24 §5 (client, the watched report): what the keep sheet put in the vault, by its vault label (`strength potion`)
  driven?: number;                                                          // Cut 24 §1 (core): sends a boss drove off (`no counter`; among `returned`) — count them apart like `stalled`; sum across slices
  salvaged: { kind: string; n: number; gold: number }[];                    // Addendum D; `kind` an unidentified kind's flavour (`brittle scroll?`) until identified (QA 1a2a4a9)
  renown: { gained: number; rank: number; ranks_up: number };               // Addendum D
  rested_s?: number; banked?: number; returned?: number; bones_found?: string[]; // Cut 2 §1–2
  swapped?: number;                                                           // QA 778fa1b (core): carried gold the absence's pack swaps took (Σ `exits[].swapped`)
  stolen_gold?: number;                                                       // QA a946e04 (core, optional): carried gold thieves took this absence (Σ `stolen[].gold`: a stolen item's worth leaves the carry with it)
  start_short?: { depth: number; toll: number; runs: number };                // QA a946e04 (core): the waystone whose night pass the purse could not pay, its toll, and the sends that went from D1 instead (`D5 · toll $50 short · 3 runs from D1`)
  stolen?: { label: string; n: number; gold?: number }[];                                    // QA e75ec29 (qaR): what thieves took this absence and no run got back, per label (flavour-named while unidentified), most first; `gold.wake_cap` = the heir purse's top-up line ($40, each death tops up to it; `wake_n` deaths did), a death that found it full reads `purse full` on its exit line
  stall?: Stall;                                                              // core addition: stall verdict
  exits?: ExitLine[];                                                         // Cut 6 §1: one ledger line per exit in the batch
  picked?: number[];                                                          // Cut 16 §1: depths picked clean (≥ 3 banks/returns, shallower than the best), ascending — `D3 · picked clean`
  restock_capped?: boolean;                                                   // Cut 19 §3: the repeat skipped a supply once the absence's spending reached what it brought home — `restock capped`
  repeat_short?: boolean;                                                     // QA 1a2a4a9: a re-pack ran short of gold and bought what it could — `repeat short` (a `$0 repeat short` ledger line at that exit)
  shelved?: { kind: string; n: number; gold?: number }[];                     // (core: `gold` = their price on the shelf) Cut 21 §2: found supplies the exits put on the shelf this absence (the next send packs them free) — `heal ×3 → shelf`
  bounty?: { depth: number; taken: boolean; gold: number };                  // Cut 20 §5: the night's bounty floor — `bounty D12 · taken $412` / `bounty D12 · missed`
  drives?: DrivenOff[];                                                       // Cut 26 §6 (core): every drive-off of the absence (the last 5), each by `run_id` — open its verdict (`no counter`, defence, the counter row to insert) as a death's opens
  lanes?: string[];                                                           // Cut 26 §2 (core): each band the absence's sends reached, with the lane the route played there (`D5–8 · the Fens`), shallowest first
  night_marks?: number;                                                       // Cut 29 §1 (core): of `marks_earned`, the night's mark (◆1 per day whose absences brought a send home; the frontier mark is gone)
  systems_opened?: string[];                                                  // Cut 29 §2 (core): the systems this absence opened, in curriculum order — glint them (no text)
  oaths_kept?: OathReward[];                                                  // Cut 29 §1 (core): the extra slots' oaths kept (`label` the oath's text); `oath` stays the first slot's
  meters?: MeterWire;                                                         // Cut 29 §3 (core): the absence's real runs metered, summed (the report's per-night meter)
  fallen?: Fallen[];                                                          // Cut 29 §6 (core; AX: Greth gone with only `party −1 ogre`): each companion that fell, named — `Greth · ogre L5 · fell D12 to lurker`
};
export type Lineage = { bloodline?: BloodlineLegacy;
  guns?:{kind:string;selected:boolean;owned:boolean;price:number;available:boolean;blocked:string|null;
    capacity:number;range:number;damage:[number,number];armour_piercing:number;reload_ticks:number}[];
                        selected_loadout?: number[];
  hero_slots?: HeroSlot[]; selected_bloodline?:number; bloodline_price?:number; bloodline_cap?:number;
                        hero_legacy?: {name?:string; heir: number; points: number; runs: number; best_depth: number; class: string; spent?: number; upgrades?: Record<string, number> }[];
                        legacy_upgrades?: { id: string; rank: number; cap: number; price: number; effect: string; affordable: boolean; next_run?: boolean; name?:string; branch?:string; parent?:string; min_depth?:number; blocked?:string; owned_effect?:string }[];   // next_run (blind b58b431, core): away, the points buy it — `upgradeHeroNext` buys it for the next run
                        class_styles?:ClassStyles;
                        legacy_respec?: {refund:number;available:boolean;points_after:number|null;blocked:string|null};
                        return_pick?: ReturnPick;                                                                     // Cut 113 §3 (core): the return's pick waiting at camp
                        runs?: RunRec[]; live?: LiveRun | null; replays?: number[]; clock_s?: number; absences?: number;   // RUNS_UI (core): the runs log, the run under way, the run ids a replay is held for, the lineage clock (s), the absences counted
                        heroes?: HeroLane[];                                                                     // RUNS_UI: reserved for Cut 31 (a lane per hero); the client derives the one hero's lane until then
                        age_h?: number; reveal_queue?: string[]; reveal_next?: { id: string; trigger: string; triggered: boolean; wait_h: number };   // Cut 30 (core; PROGRESSION_V2 §4): the lineage's age in hours (offline included); systems ready and waiting their turn (one opens a report); the next to come and the hours it still waits (`next · tactics · 3 h`)
                        glory?: number; expeditions?: number; era_gate?: number;                                   // PROGRESSION_V2 §2 (core, reserved for Cut 31)
                        packages?: Packages;                                                                        // Cut 30 §2 (core): stances, tactics, temperaments, levels, drills, scars, the pen
                        town?: Town;                                                                                // Cut 30 §3 (core): the buildings, the next plot, the bank, the quest board
                        tracks?: Track[];                                                                           // Cut 30 §4 (core): character · items · scale · town — stage, next stage and trigger. Cut 30.5: superseded by `tree` (its stage nodes); kept until the client moves, then dropped
                        tree?: Works;                                                                               // Cut 30.5 (core): the works tree — workers, the four tracks as branches, the chest, the `next` pill. NB `gold` is the purse (collected); the chest is `tree.chest`
                        systems?: SystemInfo[];                                                                    // Cut 29 §2 (core): the curriculum — every system in order, `open` or not, its `trigger` (≤ 3 words), `new` since the camp last looked (`seenSystems()` clears); gate the editor's vocabulary and the camp's tiles by `open`
                        tier?: number;                                                                             // Cut 29 §1 (core): the catalogue's tier now (0–6: T1 the first bank, T2 the Warlord met … T6 the Lurker Queen met)
                        oath_slots?: number; sworn?: string[];                                                     // Cut 29 §1 (core): oaths that may be sworn at once (1–3, `oath_slot_2/3`); every sworn oath's id (first slot's first; `oath` is that one). An oath lapses unkept at its day's end
                        oath_draw?: OathDraw;                                                                      // Cut 29 §1 (core): ◆2 for a fresh standing oath (`drawOath()`), from T2
                        works?: string[]; commission?: Commission;                                                 // Cut 29 §5 (core): the lineage's works bought with gold, and the next (`commission()`; 10 forge units × 1.25ⁿ) — the late gold sink, policy-neutral
                        orders?: StandingOrders;                                                                   // Cut 29 §4 (core): the standing orders in one panel (`setOrders`)
                        supply_cap?: number;                                                                       // Cut 29 §6 (core; AX: `pack 4` bought, the shelf read 3/3): the shelf's cap — use it, not `supply_cap_5`
                        repeat_added?: RepeatAdd[];                                                                // Cut 29 §4 (core): kinds a player's `throw` rows name that the repeat lacks — offer each as one tap (`+ fire · for throw fire` → `buySupply(kind)`; the repeat keeps it after)
                        wall?: WallEdit;                                                                           // Cut 29 §1 (core; E1): the wall's edit on offer while the best depth holds
                        meters?: { runs?: MeterWire[]; night?: MeterWire; last_night?: MeterWire };                // Cut 29 §3 (core): the last two runs (oldest first — the camp's two-run comparison), this night's runs, the last full night's
                        seed: number; heir: number; trait: string; trait_offer?: string[]; class: string; look?: string; best_depth: number; marks: number;   // Cut 13 §2: `trait_offer` — two traits a new heir may wake with; `setTrait(name)` picks
                        facts: string[]; unlocks: string[]; vault: InvItem[];
                        graveyard: { heir: number; depth: number; cause: string; deeds: string[]; death_id?: number }[];   // death_id: Cut 9 §7, a kept death (`death(id)` answers)
                        trophies: string[]; sets: RuleSet[]; active_set: number; ended: boolean;
                        party: Companion[]; kennel: Companion[]; eggs: Egg[]; party_slots: number; ledger: LedgerRow[];  // Addendum A
                        gold: number; supplies: InvItem[]; insured?: number[];                                                              // Addendum B
                        classes: { [cls: string]: { level: number; xp: number; next?: number } };                     // Addendum C; next: QA 92eb880, the XP the next level costs (core's ladder; 0 at the top)
                        forge: { [kind: string]: { salvaged: number; craftable: boolean; tier: number;
                                                   next?: { need: number; label: string } } };                         // Addendum D; next: Cut 9 §10, the ladder's next rung (`3/5 → craftable`)
                        renown: number; rank: number; keep_pref: string;                                               // Addendum D
                        keep_auto?: string[];                                                                          // QA 23ed91f: what an unwatched exit keeps, in order, after the brought vault items (`["armour"]`, `["armour","weapon"]` with quartermaster, `[]` for none); a keep replaces only a weaker vault item of its own category
                        rest_left_s?: number; bones?: BonesPile[];                                                      // Cut 2 §1–2
                        endgame?: DescentProgress;
                        ascension?: Ascension;                                                                         // Cut 3
                        chronicle?: string[];                                                                          // Cut 5 §2: one line per ended heir, oldest first (cap 40)
                        vault_pref?: string;                                                                           // Cut 5 §4: what an unwatched vault choice takes (`weapon | armour | potion | scroll`)
                        ascended?: string[];                                                                          // Cut 5: variants the lineage has finished the dungeon with
                        gold_ledger?: GoldLine[];                                                                      // Cut 6 §1: the last 20 gold movements, oldest first (`ledger` is the bestiary)
                        counters?: Counter[];                                                                         // Cut 6 §5: bosses whose counter row is known
                        combos?: ComboHit[];                                                                          // Cut 8B §1: the active set's combos, in row order (recomputed on setRules)
                        picked?: number[];                                                                            // Cut 16 §1: depths picked clean now (as ReturnReport.picked)
                        class_offer?: ClassChip[];                                                                    // Cut 16 §2: the wake's class chips — owned classes, current first; present while trait_offer is and ≥ 2 are owned; `setClass(name)` picks (sticks until changed)
                        shadowed_by?: (number | null)[];                                                              // QA 92eb880: the active set's shadowed rows, as Forecast.shadowed_by (absent when none)
                        repeat?: boolean; repeat_kinds?: string[]; repeat_gold?: number;                              // Cut 19 §3: the loadout repeats (true unless cleared by `setRestock(false)`); the kinds the next send re-packs and their shelf price (`repeat · $120`); QA 1a2a4a9: a shelf of bought supplies is the next pack, so the badge is its price (the re-pack tops the shelf up, never more)
                        repeat_short?: string[];                                                                      // QA 1a2a4a9: kinds the last re-pack could not pay for (`repeat short`)
                        repeat_due?: string[]; repeat_unpaid?: string[];                                              // QA 778fa1b (core): of the repeat's kinds, those the shelf lacks that the next send re-packs at the send (`+heal at send`), and those the purse (after the toll) or the shelf cap will not let it — the tile reads `repeat short`. The exit's re-pack pays the price quoted at its send (the badge's), so `repeat_gold` is what is charged
                        bounty?: { depth: number; pays?: string; needs?: string; boss?: string; fact?: string };   // Cut 28 §1 (core): `pays` (`$×2 · item`), `needs` (`reach` · `slay mother`), and on a boss floor the boss and its counter fact (`mother: ?`) — `bounty · D13 · $×2 · reach`
                        waystones?: number[];                                                                         // Cut 21 §1: the lit waystones (a biome's first floor the lineage has banked at or past: 5 · 9 · 14 · 19 · 24 · 29), ascending
                        start?: number;
                        start_toll?: number;                                                                          // Cut 21 §1 (core): the toll the next send pays for `start` ($10 × start; 0 at D1)
                        repeat_dropped?: string[];
                        start_payable?: boolean;                                                                      // QA a946e04 (core, optional): the purse pays `start_toll` now (or tonight's pass is held); false → the next send starts on D1 (`start → D5 · $50 short`); absent → the client compares `gold` with the toll
                        start_pass?: boolean;                                                                         // QA a946e04 (core): the toll is paid once per night (16 runs) — tonight's pass for `start` is held, the next send pays nothing
                        trait_rules?: { [trait: string]: string };                                                    // QA a946e04 (core): each trait's real rule (`cowardly` → `backs off once a floor under 50%`), the chip's words
                        renamed?: { [label: string]: string };
                        kit?: KitLadder[];                                                                            // Cut 23 §1 (core): the forge — the heir's starting kit, a ladder per slot (weapon · armour · pack); `buyKit(slot)` buys the ladder's next step
                        row_why?: (RowWhy | null)[];
                        affixes?: { boss: string; depth: number; affix: string; effect: string; counter: string }[];   // Cut 116 §1 (core): the live heir's band-boss affixes, descent order
                        forks?: ForkChip[];                                                                       // Cut 26 §2 (core): the forks the hero has seen (fact `fork:<d>`), shallowest first — the chip line above the rows; absent until D4's two stairs are seen
                        lanes?: LaneStone[];                                                                         // Cut 26 §2 (core): every lit (lane, depth) waystone; `waystones` is now the active route's lit ones (the start sheet lists `lanes`)
                        oaths?: Oath[];                                                                              // Cut 28 §1 (core): the oath board — three standing oaths of the lineage (the sworn one flagged `sworn`)
                        oath?: string | null;                                                                        // Cut 28 §1 (core): the sworn oath's id (null: none)
                        titles?: string[];                                                                           // Cut 28 §1 (core): chronicle titles oaths earned (`Firebrand`), oldest first
                        oath_open?: boolean;                                                                         // Cut 28b (core): a band boss seen or a plateau met (or an oath sworn / kept before): the ladder carves the board
                        walls?: BossWall[];                                                                          // Cut 28 §1 (core): the band bosses from the Warlord to the first one past the best depth — each counter as a fact to learn
                        locked_rows?: (string | null)[] };                                                           // Cut 26 §6 (core): per row of the active set, the gate of its first cond this lineage cannot use (`see: den`, `enter fens`, `◆2`) — mark the row; absent when none                                                                // Cut 23 §3 (core): per row of the active set (by index), what it did over the recent sends and why not — null = no sends under this row yet                                                      // QA a946e04 (core): an identified flavour's old label (`blue potion?`) → its name now (`poison`); labels stored before read through it                                                                  // Cut 21 §2 (core): kinds the last send packed that no row uses — the repeat does not re-buy them (`strength · no row`)                                                                             // Cut 21 §1: where the next send starts (1, or a lit waystone; `setStart(d)`); the send pays `$10 × start` below D1                                                                 // Cut 20 §5: tonight's bounty floor (best + 2; moves every night)
/** Cut 16 §2: a class chip at the wake (`rogue · vanish`). `signature` is a verb id (`shield_bash | vanish | mark | slow`);
 *  `level` the class's level; `opens` the level the signature opens at (`mark L7` while level < opens).
 *  §4 (no new wire): the Warlord's break is a callout `warlord breaks` + a note `The Warlord breaks.` (visible only), once, at ≤ 50 % hp.
 *  §3 (no new wire): D5–8 are biome `burrows` (Snapshot.biome, descend.biome, fact `biome:burrows`). */
export type ClassChip = { class: string; signature: string; level: number; opens: number };
/** Cut 23 §1 — the forge: one slot of the heir's starting kit, bought with gold, permanent for the lineage (every heir starts
 *  with every owned step; a death loses none). `owned` = steps bought (0..steps.length); `steps[i].label` the kit at that step
 *  (`sword +1`, `mail`, `pack 4`), `price` its gold. `next` = the next step (absent at the top): `affordable` now; `nights` the
 *  core's estimate of nights of income until it is (0 = now; absent when there is no income to go on). With `kitDeltas()`:
 *  `delta` = the paired forecast move of buying it at `depth` (0..1, signed; the same paired panel as edits), `pm` its ±, and
 *  `bank`/`death` the ends' moves — the chip reads `mail +1 · D9 +7 · $340`. */
export type KitStep = { label: string; price: number; owned: boolean; kind?: string; rarity?: Rarity; branch?: string };   // Cut 113 §2 (core): an owned tier's branch
/** Cut 113 §2 (core): one of the next tier's two steps, priced as the tier (`buyKit("weapon:edge")`): `aim · edge`, `plate · pace`;
 *  `default` the ladder's own order, `lean` the branch the apprentice follows. */
export type KitBranch = { id: string; label: string; default: boolean; lean?: boolean };
/** Cut 113 §3 (core): the return's pick — one of three offers, sized by the absence (`size` 1–5), waiting at camp until taken
 *  (`takeReturnPick(id)`); `drill` (runs on a worn package, `title` its name) · `legacy` · `forge` (a discounted step, `price`) · `marks`. */
export type ReturnOffer = { id: string; title: string; line: string; price?: number; available: boolean; target?: string };
export type ReturnPick = { minutes: number; size: number; offers: ReturnOffer[] };   // run-clear (core): the piece the step forges and its rarity
export type KitLadder = { slot: "weapon" | "armour" | "pack" | "gun_sidearm"; selected?: boolean; owned: number; steps: KitStep[]; branches?: KitBranch[];
                          next?: { label: string; price: number; affordable: boolean; nights?: number; per_night?: number;   // QA 912e135 (core): the night's net `nights` divides by
                                   depth?: number; delta?: number; pm?: number; bank?: number; death?: number } };
/** Cut 23 §3 — a row's why-not over the recent sends (the rows the set still holds; a changed row starts over). `sends` the
 *  sends it sat in, `actions` the hero's actions over them, `fired` the actions it took, `matched` the actions its conds held
 *  (whether or not a row above acted first). `blocked`: the most common reason it did not act when its conds held and it was
 *  reached (`no scroll`, `no target`, `same as R2`, `row guard`), with its count; `unmet`: its most common failing cond (`gas`
 *  for `foe: gas` never in view — `no gas met`). `text`: the core's line for the tablet (`0/164 · no gas met`,
 *  `3/164 · blocked · no scroll`, `41/164`), ≤ 5 words. */
export type RowWhy = { sends: number; actions: number; fired: number; matched: number;
                       blocked?: { why: string; n: number }; unmet?: { why: string; n: number }; text: string };
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

/** Cut 28 §1 (core) — an oath's reward, never a stat: `card` (a tactic card, `id` its unlock), `slot` (a party slot), `row` (+1 row), `title` (a
 *  chronicle title and a trophy), `waystone` (a lit waystone, `id` its depth), `verb` (a verb or cond unlock). `label` ≤ 3 words (`card: gas step`,
 *  `+1 party`, `title: Firebrand`, `waystone D14`, `verb: throw`). */
export type OathReward = { kind: "card" | "slot" | "row" | "title" | "waystone" | "verb" | "route" | "heir"; id: string; label: string };   // Cut 29 §1 (core): `route` (route2: the D9 fork seen) · `heir` (heir_pick: three traits at the wake); the pool is the oaths' own (never sold)
/** Cut 28 §1 (core) — one standing oath: a constraint (`chips`, ≤ 3 words each: `D10` `no drink`; `text` = chips joined ` · `) and a reward that is not
 *  a stat. `price` the gold swearing costs (priced from the lineage's income; spent into the reward when kept; forswearing refunds half). `kind` the
 *  pool entry (`bold` · `tamer` · `fire` · `slayer` · `lean`). `boss` / `depth` / `counter` when it points at a band boss:
 *  `counter` the boss's counter as a fact (`mother: fire`, or `mother: ?` while unknown — met, the counter is learned). */
export type Oath = { id: string; kind: string; chips: string[]; text: string; reward: OathReward; price: number; sworn?: boolean;
                     boss?: string; depth?: number; counter?: string };
/** Cut 28 §1 (core) — the sworn oath on the camp's panel: `share` the sends that keep it (0..1) with its ± (`pm`); `night` the chance a night of 16
 *  sends keeps it at least once. */
export type OathShare = { id: string; text: string; share: number; pm: number; night: number;
  steps?: { k: string; share: number }[] };   // Cut 28b (core): the steps toward it (`D13` reached · `met` · `burned`), each the share of sends that got that far
/** Cut 28 §1 (core) — a band boss as the wall ahead: `fact` its counter as the lineage knows it (`mother: fire`; `mother: ?` unknown — `learn` says how:
 *  `meet her`); `row` the counter row (known only); `slain` once killed. */
export type BossWall = { boss: string; title: string; depth: number; slain: boolean; known: boolean; fact: string; learn?: string; counter?: string; row?: Row };
/** Cut 28 §2 (core) — one part of a forecast move (`forecastMove(sent)`): a state change since the send (`party` a pet died or joined, `kit` the vault /
 *  forge / shelf / loadout, `purse` the gold, `start` the start floor / waystones, `facts` facts learned, `heir` a new heir / trait / class / level /
 *  the floors' freshness) or the set's own edit (`route`, `rows`). `text` ≤ 3 words (`party −2 jackals`, `learned 3`, `new heir`, `rows`); `move` the
 *  paired move this part alone made (the state changed in the order listed — the parts sum to `ForecastMove.whole` exactly). */
export type MovePart = { kind: "party" | "kit" | "purse" | "start" | "facts" | "heir" | "route" | "rows"; text: string; move: ForecastVs };
/** Cut 28 §2 (core) — the camp's move against the set sent, attributed: `whole` = the active set on today's lineage less the sent set on the lineage at
 *  its send (paired seeds); `parts` in order (state first, then `route`, then `rows`), summing to `whole`; `lead` the kind whose headline move is the
 *  largest; `rows` whether the set's rows differ from the sent set's (run the divergence scene only then, and only for the `rows` part's move);
 *  `state` whether any state part is present. Null when the core recorded no send (a fresh lineage, an old save). */
export type ForecastMove = { whole: ForecastVs; parts: MovePart[]; lead: string; rows: boolean; state: boolean; sims: number; refined: boolean };
/** Cut 29 §2 (core) — one system of the curriculum. ids: send · dial · headline · edit · death · exits · loadout · unlocks · reorder · vs ·
 *  tags · party · cage · walls · divergence · forge · start · route · oaths · automations · route2 · heir_pick · class. */
export type SystemInfo = { id: string; open: boolean; trigger?: string; new?: boolean };
/** Cut 29 §1 (core) — the oath draw: ◆`cost`, `available`, or the gate while shut (`meet Warlord`, `◆1 more`). */
export type OathDraw = { cost: number; available: boolean; needs?: string };
/** Cut 29 §5 (core) — the next commission: its price, the work it builds, the purse covers it. */
export type Commission = { price: number; label: string; available: boolean };
/** Cut 29 §4 (core) — the standing orders: exit keep (`best_weapon|best_armour|none`), an unwatched cage's pick (`weapon|armour|potion|scroll`),
 *  the start floor, the repeat, insuring the brought items when the purse covers it (on by default). */
export type StandingOrders = { keep: string; cage: string; start: number; repeat: boolean; insure: boolean; forge?: string; wall?: string };   // Cut 114 §3 (core): `wall` the scout's order at a wall — bank (default) · carry · push   // blind 1fb7786 (core): `forge` the apprentice's order — half (default) · all · off
/** Cut 29 §4 (core) — a kind the next send adds to the repeat, and the row's verb that wants it (`throw fire`). */
export type RepeatAdd = { kind: string; row: string };
/** Cut 29 §1 (core; E1) — a wall's edit: the floor, the edit labels (`drop R6`, `R1 → hp < 90% → rest`), the whole set with them, the share of
 *  `sims` sends past the record before and after. */
export type WallEdit = { depth: number; edits: string[]; rules: RuleSet; before: number; after: number; sims: number; start?: number };   // `start`: the lit waystone the offer was measured from (its first edit `start D24`; apply sets it)
/** Cut 29 §6 (core) — a companion that fell: `why` `fell D12 to lurker`, the heir it served. */
export type Fallen = { name: string; kind: string; level: number; depth: number; why: string; heir: number };
/** Cut 29 §3 (core) — a meter (a fight, a run, a night): totals by side (`dealt`/`taken`: hero · pets · foes), per game second (`dps_*`),
 *  healing by source (`healed[].per_s`, `hps` all of it), the time split in ticks and seconds (fight · travel · chores · rest), each row's
 *  fires and share of all rule activations (`row` 0-based; −1 a trait's or card's own step, −2 a chore), supplies used by kind, gold home
 *  and per game minute, blows taken by the hero and by his pets, fights begun. Units: always print them (`12 dps`, `3 hp/s`, `$40/min`). */
export type MeterSides = { hero: number; pets: number; foes: number };
export type MeterWire = { seconds: number; dealt: MeterSides; taken: MeterSides; dps_dealt: MeterSides; dps_taken: MeterSides;
                          healed: { src: string; total: number; per_s: number }[]; hps: number;
                          time: { fight: number; travel: number; chores: number; rest: number }; time_s: { fight: number; travel: number; chores: number; rest: number };
                          rows: { row: number; fires: number; share: number }[]; // share of rule activations; several can belong to one action
                          actions: number; supplies: { [kind: string]: number };
                          gold: number; gold_per_min: number; hits_hero: number; hits_pets: number; fights: number; credit?: CreditShare[] };
export type SnapMeters = { run: MeterWire; fight?: MeterWire; fighting?: boolean };
export interface Engine {
  // Cut 30.5 (core): the works tree — each returns the Lineage (throws with a ≤ 3-word reason). The chores by hand are the existing calls:
  // send (before the scout) · loadout (wear a find) · buyKit · keep (a keep sheet) / sellVault · bankDeposit · spendLevel · setParty / hatch ·
  // swapQuest · setStart — each fills its node's count (`WorkNode.count`) until its worker is hired.
  selectBloodline?(id:number):Lineage;
  addBloodline?():Lineage;
  upgradeHero?(id: string): Lineage;
  upgradeHeroNext?(id: string): Lineage;   // blind b58b431: away, a Legacy upgrade for the next run
  respecLegacy?():Lineage;
  setSpecialization?(id:string):Lineage;
  buildTown?(id: string): Lineage;
  hire?(id: string): Lineage;                           // hire the lit node's worker (`Works.lit`; its price from the purse, then the chest)
  openChest?(): Lineage;                                // the haul chest into the purse (`Works.chest` → `gold`); the porter's chore
  setWorker?(id: string, on: boolean): Lineage;         // switch a hired worker off (its chore by hand again) or back on
  promote?(id: string): Lineage;                        // week 2: the worker rank on offer (`Works.lit_rank`; II 5 days after the hire, III 9; a forge unit × the rank less one)
  // RUNS_UI (core; docs/RUNS_UI.md): runs go on while the app is open, and any held run replays
  advance?(elapsedMs: number): Advance;                 // the open app's clock: rest, then runs, unwatched; a run in flight stays in flight
  replay?(runId: number): Replay | null;                // a past run re-simulated from its send (null: no capsule held for it)
  // Cut 30 (core): packages, drills, the bank, the quest board — each returns the Lineage (throws with a ≤ 3-word reason)
  equipPackage?(id: string, slot: number): Lineage;     // §2: a stance, a tactic in `slot` 0/1, a temperament — free, instant
  unequipPackage?(id: string): Lineage;                 // §2: empty a tactic or temperament slot (the stance is never empty)
  pickTemperament?(id: string): Lineage;                // §2: take a wake card (`packages.offer`)
  spendLevel?(id: string): Lineage;
  setTacticVariant?(id: string, variant: number): Lineage;
  takeFix?(id: string, variant: number): Lineage;       // Cut 115 §4: a death's tactic fix (`Death.pick`), credited taught
  takeControl?(on: boolean): void;                            // take control (a secondary mode): the watched run's hero is the player's
  act?(action: ManualAct): void;                              // take control: his next action (the world waits for it — `Snapshot.awaiting`)   // Cut 111: a tactic's L3 row, `Package.variants[variant]`                     // §2: marks for a package's next level (`Package.level_price`)
  revokeDrill?(boss: string, revoked: boolean): Lineage;   // §1: revoke (or restore) a drill — one tap, it stays
  packageOptions?(sims: number): PkgOption[];           // §2: every package move priced on the paired panel (slow: background lane)
  packageOptionsFor?(sims: number, choices: [string, number][]): PkgOption[];
  packageOptionsForKey?(sims: number, choices: [string, number][]): string;
  packageOptionsKey?(sims: number): string;            // Rust-owned complete query inputs; read on the same mirror as prices
  bankDeposit?(amount: number): Lineage;                // §3: deposit (capped at `town.bank_cap`)
  bankWithdraw?(amount: number): Lineage;               // §3
  swapQuest?(): Lineage;                                // §5: the day's free swap
  drawOath?(): Lineage;                // Cut 29 §1: ◆2 — a fresh standing oath (`Lineage.oath_draw`)
  forswearOathId?(id: string): Lineage;   // Cut 29 §1: forswear the sworn oath `id` (either slot; half back)
  commission?(): Lineage;              // Cut 29 §5: the next work, for gold (`Lineage.commission`)
  seenSystems?(): Lineage;             // Cut 29 §2: the camp showed the newly opened systems (clears `systems[].new`)
  /** Cut 29 §1 (core; E1): at a best depth held 2 days, the one-row edit that passes it — offer it as a patch (`rules` is the whole set with it);
   *  null off a wall or when no edit passes. Was `ReturnReport.wall`: the search is seconds natively, up to minutes in wasm, so it left the
   *  offline run — call it after the report paints (background lane); searched once a day, then cached (`Lineage.wall`). */
  wallEdit?(): WallEdit | null;
  setOrders?(orders: StandingOrders): Lineage;   // Cut 29 §4: the standing orders at once (each through its own rules)
  forecastMove?(prev: RuleSet): ForecastMove | null;   // Cut 28 §2: the move against `prev` (the set sent) attributed to state and rows — background lane, after the forecast (2–6 extra panels when the state changed)
  swearOath?(id: string): Lineage;     // Cut 28 §1: swear a standing oath (pays `price`; one at a time — swearing another forswears the first)
  forswearOath?(): Lineage;            // Cut 28 §1: forswear the sworn oath (refunds half its price)
  newLineage(seed: number): Lineage;
  load(save: string): Lineage;  save(): string;
  vocabulary(): Vocabulary;
  setRules(set: RuleSet): void;
  loadout(itemIds: number[]): void;
  forecast(): Forecast;
  forecastVsEstimate?(prev: RuleSet): ForecastVs;
  forecastEstimate?(): Forecast;       // small, explicitly rough UI preview
  send(): Snapshot;                    // start (or resume) an expedition
  step(turns: number): StepResult;     // advance live view
  runOffline(elapsedS: number): ReturnReport;
  runOfflineQuick(elapsedS: number): ReturnReport;   // no worst-death verdict (~3 s saved per slice)
  runOfflineSlice?(elapsedS: number, last: boolean): ReturnReport;   // pending acknowledgements before `last`; final report contains the whole absence
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
  setLook?(look: string): Lineage;     // hero looks: the heirs' cosmetic look (`male | female | cat`; optional: an older core has none)
  setKeepPref(pref: string): Lineage;   // core addition (README): keep preference for offline exits
  sellVault?(id: number): Lineage;     // QA 308f045 (core; qaAC: `vault full · axe stays`, no way to take it out): the item out of the vault, salvaged at a bank's share
  insure(id: number): Lineage;          // core addition: gold bet that keeps a brought vault item on death
  // core additions (crates/riddle-core/README.md)
  unlocks(): UnlockInfo[];              // the catalogue; `available` = prereq + fact gate + affordable
  unlockDeltas(): UnlockInfo[];         // Cut 4: same catalogue with forecast `delta` computed (0.3–2 s); call after paint
  setClass(cls: string): Lineage;       // switch class (rogue needs the `rogue` unlock)
  selectSet(i: number): Lineage;        // pick one of the three saved sets; setRules writes the active one
  descentOffer?(tier:number): DescentOffer;
  beginDescent?(tier:number): Lineage;
  ascend(variant: string): Lineage;     // Cut 3: after the ending, a new lineage under a variant (keeps classes, kennel, vault, facts, rules)
  // Cut 5
  bail(): void;                         // §5: a `return` fires on the hero's next action as a chore; the rules are untouched
  choose(itemId: number): Snapshot;     // §4: take one item of the opened vault (`Snapshot.vault_choice.items[].id`)
  setVaultPref(pref: string): Lineage;  // §4: `weapon | armour | potion | scroll` — what an unanswered vault choice takes
  // Cut 6
  forecastRefine?(): Forecast;          // §9: the same forecast at 100 sims (optional; the player requests it through More samples)
  // Cut 19
  cageForecast?(refined?: boolean): CageOption[];        // §1: every cage preference's forecast for the active set (three extra camp panels, memoised; seconds in wasm — call when the picker opens or after the refine)
  setRestock?(on: boolean): Lineage;    // §3: the loadout's repeat on/off (off refunds the re-packed shelf; on re-packs an empty shelf now)
  // Cut 21
  setStart?(depth: number): Lineage;    // §1: where the next send starts (1 or a lit waystone)
  /** Cut 22 §3: the paired move of the active set against `prev` (the set as it was at the last painted forecast); optional —
   *  a core that puts `vs` on the forecast itself needs no call. Called after the forecast's first paint, never before it. */
  forecastVs?(prev: RuleSet): ForecastVs;
  forecastVsRefined?(prev: RuleSet): ForecastVs;   // client (lanes.ts): `forecastVs` on the background lane, behind the refine — the refined panels paired
  refineLane?: boolean;
  parallelForecast?: boolean;           // Cut 25 §4 (client, lanes.ts): a forecast asked while one is in flight runs beside it (an idle mirror) — the app asks at once                 // Cut 24 §4 (client, lanes.ts): the refine runs on a lane of its own — the app asks it beside an edit's first pass
  startForecast?(refined?: boolean): StartOption[];   // QA 308f045 (qaAD: `D1 · bank 90%` beside the shaft's 92%): `refined` — on the camp's pass
       // §1: D1 and each lit waystone measured for the active set (memoised; seconds in wasm — call when the picker opens)
  // Cut 26
  forkForecast?(fork: number, refined?: boolean): ForkOption[];   // QA 308f045 (qaAC: `fens D8 12%`, picked: `D8 16%`): `refined` — measured on the pass the camp shows, like `cageForecast`
    // §2: both stairs of a seen fork for the active set (one extra camp panel, memoised; seconds in wasm — call when the fork chip's sheet opens)
  // Cut 23
  takeReturnPick?(id: string): Lineage;   // Cut 113 §3: take one offer of the return's pick (`drill · legacy · forge · marks`)
  buyKit?(slot: string): Lineage;       // §1: buy the next forge step of `weapon | armour | pack` (gold; permanent; ledger `forge <label>`)
  kitDeltas?(): KitLadder[];            // full paired forecast, retained for detailed callers
  kitEstimates?(): KitLadder[];         // small depth/exit preview, with its actual uncertainty
  // Cut 27
  fold?(): FoldLine;                    // §1: right after `send()` — plays the floors the set clears ≥ 95 % (`Forecast.fold_to` at the send) and returns the fold line; the watch opens on `step.snapshot` (foreground: it ticks the live run)
  divergence?(prev: RuleSet): Divergence | null;   // §2: the edit as a scene against `prev` (the sent set) — background lane, after the forecast (reads its paired panels; ~one sim's cost)
}
export type UnlockInfo = { id: string; cost: number; owned: boolean; available: boolean; needs?: string;   // needs: Cut 2 §3, the gate still missing (absent once met); Cut 29 §1: a shut tier reads `meet Warlord` … `meet Queen` (`bank once` for T1); an automation short of gold `$N more`
                           tier?: number;                                                                  // Cut 29 §1 (core): the tier that opens it (0–6)
                           gold_only?: boolean;                                                            // Cut 29 §1 (core): an automation — bought with gold only (`buyUnlockGold`), at the Lich (T4); the catalogue sells no condition word (free vocabulary, owned with its gate) and no oath reward
                           delta?: number;                                                                 // Cut 4 §9: forecast reach delta of buying (0..1), tactic cards
                           rows?: Row[];                                                                   // Cut 6 §6: a card's rows / an automation's effect as a row
                           insert_at?: number;                                                             // Cut 12 §1: where a bought card's row goes; Cut 18 §5: with deltas, its best measured place (the old place before the engagement row, the top, before the first own row) — its `delta` is measured there
                           pm?: number;                                                                     // Cut 13 §5: the half-width of `delta`; within it the client reads `reach ~0`
                           stall?: number;                                                                 // QA 92eb880: the stall share's move at the card's best place (0..1, signed; same sims as `delta`) — the stall risk before buying; a best place never raises it > 5 pts unless every place does
                           situation?: string;                                                             // Cut 18 §5: a tactic card's foe tag (`kite_archers` → `ranged`, `gas_step` → `gas`), for `vs archers` beside `reach ~0`; absent on other unlocks
                           pinned?: boolean;                                                               // Cut 19 §3: the next `+1 row` (its prerequisite owned) — keep it on the camp's short list until bought
                           short?: boolean;                                                                // QA 1a2a4a9: on the short list (≤ 3, the pinned one included) — the core's choice from the lineage alone; the camp's shelf and the report's PENDING show these
                           auto_insert?: boolean;                                                          // QA e75ec29 (qaR): a tactic card's buy puts its row in the set (at `insert_at`) only when set — measured, reach not down at its best place, stall share up ≤ 5 pts, fewer than 3 card rows in the set; otherwise owned, not in the set (offer `add`)
                           next?: NextUnlock;                                                              // QA a946e04 (core): the chain's next step once this one is owned (`row5` → `row6`) and its prices — the sheet's `next ◆4 or $600`
                           gold_next?: number;                                                             // QA a946e04 (core): this card's own gold price after one more gold buy
                           carries?: string;                                                               // Cut 23 §3 (core): what a paid card holds that no typed row can (`verb: bash`, `pre-aimed: boss`), ≤ 3 words; absent on a card that carries nothing outside the vocabulary (those cost 0)
                           gold?: number };                                                                 // Cut 15 §2: today's gold price (`150 × cost × (4 + gold buys) / 4`); 0 when owned or free (not gold-buyable). A card short only of marks (`needs` = `◆N more`) buys with gold when the lineage has it

/** The Engine with every method returning a Promise: the wasm engine lives in a Web Worker. */
export type AsyncEngine = { [K in keyof Engine]: NonNullable<Engine[K]> extends (...a: infer A) => infer R ? (...a: A) => Promise<R> : Engine[K] };   // Cut 24: a flag (`refineLane`) stays itself
/** QA a946e04: the chain's next unlock — its marks, its gold price after a ◆ buy of this one (`gold`) and after a $ buy (`gold_after_gold`). */
export type NextUnlock = { id: string; cost: number; gold: number; gold_after_gold: number };
export type SupplyEntry = { kind: string; price: number; label: string;
                            needs?: string };                                                                         // Addendum B; needs: Cut 10 §3, why a supply is greyed (`◆ identify`), optional
