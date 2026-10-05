# Readable enemy warnings — 2026-10-06

Exact real-WASM camp-1 Captain first rally at tick650: living Captain on D5,
visible at(30,6), hero(23,7); own sprite and captain name work. Sprite/horn and
warning glyph sit at far viewport edge. Do not replace working art or zoom all
fights out. Show active core warning with foe name (`captain · rallies`) in its
existing plain nameplate, preserving hitpoint bar and existing crowd budget.
Prioritize active warning sources over ordinary nearest foes within that budget.
Never reveal unseen, remembered or dead enemies; boss/ally suppression and
quiet-view limits remain. Snapshot/replay load and live telegraph event both
supply text; attack/movement/glyph expiry remove the displayed warning with the
existing warning glyph. No new sim rules, controls, persistent facts or boxes.

Acceptance: exact recorded Captain snapshot at400/1000 gives bounded decoded
sprite plus captain/rallies nameplate; synthetic crowded visible warning fixture
wins same nameplate budget, regular names return after warning expires/attack;
unseen/remembered/dead warning sources never get labels, boss suppression/quiet
limits unchanged. Scoped renderer tests, build/copy/diff and headed screenshots.
No deployment/full progression gate for this rendering-only change.

## Results

Real WASM capture reads legal camp-1 D5 run in32-tick batches; at first Captain
telegraph, restores only that diagnostic run's preceding save and steps1 tick
at a time to exact tick650. Captures living14/14 Captain(30,6), hero(23,7),
visible rally text. Initial send snapshot omitted unseen Captain; corrected
collector respects visibility-filtered wire and discovers him later. Existing
art/name are good; camera legitimately retains hero while source sits at edge.
No game art/zoom/clock/balance changes were warranted.

Exact wire fixture committed at web/tests/fixtures/captain-rally.json (~21KiB),
with source metadata; includes actual returned snapshot/event, no invented
entities. Headed before/after1000 and after400 readonly renderer screenshots
inspected/shown; no console warnings. These are recorded graphics frames, not
new full-app live boss/King/progression coverage. The partial sprite at edge
remains; now the visible nameplate gives the actual warning explicitly.

30 scoped renderer checks at400/1000 pass5.9s: recorded warning/colour/bounds,
crowd/quiet budget priority, snapshot and live-event text, attack/move/expiry
clearing, remembered/hidden/dead exclusion, boss/ally behavior. Hidden fixture
uses actual2-tile sight radius (all-floor100-tile test vision would reveal it).
First readiness assertion corrected by waiting for atlas loading; diagnostic
fixtures corrected, no gameplay gates weakened. Existing fights46 checks
pass154.0s; TypeScript/production build, copy1500 and diff pass. Existing build
chunk advisory remains. No Rust changes/full balance audit/deployment.

Next: move from repeated Captain captures to a fresh first-session/away-return
content and graphics walk. Use exact recorded frames for any later enemy art
claims. Larger nameplates still obey existing overlap/stack culling, so this
change does not promise every simultaneous warning is labelled in a dense swarm.
