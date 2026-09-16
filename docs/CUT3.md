# Cut 3 — Content to the bottom, and what comes after

*Contract, 2026-09-16. The 14-day simulation (Cut 2 outcome) has a competent player finishing
the 16-floor v1 dungeon on day 4–9; the two failing pacing bars are content bars. This cut
brings the dungeon to depth 30 (PLAN M7), adds ascension so the arc has a second act, and
re-enforces those bars.*

## Design

### Biomes 4–6 (each breaks the program that cleared the previous one)

| Floors | Biome | Trait that breaks the last program | Monsters (tags are facts) | Boss (learnable counter) |
|---|---|---|---|---|
| 16–20 | **Foundry** | melee is reflected, bells raise the clock | `iron_golem [reflect_melee]`, `forge_imp [fire, steals potions]`, `bell_sentinel [alarm: +2 alert on sight]`, `slag_crawler [heavy, leaves fire]`, `smith [buffer: armours allies]` | D20 **Foundry Master** `[reflect_melee, buffer]`: kill the smiths first, then range/throw |
| 21–25 | **Deep** | dark (vision 4), hunters track noise, regen | `lurker [blind, hunts noise: rest and fights draw it]`, `deep_eel [water]`, `cave_troll [regen 2/10 ticks; poison stops it]`, `siren [confusion aura 2 tiles]`, `mirror_shade [copies the hero's class verb]` | D25 **Lurker Queen** `[blind, summons lurkers on noise]`: no `rest` on her floor, `darkness`/`silence` scroll, or kill fast |
| 26–30 | **Sanctum** | variety is enforced: repeated verbs are punished | `warden [alternates reflect_melee / reflect_ranged, telegraphed]`, `acolyte [heals allies]`, `echo [splits on ranged]`, `sentinel [stuns on telegraph]` | D30 **Mirror King** `[mirror: any verb used 3× in a row is reflected]`: alternate rows (`on_hurt → vanish`, `on_kill → …`), tactic card `cadence` |

Reaching D31's stairs is **the ending**. D16 is no longer an ending (v1's placeholder).

### Items (15 new, all dual-use)

Weapons `spear (3-6, reach 2)`, `mace (4-8, stun 10% )`; armour `scale (+4, −1 speed)`;
potions `regen (2 hp / 10 ticks, 60)`, `resist_fire (60)`, `clarity (immune confusion, 60;
throw: clears confusion in 3×3)`; scrolls `recall (bank from anywhere — the escape of the
deep)`, `silence (no noise 100 ticks)`, `earthquake (walls in radius 2 become floor)`,
`mirror (reflect the next hit)`; misc `lantern (vision +2, occupies a slot)`, `bell (throw:
+3 alert in a room; lures hunters away)`, `salt (throw: undead flee)`, `chalk (marks a floor:
bone_sense-like path to stairs next visit)`, `mirror_shard (breeds the mirror_shade tag)`.

### Unlocks and marks (tier 2, needs a boss)

| id | cost | needs | what |
|---|---|---|---|
| row9 · row10 | 14 · 18 | boss 3 · boss 4 | rows (cap 10 at ascension 0) |
| vault5 | 14 | boss 3 | vault |
| party_slot_4 | 14 | tamed ≥ 6 | party |
| cadence · noise_discipline · reflect_read · deep_march | 5 each | a matching fact | tactic cards |
| lantern_rig | 6 | `item:lantern` | automation: lantern always brought (no slot) |
| recall_sense | 8 | `item:recall` | automation: reads recall at `hp<15%` if held (a free row) |
| studied_all_<biome> trophies | — | ledger | 3 marks each |

Marks: depth 16–30 = 1 each (15), bosses 3 × 3, ranks, trophies. With ~15 more unlocks the
14-day cadence bar becomes reachable; the dayplayer bars in CUT2 are re-enforced by this cut.

### Ascension (after the ending)

`Engine.ascend(variant)`: a new lineage that **keeps** classes (levels), kennel, vault, ledger,
forge, facts; resets marks, gold, rules stay (the player's), heir count restarts. Variants, each
adding a system rather than a multiplier (choose one; more unlock by finishing with each):

- `no_rest`: `rest` is not a verb; camp rest is halved. (Consumables and companions carry.)
- `short_list`: rows capped at 6. (Tactic cards carry.)
- `bones_only`: the vault is empty and cannot be brought; only bones piles carry gear.
- `hunted`: a grudge monster from the previous lineage stalks every floor from D3.

`Lineage.ascension: { level: number; variant: string }`; `ReturnReport`, `Snapshot` unchanged;
the ending screen's `again` calls `ascend`. Dayplayer: after the ending, ascend with `no_rest`
and keep counting days; the 14-day bars now include the second act.

### Facts and forecast

New tags become tokens as before. Forecast stays bounded by `best_depth + 1`. The boss
counter facts (`boss:<kind>:counter`) are learned from the first telegraph.

## Gates (added; CUT2's content bars re-enforced)

| Gate | Bar |
|---|---|
| TRIVIAL never passes D5; COUNTERED reaches ≥ D11 | unchanged |
| FULL bot (`presets/full.json`, the shipped best set with all counters, all unlocks) reaches ≥ D26 | ≥ 50% of seeds over 8 h × 3 batches |
| FULL without the D20/D25/D30 counter rows never passes the corresponding boss | ≥ 90% |
| Dayplayer 14 days incl. ascension: days with ≥ 1 unlock ≥ 10; longest stall ≤ 3 | enforced |
| Unfair deaths ≤ 5%; death-cause top share < 35%; events/600 ticks ≥ 6 | unchanged across all six biomes |
| Per-tick cost | ≤ 6 µs |

## Tracks

- **Core** (`crates/**`, presets, examples): everything above. Vision radius per biome
  (`Floor.vision`), noise model (rest/fight emits noise; hunters path to the last noise),
  reflect flags, regen, confusion aura, mirror counter, `ascend`, unlocks, gates.
- **Art** (`art/**`, `web/public/art/**`): 15 monsters + 3 bosses (keyed watercolour, tag in
  silhouette), three 8×8 tile sets (`foundry`: rust/ember/iron; `deep`: black/ink-blue/bone;
  `sanctum`: white/gold/slate) with the standard eight tiles + bones frames, palettes in
  `atlas.json meta.palettes`.
- **Client** (small; after core lands): verb/token labels for new verbs (`cadence` card, etc.),
  the ending screen's `again` → `ascend(variant)` with a four-chip variant picker (copy budget:
  one word per chip), `ascension` in the lineage strip (`↑2 hunted`).
