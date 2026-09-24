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

## 5. Acceptance (the UI cut's gates)

- Every screen has the top bar and the console; the primary action is in the gem slot.
- No screen shows more than 12 distinct elements above the fold at 400 × 800.
- Every sheet has a close stud; every tile has a pressed state.
- Frame time on the GPU harness holds 60 fps on the watch with the console drawn.
- `screens.mjs` / copy-lint unchanged in content; the skin is layout and art, not new copy.
- A blind rater's aesthetic and feel ≥ 0.8.
