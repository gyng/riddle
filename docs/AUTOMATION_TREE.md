# The automation tree — start manual, automate

*Design contract, 2026-10-02. No game code. The owner, after the Cut 30 preview: "progression is better but needs
more understandability to player"; "maybe at start, follow. tech tree can start from stuff like 'auto collect' 'auto
equip' etc". Asked to choose, the owner picked **start manual, automate**: early on the player does the chores by
hand, and the first nodes of a visible tree retire them, Melvor/NGU-style. PROGRESSION_V2 §3's automation ladder
moves to the front of progression. Inputs: AGENTS, HANDOFF §0, PLAN, IDLE_FIRST, PROGRESSION_V2 §3–4, CUT30 (Reveal,
§4 tracks, §6), TOWN, TOOLTIPS (branch `tips`, `0bfff46`), `systems.rs`, `town.rs`, `engine.rs` (restock, keep,
auto-keep, hatch), `kit.rs` (`unit_of`, commission), `meta.rs` (the Cut 29 gold automations), the owner-check pack
`scratchpad/ownercheck/sheet.png`. [U] marks a number we did not verify.*

## 0. The tension, and how this design resolves it

The owner has also said "a tap is a decision (no busywork)", and PLAN's pillar 2 reads *automate no-brainers
silently*. "Start manual" deliberately makes a few chores visible and manual. Three rules make that honest:

1. **A chore is manual only until the player has done it a few times** (3 for most), and its automation is bought
   within **minutes to hours, never days**. The manual phase teaches what the chore does. It is not a long grind.
2. **Each manual chore is one tap with a visible consequence**: coins arc into the purse, the sword appears on the
   hero's sprite, the bank's sign flips to look 2. The same tap fills the pip toward its automation (`2/3`).
3. **An undone chore never stalls the climb and never loses anything.** The hero goes down on his own whether or not
   you tap, so offline stays uncapped. The engine's own needs, the pack's restock and the insurance, draw on
   everything earned, collected or not. An undone chore only *defers an upgrade you would make by hand*: the haul
   waits in the chest, the forge step stays unbought. Nothing decays, expires or overflows (anti-Melvor, anti-Idleon,
   §1). The strongest form is a gate: **IDLE's 14 days are byte-identical with the tree in** (§5).

What an automation is worth, then: **an upgrade that happens while you are away** (the apprentice buys the step at
3 a.m., so the 4 a.m. runs are stronger), plus the taps it saves. That is the genre's honest value (AD's
autobuyers). It is not a penalty for being absent.

