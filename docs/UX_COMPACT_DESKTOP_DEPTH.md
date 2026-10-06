# Compact desktop depth — 2026-10-06

Replace the desktop shaft's long forecast ladder with current/start/watched
floor, lifetime record and one next meaningful forecast milestone (boss or
bonus floor; otherwise next record). Keep the complete forecast accessible.
Preserve phone layout and original forecast values/uncertainty. Watch summary
must follow the pictured floor when engine state advances ahead; it must not
switch to a newly inherited hero's floor at the end. Watch reference is read
only for tactic edits. Dispose reference subscriptions when closing.

Checks: at1024/1440/1920 compact <=320px high with <=3 summary rows, no
horizontal overflow. Current watch floor stable while paused core moves ahead.
Next milestone chosen in floor order, missing/unknown forecast handled, non-D1
starts retained. Full reference opens/closes and still contains original floor
values. Phone400 keeps existing ladder and camp forecast navigation. Existing
watch status/geometry and frame checks; build/copy/typecheck. Actual earned
WASM camp/watch desktop screenshots. No gameplay/Rust/WASM/deployment change.


Implemented a shared three-row summary inside the existing shaft plate,
selected by desktop CSS. Phone keeps the existing notches. Known boss/bonus
milestones follow floor order; no future milestone falls back to the next
record. Reuses unit icon and enemy tooltip components, numeric uncertainty and
estimate labels. Active challenge badge retained. Full forecast opens from
camp or a desktop reference window; reference tactic hints remain visible but
cannot apply patches. Window close disposes forecast listeners.

Watch passes its pictured depth to the summary and updates it with the HUD;
engine-live updates cannot replace a paused picture's floor. Summary writes
are keyed; no new animation-loop layout measurements. The previous unknown
shaft visibility rule hid even its known current depth, so desktop summaries
now remain visible with missing forecasts. Challenge test initially measured
325px from an old camp padding override; corrected spacing/specificity without
relaxing the320px gate. Preserved text sizes and chunky frame.

Final compact/status/console +all six UI sections9/9 pass40.7s, including
all132 legacy UI checks. New checks400/1024/1440/1920 cover three rows/height,
starts, records, boss/bonus floor order, uncertainty, active challenge, live
update, full-reference open/Escape, mobile notches and retained read-only hint.
Status checks assert visible missing-forecast summary, pausedD5 while engineD6,
then watched/endD6. Build/typecheck/copy1581/diff pass; existing bundle advisory.
No full client/balance suite certification.

Actual headed WASM earnedseed3 current ownedSteady,1440/1920 camp/watch/reference
and400camp captures pass: picturedD4 summaryD4, real forecast38%±34 estimate for
D8warlord, recordD13; no errors/overflow. Source forecast saved in results, no
fabricated gameplay state. Artifacts scratchpad/compact-depth-20261006/.
Screenshots shown. No Rust/WASM or gameplay changes. No deployment.
