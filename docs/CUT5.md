# Cut 5 — The story of a run, and the watch worth watching

*Contract, 2026-09-16. Cohort 1 (α 0.887, mean 63.7) lost most weight on `story` (A 0.3, B
0.6: "the reel is fragments with no turn in them"), then `surprise` ("D1–5 monsters repeat"),
`pacing` ("1× watching is dead"), `expression` ("two sets seen") and `clarity` ("money never
reconciles"; Cut 4 core fixes the unit). This cut is story and the watch.*

## Why the reel has no turn

A story needs a setup, a turn and an end that the *listener* can connect. The chronicle emits
notes as they happen (`Met a jackal.` … `Down to 2 HP.` … `Banked $313.`), and the sifter
scores single events. Neither knows what changed. The turn lives in the **relationship
between events**: the row that fired at the low point, the item that was gambled, the ally
that fell so the hero could bank. So the story layer must be built from *episodes*, not events.

## Design

### 1. Episodes (core: `sifter.rs` rewritten around arcs)

An **episode** is a contiguous span of a run with a low point and a resolution. The engine
tracks `run.arc`: hp low-water mark since the last resolution, the row that fired at it, the
foes present, items used, allies lost. An episode closes on: hp recovering above 60% after a
low ≤ 25%, a boss dying, the floor changing after a low, a companion dying, an exit. Each
closed episode yields one **story line** in three beats, ≤ 12 words:

`Two jackals took him to 3 HP; R2 drank; banked $58.`
`The Warlord rallied twice; R4 bashed him; first boss.`
`Uleth the rat fell holding the corridor; he reached D6.`

Grammar (data-driven, no free text): `<threat> <took|cornered|chased> him to <hp>; <row-or-
trait> <verb-past>; <resolution>.` where resolution ∈ {banked $N, reached Dn, first boss,
returned, died (cause)}. Rows are named `R2 drank` (verb past tense from a table), traits
`greed took the gold`. Companions by `kind name`.

The **reel** = the top 3 episodes of an absence by score (low-point depth × resolution
weight; deaths of named heirs and first bosses weigh most), plus the run that reached the
best depth. Never two episodes with the same threat and resolution.

### 2. The lineage chronicle (core + client)

`Lineage.chronicle: string[]` (cap 40): one line per **heir**, written when the heir ends:
`♟3 the greedy fighter · D7 · took the Warlord · fell to gas · left bones on D7.` The camp
gets a `chronicle` sheet (like ledger). This is the retelling the research asked for
("what would a player paste into a YASD thread"), and it makes heirs individuals.

### 3. The hero has a voice, sparingly (core → callouts)

At an episode's low point and resolution the hero says one line ≤ 3 words, from a table keyed
by trait × moment: greedy at a gold pile with foes adjacent `worth it`; cowardly at hp 25%
`not today`; brave cornered `come on then`; curious drinking unknown `let's see`. Emitted as
`callout` events (the renderer already shows them); max one per 100 ticks.

### 4. Surprise in the first five floors (core: content)

Warrens D1–5 were "the same three monsters". Add two situations per floor band that change a
run's shape without new monsters: **shrines** (altar tile: `pray` verb; a trait swap or a
+1 max row for the run at the cost of 20% max HP), **vaults** ("choose one of three" cage
with a rule-visible `vault` fact; the only direct interrupt, only when watched), **nests**
(a jackal den: 4 jackals asleep, a gold pile; `on_see: nest` token), **a lost heir's
companion** (a wild companion of a previous heir, tameable at 60%). Facts: `shrine`, `vault`,
`nest`, `stray`. Each is a token and each produces an episode.

### 5. The watch at 1× (client + core)

- **Skip is the default speed.** The watch opens at `▶▶|` cadence: it plays continuously but
  jumps dead stretches (no hostile in view, no item within 3 tiles, no hp change) at 8×, and
  drops to 1× when something is in view. Buttons become `slow · fast · ▶▶|` (1×, 4×, auto).
- **The camera leads**: when a hostile is remembered or in view, the camera frames both the
  hero and the nearest hostile (midpoint, clamped so the hero stays inside the middle third).
- **Bail is not a black screen**: bail = `return` fires on the next hero action; the run
  plays out to the exit at 8×, then the exit sheet.

### 6. Expression: sets have names and the class picker is a loadout (client)

The three saved sets get names the player types (≤ 12 chars, the only free text in the game),
shown in the tab and in the lineage chronicle line (`♟3 · "corridor" set`). The class picker
shows the verb ladder for each class as chips, so a player can see what a class *is* before
buying.

## Gates

| Gate | Bar |
|---|---|
| Story lines: setup + turn + end in ≤ 12 words, verb past tense from the table | unit test on 100 runs: 100% |
| Reel: ≥ 2 distinct (threat, resolution) pairs per 8 h absence | 30 seeds: ≥ 90% |
| Tell-a-friend proxy: the reel's top line names a row, a trait or a companion | ≥ 80% of absences |
| Situations: a shrine, vault, nest or stray appears in D1–5 | ≥ 1 per run in 90% of runs |
| Watch at auto cadence: median time between hostiles-in-view ≤ 20 s at 1×-equivalent | playtest walk |
| Cohort 2: `story` ≥ 0.6 from both raters, α ≥ 0.67 | two blind cards |

## Tracks

- **Core** (`crates/**`): §1, §2 (chronicle field), §3, §4, §5 (bail as a queued return).
- **Client/renderer** (`web/src/**`): §2 sheet, §5 cadence/camera/bail, §6.
- **Art** (`art/**`): shrine, vault cage, nest 8×8 props ×3 biomes of the Warrens/Fens/Crypt.
