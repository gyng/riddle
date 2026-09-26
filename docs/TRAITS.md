# Riddle — heir traits (design)

*2026-09-27. The owner: "add depth with a trait system plus proc-gen traits". Design only; the contract is
`docs/CUT30.md`, the prototype `crates/riddle-core/examples/traits_proto.rs` (§6). Evidence it answers:
cohort 23 AU "traits can backfire invisibly (curious read a recall scroll and wasted a $152 passage)";
cohort 9 "R4 retreat — brave held … two losses I could not own"; PATH_TO_95 expression/autonomy 0.6 on
every plateau card: "the facts pick the row, I only order them", "every run the same".*

## 1. The rule this design keeps

**A trait never acts. It changes what an action does, or what a condition sees — in a context.**
The four temperaments pick a verb over the player's row once a floor (`turn::choose_and_act`: the coward
steps back, the brave skips a retreat, the curious uses an unknown, the greedy grabs). That is policy the
player did not write (AGENTS.md: "policy is written"), and it is why they read as random backfires. A
generated trait is a *condition on the hero*, not a rule: a gift that is live in a context, and a cost.
The player's rows are the only thing that chooses; a trait changes which rows are worth choosing.

Consequences, each gated (§5): a trait can never cause an action the rows did not pick (no unseen
backfire by construction); its effect is a number in a context, so the forecast prices it and a
replay can remove it (the verdict can name it); what makes it *depth* is that the context is one the
rules steer into or out of.

## 2. Parts and the lexicon

A trait is **when × gift · cost**, drawn from parts. The engine owns the parts; names are composed.

| Part | Items (v1: 6 + 6 + 5 parts, plus 4 verb twists; the prototype measured 6 × 4 × 4 at double size) |
|---|---|
| **when** (the context the gift is live) | `hurt` (hp < 50 %) · `crowded` (≥ 2 foes adjacent) · `boss` (a boss in view) · `quiet` (no foe in view) · `deep` (a band: Fens and below; later a biome) · `first` (the first 20 turns of a floor) |
| **gift** (while live; *row-coupled* first — §6) | `mend` +1 hp / 8 turns (the hp that hp-rows read moves) · `rested` a rest heals double · `fury` +1 damage · `guard` +1 armour · `quick` +3 speed · `sure` a retreat or return step is free |
| **cost** (always on — shown at the wake) | `frail` −10 % max hp · `slow` −1 speed · `thin` a heal potion heals ⅔ · `dim` −2 sight (the Deep only) · none (rare tier only) |
| **twist** (a verb changed; rare, one per heir) | `iron gut` a malevolent drink harms half · `light hands` pick up takes no turn · `grudge` +1 damage vs the kind that killed the last heir · `hoarder` +2 pack, −1 supply |

Names: one word per (when, gift) pair, then the cost after a dot, the way item kinds read. The table is the
whole lexicon (20 heads × 4 costs); a head never changes meaning across heirs.

| | fury | guard | quick | mend |
|---|---|---|---|---|
| hurt | wrathful | stubborn | skittish | tough-blooded |
| crowded | brawler | back-to-wall | slippery | thick-skinned |
| boss | grudge-keeper | unbowed | duellist | defiant |
| quiet | — (rejected) | — (rejected) | restless | light sleeper |
| deep | fen-born | mud-hide | night-eyed | marsh-blood |

The row-coupled gifts take the same grammar: `rested` → `light sleeper` (quiet), `deep sleeper` (deep); `sure`
→ `sure-footed` (hurt), `light-footed` (crowded). The prototype's `hurt × mend` measured as `iron gut` in its
output; the lexicon's word is `tough-blooded`.

Examples on the chip: `wrathful · frail`, `night-eyed · dim` (sharp in the Fens, short-sighted above),
`slippery · slow`, `grudge-keeper`, `hoarder`. No coined handle for a mechanic that has a plain word
(COPY.md §1.3): the chip names the trait; the card under it reads the formula `[hurt] → fury +1 · frail`,
the way a rule reads `cond → verb` and an oath reads `[D3] [no rest] → ▤` (Cut 28b).

