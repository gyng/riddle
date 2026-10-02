# Riddle — game UI target (UX, IA, and the skin)

*2026-09-24. Goal: the chrome stops reading as a web form and starts reading as a game —
Warcraft III / StarCraft II as the reference register: a sprite-based, skeuomorphic, diegetic
frame (carved stone and iron bezels, riveted plates, a portrait well, a command card, a
resource bar), on a phone held in portrait (400 × 800 CSS px). Every cohort has rated feel and
aesthetic 0.6 (rater T 0.3: "a black screen with a lit patch of olive tiles and one well-drawn
hero"; S: "monospace chrome").*

## 1. What the references teach (and what we take)

| WC3 / SC2 element | What it does for the player | Riddle's equivalent |
|---|---|---|
| **Console frame** (bottom third, carved per race) | Frames the world as a window; the chrome *is* the faction | A bottom console plate, the Warrens' carved umber stone with iron corner caps; it holds the watch controls and the camp's actions |
| **Portrait well** (animated unit face) | The unit you control has a face and a mood | The heir's portrait in a round well at the console's left: class sprite, hp ring around it, the trait as a small sigil |
| **Command card** (3 × 4 grid of ability buttons, hotkeys) | Every order is a tile in a fixed place | The rule list: each row is a carved tablet (`cond → verb`) in a fixed slot, R1 at the top; the verb is an icon tile at the row's right end |
| **Resource bar** (gold, lumber, food, top right) | Stock at a glance, always in the same place | Top bar: `$` gold, `◆` marks, `★` renown, depth best — iron plate, engraved icons |
| **Minimap** (bottom left) | Where am I, what is coming | The forecast as a depth column: a vertical shaft D1 → D(best+1), each floor a notch lit by its reach; bank/return/death as three small gems under it |
| **Tooltips** (framed, title + cost + body) | The rule of a button lives on the button | Sheets become framed tooltips/panels (parchment inside an iron frame); the unlock sheet is a tooltip with a cost line |
| **Idle worker / alert pings** | The game tells you what needs you | `pending` as a pulsing rune on the camp tab it concerns |
| **Victory / defeat screen** (framed banner, score tables) | The ending is an event | The death screen as a defeat banner (a torn crimson banner with the verdict word), the report as a score table on parchment |

What we do not take: RTS density (a phone at arm's length reads ~12 things per screen), hotkeys,
multi-unit selection, and anything that would need text to explain the frame.

## 2. Information architecture

Four places, one frame. The console and the top bar are the same objects on every screen; what
changes is the **well** (the area between them).

```
┌───────────── top bar (iron plate) ─────────────┐
│ ♟3 portrait-mini · $ 1 488 · ◆ 7 · ★ 3 · D12   │   always: stock and the lineage's reach
├────────────────────────────────────────────────┤
│                                                │
│                  THE WELL                      │   camp: the rule tablets + the depth shaft
│                                                │   watch: the dungeon (canvas), no chrome over it
│                                                │   death: the defeat banner over the frozen floor
│                                                │   report: the score parchment
├────────────── console (carved stone) ──────────┤
│ (portrait)  [ command card: 4 × 2 tiles ]  (big│   camp: edit · loadout · unlocks · ledger · send
│  hp ring    per screen                   action│   watch: fights · fast · ▶▶| · ⏸ · bail
│             )                             gem) │   death: patch tiles · edit · camp
└────────────────────────────────────────────────┘
```

- **One primary action per screen, always in the same place** (the big gem at the console's
  right): camp → `send`; watch → `⏸/▶`; death → the top patch; report → `camp`.
- **The rule list is the game's command card.** Rows are tablets with fixed slots; a card
  (tactic) is a tablet with a gold border; locked slots are chained; `+ drop one` shows a
  chisel on the tablet it would remove.
- **Numbers live on objects, not in sentences**: the forecast's reach is the light in the
  shaft's notch; `death 12 %` is the red gem's fill; the patch's `survives 92 %` is a gauge on
  the patch tile.
- **Sheets are tooltips with an iron frame and parchment body**, anchored to the tile that
  opened them, with a close stud (×) top-right — every sheet (Cut 15 QA: "no close control").
- **Camp's secondary objects** (vault, forge, party, ledger, chronicle) are console tiles that
  open panels; they leave the main scroll (rater X: "35-tile unlock wall … bought on a hunch").

## 3. The skin (sprite-based, 9-slice, per biome)

- **Frames**: 9-slice sprites (corners, edges, centre) for: the console plate, the top bar,
  a panel (iron frame + parchment), a tablet (rule row), a tile (button), a gem button (primary),
  a portrait well, a gauge. Authored at 2× in the keyed register's ink-and-wash, packed into the
  atlas beside the sprites; CSS `border-image` from the atlas (or pre-cut PNGs) in the client.
- **Per biome tint**: the stone takes the biome palette (Warrens umber, Burrows red clay, Fens
  moss, Crypt bone-grey, Foundry iron-red, Deep blue-black, Sanctum pale gold) by the same
  palette pass the tiles use.
- **Type**: a carved serif display face for titles and verdicts (Google Fonts: `Cinzel` or
  `IM Fell English SC`), the pixel face for in-world callouts (kept), a condensed sans for
  numbers (`Barlow Condensed`). Monospace goes.
- **Motion**: tiles depress (2 px, darker) on tap; the primary gem pulses when an action waits;
  panels unfold (120 ms scale-y from the anchoring tile).
- **Art never blocks the game**: every frame has a CSS fallback (the current flat styles).

## 4. Targets (Codex mockups, `art/ui/targets/`)

Four portrait mockups at 1024 × 2048, one per place, painted in the frame above: `camp.png`,
`watch.png`, `death.png`, `report.png`. They are *targets*, not assets: the client track
matches their layout, frame language and hierarchy; the assets are cut separately
(`art/ui/frames/`, §3).

## 5. Progression of the interface — a reveal ladder

Today a fresh camp shows everything at once: two traits, three set tabs, the forecast with
killers, party, ledger, chronicle, vault, forge, keep/vault preferences, a 35-tile unlock wall.
Rater X: "the unlock wall has about 35 tiles, and I bought most of them on a hunch". The frame
from §2 lets the chrome *grow*: a console tile is carved in when it first means something, with
a one-beat reveal (the tile unchipped from the stone, a glint), never a tutorial sentence.

| Step | Trigger (engine fact) | What appears |
|---|---|---|
| 0 | fresh lineage | top bar ($ only), 2 rule tablets, the shaft (D1 only), gem `SEND`; nothing else |
| 1 | first death | the verdict banner and patch tiles; the `edit` tile |
| 2 | first gold carried home | `$` fills; the `loadout` tile (supplies) |
| 3 | first mark (◆) | `◆` on the bar; the `unlocks` tile — showing only the next **3** affordable-soon unlocks, not the catalogue |
| 4 | an item worth keeping found | the `vault` tile |
| 5 | first item salvaged | the `forge` tile |
| 6 | a tamed or freed companion | the `party` tile |
| 7 | a 3rd row owned | the shaft's gems (bank / return / death) |
| 8 | a second class owned | the class chip at the wake (Cut 16 §2) |
| 9 | 5 heirs | the `chronicle` and `ledger` tiles; set tabs 2–3 |

- **The unlock panel is a short list, not a wall**: the three next unlocks by the lineage's
  facts (what the last deaths and bests point at), each with its reason (`fact: ranged`), and a
  `more` stud that opens the full catalogue for players who want it.
- **Nothing is removed for a returning lineage**: the ladder reads the lineage's facts, so an
  old save starts fully revealed.
- **Gate**: a fresh camp at 400 × 800 shows ≤ 8 interactive elements; each step's tile appears
  on its trigger (a fake-engine walk); the dayplayer's `Days with ≥ 1 unlock` does not fall.

## 6. Acceptance (the UI cut's gates)

- Every screen has the top bar and the console; the primary action is in the gem slot.
- No screen shows more than 12 distinct elements above the fold at 400 × 800.
- Every sheet has a close stud; every tile has a pressed state.
- Frame time on the GPU harness holds 60 fps on the watch with the console drawn.
- `screens.mjs` / copy-lint unchanged in content; the skin is layout and art, not new copy.
- The reveal ladder (§5) holds on a fresh lineage.
- A blind rater's aesthetic and feel ≥ 0.8.

## 7. Auto-continue — idling never stalls on a button (2026-10-02)

The owner: "for idle, buttons need a timer to auto close". A screen or panel that waits for a tap closes or continues on its own,
with the countdown drawn on the thing it will press. One helper, `autoDismiss(el, { ms, onExpire, scope })` (`web/src/ui/autodismiss.ts`),
and a one-line hook per screen.

- **The mark**: a thin gilt ring drains around the element whose action the timeout takes — the gem, the `send again` chip, a
  console tile, a panel's close stud. No text in the chrome; under `prefers-reduced-motion` the ring stands full and carries the
  seconds left as a small number instead of draining.
- **What it may do**: continue, close, or press what that element already shows. Never a buy, an equip, a spend, an applied fix
  or a card pick (a send without a pick still takes card 1, as before).

| Where | After | It presses | Why that long |
|---|---|---|---|
| the report | 12 s | the gem, `camp` (the town) | the grew lines, ≤ 5 beat plaques and the tiles are ~12 things at ~1 s each; the plaques land in its first 2 s |
| the death screen | 12 s | the gem when it sends or goes to camp (lever `wait`, none); else `send again` when shown; else the `camp` tile | the what · why · what-now read is three lines; the lever `forge` / `wear` and a fix's `apply` are decisions, so the ring goes elsewhere |
| a death opened from the report or the chronicle | 20 s | `report` (back), else `camp` | the player opened it to read: a panel's time |
| an open sheet or camp panel | 20 s of no input | its close stud | long enough to read a full sheet, short enough that a phone left on the table is back in the town within a minute |
| beats and plaques | (3 s) | — | none waits for a tap outside the watch: the level-up and arrival beats are the report's plaques and share its clock |

- **Input** (a press, a key, the wheel, a scroll, a pointer move) restarts the clock; a press held down, a pointer resting on the
  ringed control or on the open panel, an open tooltip, an open sheet over a screen, or a busy engine pauses it.
- **A hidden tab** pauses it; on return the clock starts over full, so a report waiting for a returning player is always seen.
- **The watch** has none (its exit already walks on by itself); its sheets (the cage, the keep) have none.
- **Settings**: `auto continue` on / off, on by default (`riddle.autoContinue`).
- **Tests**: off under automation (`navigator.webdriver`) so the old gates keep their meaning; `?autodismiss=1` opts in,
  `?autodismiss=0.1` also runs the clocks 10× fast, `?autodismiss=0` turns it off anywhere. Gates: `web/tests/autodismiss.mjs`.