Most of these chores are already manual in Riddle today, just hidden in panels. The forge (`kit::buy`), the bank
(`bank_deposit`), wearing a find (the loadout), levels (`spend_level`), the quest swap, the party and the start stone
are all panel actions, and the IDLE bot does none of them. The tree **makes them visible as a sequence**. Only one
chore is new: the haul chest (§2, node 1). Two stay silent on purpose: the restock (the idle floor needs it) and
everything inside a run (pathing, pickup, in-run equip; PLAN's chores row, which is the sim itself).

## 1. Benchmark: manual first, then automate

| Game | The manual chore | Its first automation, and when it lands | What we take |
|---|---|---|---|
| **Cookie Clicker** | click the cookie | Cursor (auto-click) at 15 cookies, ≈ 15 clicks, **< 30 s**; Grandma at 100, ~1–2 min | the first automation within a minute; the store shows the next building as a priced silhouette |
| **Universal Paperclips** | `Make Paperclip`, buy wire | AutoClippers at $5, **~1–2 min**. WireBuyer appears **after you buy 15 spools by hand** (7,000 ops; ~30–60 min [U]) | the trigger is *having done the chore N times*: our pip counter |
| **Kittens Game** | gather catnip, refine it by hand | Catnip Field at 10 catnip, **< 30 s**; a hut (4 wood) → woodcutters, ~3–5 min [U] | a greyed button before you can afford it: the next step is always on screen |
| **Trimps** | Gather food/wood/metal; press Build | hire a Farmer for 5 food, **~1 min**; Foremen automate building; AutoFight at zone 1 cell 10, ~5 min [U] | workers *are* the automation: a person you hire does your chore |
| **Antimatter Dimensions** | buy dimensions | first autobuyer (1st dimension) at **1e40 antimatter**, ~20–40 min [U]; the rest by challenges after the first Infinity (hours) | the autobuyer tab lists the locked ones with their requirement |
| **Melvor Idle** | eat, loot | Auto Eat I costs **1,000,000 GP**, hours to a day [U]. Loot sits in a container that must be emptied by hand, and **past 100 stacks the oldest drop disappears** unless you wear the Amulet of Looting | *anti-example*: an undone chore costs you loot. Our chest never drops anything |
| **NGU Idle** | assign energy, fight Adventure | Idle Mode in Adventure as soon as it unlocks (boss 4, ~10–20 min [U]); Auto Advance from the EXP shop; rebirths 30–60 min | idle from minute one, with the toggles bought later; its menus are dense, so we take the curve, not the screens |
| **Idle Slayer** | jump and shoot (the run is automatic) | no auto-jump in the base game; players write mods and scripts for it | *anti-example*: a manual chore that never automates becomes a script target. Every chore here has a node |
| **Soda Dungeon 2** | fight turns | auto-battle as a toggle from the first runs; Soda Script (policy) after 1–3 resets | the chores get automated early, the *policy* late, which is exactly our packages → the pen |
| **Legends of Idleon** | drag-collect the AFK claim pile; pick up drops | Auto Loot (instant since v1.07). Ground drops despawn after **60 min (coins 20)**, and auto-loot never grabs the AFK pile ("a common silent resource loss") | *anti-example* again: the collect chore must never cost anything |

**How trees read.** *Factorio*: the first research **is** "Automation" (10 red packs [U], a few minutes in).
The research screen shows the prerequisites. The HUD shows the current research with a progress bar, and the early
tree is nearly linear. *Civilization*: one current research in the corner, with "N turns" and what it unlocks; the
whole tree is one tap away and you never have to open it. *Path of Exile's passive tree* (~1,300 nodes) is the
anti-example: new players plan it in external tools. **Rule taken: show the next node, the one after it and
silhouettes; never the whole tree at once.**

## 2. The tree

### 2.1 The chores in Riddle today (from the code)

| Chore | Today | Where |
|---|---|---|
| collect the haul | silent: gold credited at the exit | `engine.rs` exits, `gold_move` |
| keep the find | keep sheet at a watched, close exit; offline `auto_keep` by `keep_pref` | `keep`, `auto_keep_plan` |
| wear the find | manual: pick the vault item into the loadout (IDLE never does) | `start_run` → `loadout` |
| restock the pack | silent: repeat + quartermaster (the idle floor) | `restock_at` |
| insure what's brought | silent standing order (`insure`) | `StandingOrders` |
| send | gem or mouth; offline sends itself after the 20-min rest | `send`, `REST_MIN_TICKS` |
| buy a forge step | manual (sword marker over the forge) | `kit::buy`, `commission` |
| deposit to the bank | manual | `town::deposit` |
| level a package | levels arrive from runs; marks buy the next level by hand | `packages::spend_level` |
| swap the quest | manual, one free swap a day | `town::swap` |
| field pets / hatch | `set_party` by hand; eggs hatch after rests, or $50 by hand | `hatch`, `set_party` |
| pick the start stone | manual (waystones) | `set_start` |
| sell from the storehouse | at the keep sheet, by eviction | `keep`, `salvage` |
| identify | the hero learns by using in-run (`identify_used`); unknown finds are salvaged | `ai.rs` |

### 2.2 The first nodes: the trunk

One trunk, lit one node at a time. Each node: **the manual act → its worker → the trigger (do it N times by hand)
and the price → when it lands → the town hook**. Prices are in the forge's unit (`kit::unit_of`: it scales with the
record, so a price keeps meaning "about N runs' haul"). The dollar figures are illustrative at the early unit;
tune them in the cut. Times are the median engaged player (a 20–45 min first session watching ~2-min runs, then
3–4 check-ins a day); IDLE's column is in §2.4.