**Generator filters** (static, before any sim): a gift its trigger can never use is rejected (`quiet`
× `fury`/`guard`); a cost that cancels its own gift is rejected (`deep` × `quick` · `slow` is a wash in
the band, a loss above it — kept only if §5's build test passes); at most one twist per heir.
**Measured filter** (offline, per build, the generator's table — §5): a (when, gift, cost) shape ships
only if it passes the build test and the lever; the table is data (`traits.json`), not rolled at runtime.

## 3. Rarity, inheritance, agency

**Tiers.** *Common* (70 %): when × gift · cost. *Uncommon* (25 %): the gift one step larger (+2 / +2 /
+5 / 1 per 5 turns), same cost. *Rare* (5 %): when × gift, no cost; or a twist. *Marked* (not drawn):
born of a lineage event — the kind that killed an heir three times gives the next heir `grudge: ogre`;
a boss slain by an heir gives `unbowed` to one of the next wake's offers. Tier shows as the chip's rim
(no word).

**Two slots per heir.** *Blood* (inherited) and *born* (drawn). The first heir has one born slot; blood
opens at heir 3 (§7 reveal).

**The wake (agency with surprise).** When an heir wakes, the camp offers **three cards; the player picks
one for the born slot**, and decides whether the blood trait passes:
- card 1: a fresh draw; card 2: a fresh draw with a different `when` than card 1 (so the three never
  all ask for the same rows); card 3: the dead heir's born trait, **twisted** — one part re-rolled to a
  neighbour (`wrathful · frail` → `wrathful · slow`, or → `stubborn · frail`), labelled `twist`.
- Blood: the parent's blood trait **passes** by default; each passing, a trait whose gift never went live
  in the parent's runs **fades** one tier (rare → uncommon → common → gone: a trait that was never used
  leaves the family); one that fired in a banked run keeps its tier. A player may **cut** the blood trait
  (the slot empties for this heir; the next heir's born pick may be taken into blood).
- Every card shows its forecast price with the current set (`bank +4 · death −2`, paired) and the
  formula; the exact numbers of an unseen part read `?` (§4).
- **Bloodline** (lineage-level, at heir 5 and at each ascension): choose one `when` the family leans
  to — half of all future fresh draws take it. It is the lineage's long bet (a family of `deep`-blooded
  heirs wants deeper banks) and it is written into the chronicle (`the fen-born line`).

Nothing punishes absence: offline heirs take card 1 and keep blood (the defaults), exactly as the send
button does today with no pick; the report lists the heirs' traits (`heir 14 · slippery · slow`).

## 4. Learnable, legible, priced

- **Learnable.** A trait arrives as its name, its cost (visible at once: `max hp 29/36`) and its `when`
  as a word; the gift's size and exact trigger are a **fact** learned the first time it goes live
  (`fact trait:wrathful` = `hurt < 50 % → fury +1`), as item kinds are. Until then the card reads
  `[hurt] → fury ?`. Facts are learned; the rows that use them are written.
- **Legible in the watch.** The first turn a gift goes live on a floor the hero's plate stamps its word
  (`WRATH`, `GUARD`, `QUICK`, `MEND`, `RESTED`, `SURE`; 1 word), and a live gift keeps a small glyph on the plate; the
  trace carries a `trait` column (`fury +1` on every turn it is live).
- **The verdict can name it.** Death replays already rerun from t−10 (`trace.rs`); a trait death adds
  one counterfactual: the same replays with the neutral heir. When removing the trait changes the
  outcome in ≥ 6/12, the cause line leads with it (`frail · max 29 hp`), and the forecast's move
  attributes the share to `heir` (the Cut 28 attribution already has the part: `forecast.rs` `heir
  <trait>`). The verdict word stays one of the existing seven.
- **Rules can reference them.** Two conditions, unlocked with the blood slot: `if: trait <head>` (the
  heir has it — a set can carry rows for more than one heir) and `if: gift live` (the heir's gift is live
  now: `gift live → attack boss`; `hurt` × `wrathful` makes a hurt heir the strong one). The divergence
  scene and the patches treat them as any other condition.
- **Priced.** The forecast runs the heir's traits (they are hero state); the wake's cards are priced;
  the vs line shows a trait change as `heir` (Cut 28's attribution), never as a row move.

## 5. Guardrails (numbers; the prototype measures the first two)

| Guard | Bar | Where |
|---|---|---|
| **Lever** | every shipped trait's paired bank move < the set's best single-row edit on the written cohort set, and < the largest row drop on its plateau set (B0), on every cohort set; with the strongest trait owned, the Cut 25 lever row holds | `traits_proto`, metrics row |
| **Different build** | on ≥ 1 cohort set, the plateau search under the trait (from B0, one-row edits) ends ≥ 1 row from B0, and on fresh seeds that edit gains ≥ 3 pts under the trait and ≥ 3 pts more than it gains without it; else the shape is a number and the generator drops it | `traits_proto`, metrics row |
| **Bounded** | per trait on B0: \|Δbank\| ≤ 10 pts, \|Δdeath\| ≤ 8 pts | metrics |
| **Bots unchanged** | DEFAULT / EDITED / RANDOM / PASSIVE / LEARNED / FULL / PETS run with the neutral heir (`Trait::None`); plus DEFAULT with the strongest shipped trait still dies by D8 ≥ 80 % | gates |
| **Never unseen** | a trait never picks a verb (test: over 30 seeds × 8 h, every `acting_row` is a rule row or a chore; no `-1` trait rows); every live gift turn has a trace mark; its first live turn a callout | tests, qa.rs |
| **Dice** | `dice`/`luck` deaths ≤ 5 % unchanged with traits on | gates |
| **Determinism** | offers from `Rng::derive(seed, hash("trait_offer") ^ heir)`; twist and fade from the same stream; the replay hash covers the heir's traits | tests |

## 6. Prototype (examples only)

`examples/traits_proto.rs` ticks `forecast::sim_game` itself and applies the trait to the hero between
ticks (a status kept up while the context holds: `str_bonus`, `ward_t`, `speed_t`, hp; costs: max hp at
the start, energy drained per tick, `floor.vision`); the legacy temperaments are held off in every arm
(`trait_floor` pinned). Per cohort set (AU, AV · 631fe23; AW, AX · 9720ff7): the set's lineage after
8 h of its own sends, B0 its bank-optimal set (a 2-step plateau search), the lever; per trait × set:
the paired move on the written set and on B0 (128 sends), and a 2-step search from B0 under the trait,
confirmed on 160 fresh sends with and without the trait.

**Run** (seed 1, 4 sets × 66 shapes = 264 jobs, 8 threads niced, 53 min; raw: `scratchpad/traits/`
`run.txt`, `traits.json`). The sets' lineages: AU D13 bank 77 % (B0 = the set), AV D13 23 % → B0 85 %
(`rest at 90%` added: +62), AW D13 49 % → 92 %, AX D12 20 % → 91 % (the same `rest` row: +72). Paired
bank moves carry ± 8.7 pts (median, 128 sends); the confirmation ± 5.2.

| Finding | Numbers |
|---|---|
| **Stat gifts at these sizes outweigh rows.** Mean \|Δbank\| per shape 13–18 pts on the written sets; the controls (`always`, no cost) move AV/AX +48 to +80 (`hale`, `tough`: over the set's best row edit, +62/+72) | 2/66 shapes hold the lever *and* ≤ 10/8 pts on all four sets (`grudge-keeper · dim`, `fen-born`) — both numbers |
| **Costs make "builds", gifts do not.** Shapes whose search moved ≥ 1 row and paid ≥ 3 pts more with the trait: by cost `slow` 34/72, `frail` 19/72, `dim` 2/72, none 2/48; by gift 12–16 of 64–68 each (flat) | the loose gate passes 55/66 shapes on ≥ 1 set |
| **…and most of those are one notch.** The edit is a threshold (`bank at 30 → 40%`, `drink heal at 30%` moved up): a weaker heir banks earlier. On AU the same notch gains +12–15 without the trait too (B0 was not a true plateau at 64 sends) | 169/264 pairs differ by ≥ 1 row, 32 by ≥ 2; interaction mostly ≤ 1 × its ± |
| **Real builds: a trait that changes the truth of a row's condition.** `hale` (mend, always) on AW: the hurt-bank row never fires, the heir dives and dies (bank −80 on B0); the search writes `D14 → bank` (+74 with, +3 without). `light sleeper` (mend when quiet): the `rest at 90%` row goes redundant — dropping it costs −2/−6 with the trait, −75/−59 without (a row slot freed). `wrathful`/`stubborn`/`brawler` on AV: `retreat at 40% vs 2+` dropped (+4–8 with, +0–1 without) | 3 strict builds (neutral gain ≤ 5, interaction ≥ 8), all **mend**-shaped or retreat-shaped |
| `dim` is inert above the Deep (sight 7 in D1–13) | 2/72 builds |
| Rare no-cost shapes are numbers | 0/8 builds strict; `fen-born` +0–2 |

**Read.** A +2 stat in a context is a number the forecast prices; the player's answer is a threshold
notch, which is the "hill-climbing the number" players already report. What moved rows was an effect on
a **quantity a condition reads** (hp under regen → the hp exits; rest made redundant) or on what a verb
is worth (retreat under hurt-fury). Hence the generator (§2) draws gifts from *row-coupled* effects at
half the prototype's size, drops `dim` until the Deep, and ships only shapes whose strict build test
passes (interaction ≥ 2 × its ± on ≥ 256 sends, the neutral gain ≤ 5 pts). Not measured here: verb twists
(`iron gut`, `light hands`), class forks, the wake economy; bots and dice (neutral heir by construction).

## 7. Reveal (the progressive-unlock ladder, never day 0)

| Step | When | What appears |
|---|---|---|
| temperament gone | day 0 | heirs 1–2 wake with no trait (the neutral heir; the chip is absent) |
| `traits` | the first heir who dies after reaching D5, or heir 3 | the born slot; the wake's three cards; the first card is always a common with a cost that shows at once |
| trait facts | first live turn | the `?` on the card becomes the number |
| `blood` | heir 5 (the `heirs` step) | the blood slot; fade and twist; `if: trait` and `if: gift live` as unlocks (marks) |
| `bloodline` | the first ascension, or heir 12 | the family's `when` |
| twists, marked traits | a band boss slain / a grudge three deep | the rare shapes |

## 8. With classes, items, pets, oaths

- **Classes — specialisation forks.** A class verb takes the heir's live gift: `fury` → `cleave` /
  `backstab` / `volley` / `bolt` +1; `guard` → `bulwark` / `ward` a turn longer; `quick` → `kite` /
  `vanish` / `blink` a step further; `mend` → `second_wind` / `drain` +1. At class L5 one trait may be
  *set* into the class for the lineage (`brawler fighter`): its gift is live for that class's verb even
  outside its `when` — the fork is a school, not a stat (it only changes rows that use the verb).
- **Items.** Found weapons and armour can carry one `when × gift` (an *edge*: `axe · brawler`) from the
  same lexicon, learned as a fact on first live turn. A gift from two sources does not stack (max); a
  forged kit piece never carries one (the lever).
- **Pets.** A pet has one `when` (its temperament: a jackal is `crowded`, an ogre `boss`); a heir's gift
  extends to pets while both are live (`brawler` + a crowded jackal → both +1). Rows already scope
  `party`.
- **Oaths.** An oath may be sworn *for* a trait (`wrathful · D10 · no drink`: the build test as a
  goal), and an oath's reward may be a wake choice (a fourth card, a twist chosen instead of rolled) —
  never a trait's gift itself (rewards are not stats; Cut 28's rule).
- **The four temperaments** map onto the new shapes and leave the override code: `cowardly` →
  `skittish` (quick when hurt); `brave` → `unbowed` (guard vs a boss); `greedy` → `light hands`;
  `curious` → `iron gut`. Old saves keep the name, gain the shape; `Trait::rule` strings go.
