# Cut43 — clearer gun names and class symbols

Owner requests renaming long/short gun and class icons. Default names Rifle /
Scattergun. Internal long_gun/short_gun IDs
remain unchanged for saves, rules, art and forge purchases.

- Rifle/Scattergun in forge, inventory/floor item labels and played gun details;
  blocked action reasons/glosses use these names, retaining old-wire aliases.
- Shared class icon component covers Fighter/Rogue/Ranger/Caster/Gunner plus
  existing Sentinel/Hexbinder paths, primitive fallback for unknown/absent art.
- Painted silhouettes from existing assets; consistent class choices, active
  hero roster, hero detail class control and earned XP reports. Mobile sizing
  preserves readable labels and current desktop geometry.
- Verify actual assets/load, class selection/ownership persistence and paid gun
  selection through existing checks; real earned WASM400/1440 screenshots.
- Numeric gun mechanics/IDs/costs unchanged. No deployment or95claim.

Acceptance:

- 677 Rust tests passed / 1 ignored (48.07 s), including stable gun IDs and
  enchanted labels. Fast all-target clippy, TypeScript, copy lint (1734 tags,
  zero violations), real fast WASM (5,478,683 bytes), web build and diff checks pass.
- Actual earned Gunner save at 400/1440: five distinct painted class symbols
  decode, menu inspection leaves the complete save unchanged, no overflow or
  page errors. Fighter uses shield, Rogue shadow, Ranger bow, Caster magic,
  Gunner gun sight; picker icons are 30 px, roster 18 px / mobile 14 px.
- Forge displays Rifle / Scattergun. Actual paid Scattergun selection deducts
  exactly 275 gold (48,536 → 48,261), persists ownership and stable short_gun ID
  after flush/reload. Live watch announces Scattergun with played damage/range/
  reload details. No simulation tuning or grants used.
- Screens and actual-browser assertions:
  scratchpad/class-icons-cut43-20261007; /tmp/riddle-cut43-real-watch.log.
- Final rebuilt-WASM scoped client checks: report-class-xp, live-roster,
  class-styles and gunner-observer, all 4/4 pass at 400/1440 (15.4 s),
  /tmp/riddle-cut43-ui-final.log. Owned QA ports5395/5396 closed; shared5219 kept.
- Historical broad client result remains 91/112; this scoped change does not
  certify the full balance suite or the independent 95-point fun goal.