| # | Node | Manual act (one tap) | Automation | Trigger · price | Lands | Town hook |
|---|---|---|---|---|---|---|
| 0 | **Quartermaster** | none (owned at start) | packs the heal and the drill's item | given | 0:00 | the supply crate; a quartermaster beside it |
| 1 | **Porter** (auto haul) | tap the **chest** at the mouth: coins arc into `$`, the find glows, the report unfolds | hauls land in the purse as they come, offline included | 3 chests opened · **free** (the tree's first lesson is "done it 3×, it's yours") | **~min 8** | a porter with a handcart, mouth → fire |
| 2 | **Armourer** (auto equip) | tap the glowing find in the storehouse → the hero wears it next send (sprite changes, chip `+2 dmg`) | the better find is worn at each send; the worse goes back to the shelf | 2 finds worn · 1 unit (~$40) | **~min 20–45** | a weapon rack outside the storehouse |
| 3 | **Apprentice** (auto forge) | tap the sword over the forge → the next step (clang, the piece on the sprite) | buys the next step when affordable, keeping a reserve of one night's restock | 3 steps bought · 3 units | **~hour 1–3** (end of session 1 or check-in 1) | an apprentice at the anvil; blacksmith look 2 |
| 4 | **Keeper** (sorter) | the keep sheet at a watched exit, storehouse full: tap keep or sell (coins pop) | the sheet never asks: the standing order sorts watched exits as it already sorts offline ones (`auto_keep_plan`) | 2 sheets answered · 2 units | **~hour 2–8** (check-in 1) | the storehouse's shed grows a lean-to; a keeper sweeps |
| 5 | **Clerk** (bank sweep) | tap the coin over the bank → the purse above the reserve goes in | after each run, sweeps the purse above a reserve into the bank (to its cap) | bank built (age ≥ 8 h) + 3 deposits · 4 units | **~hour 12–24** (day 1) | a clerk at the bank door; bank look 2 |
| 6 | **Drillmaster** (auto level) | tap ◆ on the worn stance → its next level | spends marks on the worn stance's next level, then the tactic's | 2 levels bought · 3 units | **~day 1–2** | a training dummy by the tent; the hero spars at home |
| 7 | **Kennel-hand** | tap the paw → field the best pet; hatch a ready egg | fields the best pets, hatches eggs as they're ready | kennel built + 2 fieldings · 3 units | **~day 1–2** | a kennel-hand with a feed bucket |
| 8 | **Herald** (quest reroll) | tap the board → swap a quest the set can't keep | the day's swap, rerolling to one the set can keep | quests open (age ≥ 12 h) + 2 swaps · 2 units | **~day 2** | a herald at the notice board |
| 9 | **Guide** (start stone) | tap a waystone → start there | starts at the deepest lit stone the stance survives (forecast ≥ 50 %) | waystones open (age ≥ 20 h) + 3 picks · 4 units | **~day 2** | a guide with a lantern at the mouth |

**Every chore is automatable by day 2** (Y = 48 h, median); the first automation lands **by minute ~10** (X).
Node 1 being free is deliberate: it is bought the moment it lights, so the first thing the tree teaches is its shape
(do it → it's lit → it's yours). From node 2 on, every node costs gold, so buying it is a real decision against a
forge step. That keeps the rule that a tap is a decision.

**Deliberately not nodes:**
- **Auto-send.** The hero always goes down on his own after his rest; that *is* the idle floor. Making sending
  manual would cap offline progress, which is an invariant. The tap on the gem stays what it is today: *go now* (it
  skips the rest). The owner's "auto send" belongs with the **second hero** (Cut 31's house 1): a lane's hero is
  sent by a hand at the tavern, rung 11 below. Owner question 2.
- **Identify.** Facts are learned by the hero, never typed in by the player (an invariant). A "taste it at camp"
  tap would hand fact learning to the player. Out.
- **Restock, insure, in-run pickup and equip.** These stay silent: the idle floor depends on them, and they are the
  sim's chores, not the town's.
- **Quest reward claim.** It stays automatic. A claim tap would change IDLE's fortnight (IDLE gets the rewards today).

### 2.3 One tree: the trunk, then the four tracks as branches

```
                       ┌ prestige / policy crown (opt-in toggles, day 4+)
                       │  stance at the wall · wall edit as drill · auto-expedition · lanes · auto-era
     character ────────┤  items ──────────┤  scale ──────────┤  town ──────────┤
     2nd stance        │  storehouse       │  waystones (9)   │  blacksmith (3) │
     tactics → slot 2  │  forge steps (3)  │  party slots     │  storehouse (4) │
     temperament       │  a counter packed │  house 1 (Cut 31)│  kennel (7)     │
     classes, pen      │  armourer (2)     │  lanes (rung 11) │  bank (5)       │
     drillmaster (6)   │  keeper (4)       │                  │  herald (8)     │
                       └──────────── the trunk: 0 quartermaster → 1 porter → 2 armourer → 3 apprentice ────────┘
```

