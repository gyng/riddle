# Historical hero identity in death feedback — 2026-10-06

Capture optional structured hero identity in Death at record creation: Rust name
from lineage seed + run heir, bloodline id, heir and run hero class. Show name,
class and Bloodline above death banner. Preserve owner-requested You died seal,
causes/verdicts/actions, patch logic and existing layouts. Do not derive names
from currently selected heir or change historical identity when camp changes.
Old records without identity show existing screen, no fabricated name. Capture
also applies to stalled verdict records, without calling their hero dead.

Numeric gates: name matches original run heir, independent of later heir/class/
bloodline changes; serialized identity roundtrips; old wire missing field loads;
record creation does not alter save/RNG. UI320/400/1440 preserves historical
name/class/bloodline despite selected hero changes; old wire has no extra line;
no overflow. Fast workspace tests/typecheck/copy, rebuilt real WASM, build and
headed real recorded-death screenshots. No tuning/rules changes or deployment.

Implemented: Death.hero optional Rust DeathHero captured by trace::record;
name uses run heir, class uses run hero, slot uses lineage bloodline_id. Old
records remain unnamed; no backfill from current hero. New line above banner
also appears on stalls, preserving stall presentation and owner seal.

Validation: workspace fast551 tests pass34.64s/1 ignored; historical identity
test checks read-only record creation, captured run heir/class/slot, later heir
change, serialization and old-wire omission. Initial test called nonexistent
start; corrected to existing start_run before full run. UI27 checks across
320/400/1440 +existing death-actions44 pass5.1s batch, including selected hero
change, lineage pulse, old wire and stall. Clippy/typecheck/build/copy1505/diff
pass; existing chunk advisory. Real WASM rebuilt5,240,080 bytes.

Headed actual old seed15 camp -> new8h simulation death: Iris/fighter/Bloodline1,
heir4/current5, identity persists save-load at400/1440; no warnings/overflow.
Screens shown. This is newly recorded death coverage on an existing legal camp,
not fresh first-session progression or other-slot real death coverage. Other
selected-slot isolation is UI fixture/Rust slot-id coverage. Captures/logs/script
in scratchpad/death-hero-20261006; Rust/WASM/clippy logs in scratchpad/.
No rules, tuning, historical portraits, full balance audit or deployment.