- **The trunk** is nodes 0–3: linear, minutes to hour 3, one lit at a time. It is the first session's whole goal.
- **The four branches are the four Cut 30 tracks** (character · items · scale · town), with the same stages from
  `town::stages` and the same triggers and age gates from `systems.rs`. Each automation node from 4 on sits on the
  branch of the building that hosts its worker. The tree **replaces the tracks panel**: it is that panel, drawn as a
  tree, with workers on it. One screen, not two (owner question 3).
- **The crown** holds PROGRESSION_V2's later rungs, unchanged in their gates: 6 stance at the wall (opt-in), 8 route
  follows the record (merged into node 9 above), 9 wall edit as a drill (opt-in, pen), 10 auto-expedition, 11 lanes
  (heroes auto-send), 12 auto-era. Policy rungs stay **opt-in toggles that name their rows**: the never-silent
  invariant. V2's rungs 1–5 and 7 are nodes 2–8 here, earlier (V2 had them on days 1–5; the chores now finish by
  day 2, the policy rungs keep their dates).
- **Queueing**: PROGRESSION_V2's one-system-a-report rule stays for systems. Nodes have their own budget: **at most
  one node lit at a time**; the next lights when the current one is bought, or when its own trigger is met if
  that comes later. A node whose trigger was met early waits lit, so at most 1 node is ever in "buy me" state.

### 2.4 IDLE, who never taps

IDLE never opens a chest, buys a node or does a chore, and still climbs: offline is unchanged, and the chest's gold
pays the restock and the insurance. Its purse counter shows collected gold only, with the chest's sum as a small
badge on the chest (`$1,240` waits). Nothing is lost, and the chest never caps. The tree still lights node 1 for
IDLE on its first collect, if one ever comes; otherwise the `next` pill keeps saying `open chest`, a goal that is
always one tap away.

## 3. Legibility

The player must always know three things: **what to do next · what the next unlock does · how far away it is.**

**A. The `next` pill**, one on the home screen, under the top bar. It replaces nothing on day 0, where it is the
fifth surface (Cut 30's day-0 cap of 4 interactive surfaces becomes 5: mouth, crate, tent, gem, pill).
Format: `icon · ≤ 3 words · the number`, for example `open chest`, `porter · 2/3`, `apprentice · $96/$120`,
`bank · in 3h`. Tapping it opens the tree on that node. It always shows the single next goal, in this order: an
affordable lit node, then the lit node's trigger count, then the next system from `reveal_next` with its wait. A
thin progress bar under the text, in the palette's gold.

**B. The tree screen** (the scroll sheet, UI.md). It shows **done nodes dimmed, the lit node full with its price,
the next two as outlined silhouettes with their trigger, the rest of the branch hidden**. Nobody sees past three
ahead (the anti-PoE rule). Each node is an icon (the worker's face), a name of ≤ 2 words and one state line: `done`,
`2/3 worn`, `$120`, or `bank · in 3h`. Tapping a node shows its tooltip: what it does in ≤ 10 words (TOOLTIPS:
`Wears the better find each send.`), and for a lit node a BUY gem.

**C. The town is the tree's picture.** Every bought node puts a worker on the scene (porter, apprentice, clerk…),
and its building steps a look. The staked plot (Cut 30 §3) gets a sibling: a **lit node's worker stands greyed at
their post** with the price marker above (`$120`), so the town shows what's next before any menu does.

**D. The beat.** Buying a node gives one beat, `AUTO HAUL`, `AUTO FORGE`… (≤ 2 words), the worker walking out of
the tent to their post, and one report line the first time the worker acts while you're away (`apprentice · +1
step`). Then it is quiet: workers' acts fold into the report's existing lines.

**Density caps** (the owner's density preference; TOOLTIPS' budget): home screen ≤ 1 pill, ≤ 12 elements above the
fold (Cut 30). Tree screen ≤ 7 nodes visible without scrolling on a phone, ≤ 2 words per node name, ≤ 1 number per
node, ≤ 1 BUY at once, ≤ 4 highlighted keywords per screen, tips ≤ 10 words that fade after 2 opens or 8 sightings.
No paragraph anywhere. Locked nodes beyond the next two are not drawn.

**Phone (400 × 800), home with the pill, and the tree:**

```
┌──────────────────────────────┐   ┌──────────────────────────────┐
│ ◉ 1st heir      $34       ⚙  │   │  WORKS                    ✕  │
├──────────────────────────────┤   │                              │
│   ▸ porter · 2/3   ▬▬▬▬░░    │   │        ◌ apprentice           │
│                              │   │          3 steps · $120      │
│          ╱▔▔▔▔╲  mouth       │   │        │                     │
│          ▏ ▓▓ ▕   ▣ chest !  │   │        ◌ armourer             │
│              ┊               │   │          2 finds · $40       │
│   ⌂ smith    ┊      ⛺ tent   │   │        │                     │
│   (porter    ┊               │   │        ● PORTER        2/3   │
│    greyed)   ✦ fire          │   │          ▬▬▬▬▬▬░░░            │
│                              │   │          "Carries hauls      │
│              ▦ crate         │   │           home."             │
│                              │   │        │                     │
├──────────────────────────────┤   │        ✓ quartermaster       │
│ (◉)  [forge]          ( SEND )│   │                              │
└──────────────────────────────┘   └──────────────────────────────┘
```

**Desktop (1440 × 900)**: the pill sits under the top bar, left of centre. The tree opens as a sheet anchored above
the building bar, the trunk vertical on the left (≤ 9 nodes) and the four branches as columns to its right, each
showing only done + lit + 2 silhouettes:

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ ◉ 5th heir  $1099  ◆29  ★3  best D7                                                ⚙ │
│   ▸ clerk · 2/3 deposits ▬▬▬▬▬░░                                                      │
│  ┌ WORKS ──────────────────────────────────────────────────────────────────────────┐ │
│  │ TRUNK          CHARACTER       ITEMS            SCALE          TOWN             │ │
│  │ ✓ quartermstr  ✓ 2nd stance    ✓ storehouse     ✓ waystones    ✓ blacksmith     │ │
│  │ ✓ porter       ● drillmaster   ✓ keeper         ◌ guide        ● CLERK  2/3     │ │
│  │ ✓ armourer       2 lvls·$90    ◌ a counter      ◌ party 2      ◌ herald         │ │
│  │ ✓ apprentice   ◌ temperament     packed                        ◌ house 1        │ │
│  └─────────────────────────────────────────────────────────────────────────────────┘ │
│                       [ town scene … ]                                               │
│      ┌ console: portrait · building bar · SEND ┐                                     │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

(The sheet's name is **works**: COPY's register, and `kit.rs` already says "works".)

## 4. The first 10 minutes

| Time | Tap | What the player sees |
|---|---|---|
| 0:00 | none | Camp: fire, tent, crate, the mouth. Pill `▸ send`. `$0`. The lone hero by the fire. |
| 0:05 | **SEND** (gem or mouth) | The hero walks to the mouth and goes in; the watch opens at D1. |
| 0:05–2:30 | none (watching) | Run 1: D1–D2, a jackal fight, home on `depth ≥ record + 1`. |
| 2:30 | none | Back at camp: a **chest** bounces by the mouth with a `!`. Pill `▸ open chest`. |
| 2:35 | **chest** | The lid flips, coins arc to `$` (count-up `$0 → $34`), dust. The blacksmith scaffolds and builds (first gold home). The report parchment unfolds (unchanged). A pip flies to the pill: `▸ porter · 1/3`. |
| 2:45 | **CAMP**, then **SEND** | The report closes; send. (The rest timer shows `heir rests 20m`; the send skips it.) |
| 5:10 | **chest** | `$34 → $71`; a sword glows in the chest and flies to a new **storehouse** (first find kept). `porter · 2/3`. |
| 5:20 | **SEND** | Run 3. |
| 7:40 | **chest** | `$71 → $112`; `porter · 3/3`. The pill turns gold: `▸ PORTER · free`. A greyed porter appears by the mouth. |
| 7:45 | **pill** | The works sheet opens on PORTER (lit, BUY). Silhouettes: armourer `2 finds`, apprentice `3 steps`. |
| 7:47 | **BUY** | `AUTO HAUL`: the porter walks out of the tent, takes the handcart, stands at the mouth. Pill: `▸ armourer · 0/2 worn`; the sword over the storehouse glints. |
| 7:55 | **storehouse**, then the **sword** | The sword moves onto the hero's sprite (`+2 dmg` chip). `armourer · 1/2`. |
| 8:00 | **SEND** | Run 4. The sword marker over the forge glints (`$40`): the forge, with its first step affordable. |
| 10:20 | none | The run ends; **the porter wheels the haul to the fire** and `$` counts up on its own. No chest. The report unfolds as before. The player has now done 3 chores by hand, retired one, and the next two are on screen with numbers. |

Taps in 10 minutes: ~11, each one a send, a collect that pays, a buy or a wear. PROGRESSION_V2's bar is "≥ 2 new
things in the first 10 min": here the blacksmith, the storehouse, the tree, the porter and the first worn find (5).

## 5. Gates and invariants

### 5.1 The bots (each must say what it does with the tree)

| Bot | Chores | Nodes |
|---|---|---|
| **IDLE** | none, not even the chest | none. **Its fortnight must be byte-identical to Cut 30's** (replay hash over 16 seeds × 14 days) |
| **HANDS** (new) | IDLE + at each check-in does every chore whose system is open, by hand: chest, wear the better find, forge step(s) as affordable, deposit above the reserve, level with marks, field pets, swap an unkeepable quest, pick the deepest survivable stone | never buys a node |
| **PICKED** | as Cut 30 (packages, forge, bank) and the chores by hand until automated | buys each lit node as soon as it's affordable, before a forge step; then stops doing that chore |
| **TUNED** | PICKED + the pen | as PICKED; the policy rungs (6, 9) switched on |
| **RANDOM** | random chores | buys a random affordable lit node at a check-in with p = ½ |
| **AUTO0** (probe, not a gate) | PICKED with every node owned at t = 0 | measures what the workers are worth between check-ins |

The dayplayer's session model gains the **first session** (V2: IDLE 20 min, PICKED 45, TUNED 90, watched runs) so
the minute-scale rows can be measured. A scripted client walk (`cut30.mjs` → `tree.mjs`) measures the same on the
real client.

### 5.2 New rows

| Row | Bar | Where |
|---|---|---|
| First automation (porter) bought | ≤ **min 12** on the scripted walk; PICKED median ≤ min 15 | `tree.mjs`; dayplayer session model |
| First paid node (armourer) | ≤ min 45, PICKED median | dayplayer |
| Every chore node lit · bought | lit by **48 h**, bought by 72 h, PICKED median; HANDS lit by 48 h | dayplayer |
| No node takes long | trigger-to-affordable ≤ 1 check-in (median), ≤ 2 (every seed) | dayplayer |
| IDLE unchanged | replay hash identical to Cut 30's, 16 seeds × 14 d | dayplayer + test |
| Nothing lost | purse + chest + bank + spent = the gold ledger's total, every check-in, every bot; the chest never caps | test, qa.rs |
| A chore never stalls the climb | IDLE with the chest opened at every check-in (a twin) is byte-identical to IDLE: collecting changes the purse's display, never the sim | dayplayer twin, test |
| Doing it by hand works | HANDS ≥ IDLE at D13/D18/D23 (never slower ± a check-in) | dayplayer |
| Automation pays | PICKED (nodes) reaches D18 sooner than PICKED-by-hand (PICKED with nodes off) on the median seed | dayplayer leave-one-out (`nodes` joins the systems S) |
| Not required | the nothing-required row (median ± a check-in, 48 h worst from day 5) with S = nodes | dayplayer |
| Taps per check-in | before the trunk is done ≤ 6 chore taps; from day 2 (PICKED) a check-in ≤ 3 taps (Cut 30's row, now with chores) | `tree.mjs` |
| One lit node at a time; never more than 1 BUY | pass | test, `tree.mjs` |
| Legibility | the `next` pill present at every stage; its number moves after each chore tap | `tree.mjs` |
| Owner check | the owner says what the tree is and what the next node does from screenshots alone; *clear* = yes | CUT30 §8 protocol |

The Cut 30 rows stay as gated. PICKED's buying shifts gold off forge steps early, so PICKED ≥ 1.5× IDLE is re-run
and must still pass; if it slips, tune the node prices, never the bar.

### 5.3 AGENTS.md edits (made in the merge commit, after the gates)

- "What the game is": *The engine does chores silently.* → *The town takes your chores: each one you do by hand a
  few times, then hire a worker for it; the hero's own chores in a run stay silent.*
- Loop diagram, line 1: `town (packages, buildings, the works tree, forecast; the pen late) → send → …`
- Hard invariants, new bullet: *A chore never stalls the climb: undone, it waits (the haul in the chest, the step
  unbought). It never decays, expires or is lost, and the restock draws on all gold, collected or not. IDLE's
  fortnight is byte-identical with the works tree in. Every manual chore has a worker by day 2; the first is
  free by minute ~10.*
- The never-silent bullet gains: *a worker is named on the scene and in the report the first time it acts; policy
  rungs are opt-in toggles that name their rows.*
- PLAN pillar 2: *Automate no-brainers silently* → *Automate no-brainers: by hand first, then a worker you hire;
  surface trade-offs.* PLAN's division table, Chores row: *the engine in a run (silently); the town between runs
  (by hand, then by workers)*.

### 5.4 Copy budgets (`eval/copy-budgets.json`, new classes)

| Class | Cap | Examples |
|---|---|---|
| node name | ≤ 2 words | `porter`, `apprentice`, `bank sweep`* |
| node state | ≤ 3 words + a number | `2/3 worn`, `$120`, `bank · in 3h` |
| `next` pill | icon + ≤ 3 words + a number | `porter · 2/3` |
| beat | ≤ 2 words, caps | `AUTO HAUL`, `AUTO FORGE` |
| node tip (TOOLTIPS) | ≤ 10 words, one clause | `Carries hauls home while you're away.` |
| report line | ≤ 4 words + number | `apprentice · +1 step` |

*Workers are named as people (porter, clerk); the beat names the act. Glossary rows (COPY §2): **works** (the tree),
**worker**, **chest**, **haul**. No tutorial text: the greyed worker at their post and the pill's counter are the
lesson.

## 6. Fit with the plan: Cut 30.5, before the owner check

**Recommendation: a short Cut 30.5 on `cut30-client`, before the formal Cut 30 owner check and cohort 25.**

- The owner's comment *is* their reaction to the Cut 30 preview. The check would re-ask a question they have just
  answered ("clear?" → not enough). Running cohort 25 on a build the owner already wants changed spends a cohort.
- It is small and mostly re-presentation. The tracks panel becomes the works tree (the same data: `town::stages`,
  `systems.rs`); the chores already exist as API calls (`kit` steps, `deposit`, `spend_level`, `swap`, `set_party`,
  `set_start`, the loadout). The workers are standing orders the core already knows how to keep (Cut 29's
  `StandingOrders`, `commission`). New core: `tree.rs` (a table like `SYSTEMS`: id, branch, chore counter, trigger
  count, price in units, min age, the worker's standing order), the chest (collected vs earned, one field), the six
  workers' acts in the batch, and the HANDS bot. New client: the chest, the pill, the works sheet (from
  `tracks.ts`), workers on the scene with primitive fallbacks. Art: 9 worker sprites (Codex, one frame plus a walk
  frame, fallback: a coloured figure with the node's icon).
- **It displaces**: cohort 25 waits for 30.5 (it scores the build the owner will judge). The Cut 30 owner check runs
  on 30.5, with the tree as a sixth system. PROGRESSION_V2 rungs 1–5 and 7–8 move **out of Cut 31** into 30.5, so
  Cut 31 shrinks to the expedition, house 1 and the second hero's lane (rung 11, which is where "auto send" lives),
  and the opt-in policy rungs. The hold before Cut 31 stands: 30.5 ends at the owner check, HANDOFF updated, then
  stop.
- Not in 30.5: the crown's rungs (6, 9–12), new buildings (library, tavern), the camera dive, any change to the
  pen, packages or the sim.
- Order: core `tree.rs` + chest + IDLE hash test (first, the invariant) ∥ client pill + works sheet against
  `fake.ts` → workers' acts + HANDS/PICKED bots → `gates.mjs --fast` then `--full` → the AGENTS/PLAN edits → owner
  check → cohort 25.

## 7. Open questions for the owner

1. **Is the first worker free, the rest bought with gold?** *Recommended: yes.* Free teaches the tree's shape in
   minute 8. Gold from node 2 on makes each a real choice against a forge step ("a tap is a decision").
2. **"Auto send": the hero keeps going down on his own; automatic sending arrives with the second hero (Cut 31).**
   Making the first hero's send manual would stop him while you're away, and offline must stay uncapped.
   *Recommended: yes. The gem stays "go now".*
3. **Does the works tree replace the tracks panel (the four tracks become its four branches)?** *Recommended: yes.*
   One screen answers "what's next" and "what's done"; two panels split the answer.
4. **Cut 30.5 now, before your Cut 30 check, or the front of Cut 31?** *Recommended: Cut 30.5 now.* You would then
   check one build that has the tree, and cohort 25 scores that build.
