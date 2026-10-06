# Saved-town choice feedback — 2026-10-06

Compare actual next-absence outcomes from the same saved Session, changing only
one selected hero's earned style/tactic. Baseline plus each requested choice;
same elapsed time; no extra hires, construction or upgrades are scripted.
Existing workers and earned progression operate normally. This is a local content
diagnostic, not a forecast or acceptance gate. Preserve all active heroes and
shared town. Reject unavailable choices before any simulations; preserve input
and refuse to overwrite existing output artifacts.

Usage: `node tools/choice-check.mjs SAVE --choices boss_focus,kite_archers,bold --hours 8 --out DIR`.
Keep raw reports and final saves, input/runtime hashes, build and simulation
timings. Warm feedback should finish in seconds for the first-boss fixture;
verify outcomes against the real WASM diagnostic, plus a multihero save and
invalid choices. Don't generalise one saved-town result into a balance verdict.

SAVE can be the game's raw saved JSON or a Settings → Debug export dump.
Dumps are validated before running; the embedded engine string is loaded exactly.
The manifest retains source capture/build metadata, source and engine SHA-256,
and the current native executable SHA. This runs the current code against the
captured state; it does not restore the tester's older executable. Each case changes the saved Session's
selected hero, while every active bloodline continues its normal absence.
An optional `:SLOT` is the **tactic equipment slot**, not a bloodline id; it
defaults to0. Each listed choice is an independent comparison, not a combined
loadout. Already equipped choices produce a useful unchanged control.
Hours are integer1–336, default8. Before the scout, the usual single-run limit
still applies. No extra building, worker hiring or hero upgrades are scripted.

`--out` writes a new directory of case files with raw report and final save.
The save and executable are copied to a private temporary directory for the
comparison; a concurrent build cannot replace the executable being measured.
The manifest binds input, native executable, optional balance profile and each
case file by SHA-256; timings include build separately from simulation work.
Without `--out`, print the outcome/timing table only. Existing output
directories are refused. Keep these state artifacts out of Git.

`--balance tuning/balance.json` enables the existing Rust dev-balance feature;
other profiles use the same Rust schema/bounds. Each process installs one
profile before loading the saved town. Omitting the flag clears inherited
profile JSON and rebuilds/selects the shipping-default native executable.
The first build after a source/feature change can take longer than a warm run.

Fixture acceptance: four first-boss next8h cases complete in≤5s with a warm
native build; reports and final saves exactly match WASM on all four choices.
Two-hero cases preserve both per-slot reports and exact final saved states.
Invalid/unavailable/duplicate choices, out-of-range hours, invalid profiles
and existing output paths must fail without changing the input.

First-boss evidence: seed3 legal house/three manual sends/scout/32h state;
unchanged next8h versus Boss Focus/Kite Archers/Bold. Steady14 runs,D12,$1267,
one gas death; Boss Focus14,D12,$1234,one gas death, no tactic fires; Kite
Archers14,D10,$1098,one eel death,256 tactic fires. Bold12,D13,$0,12 deaths,
first Bloat Mother kill. Baseline repeat report/save exact. These are one
seed/state and do not justify tuning content. Browser artifacts under
scratchpad/earned-tactic-comparison-20261006/.

Verified: final pinned-runtime four-case8h comparison2.20s total (build0.09s),
versus earlier warm runs1.11/1.12/1.37s. Five cases including equipped Steady
completed2.00s; Steady report/save exactly equal baseline. These are fixture
observations, not a comparative benchmark or general simulation bound.
All four final reports and raw save strings exactly match real WASM; case
hashes and executable/input binding verified.

Two-hero baseline/BossFocus each retain31runs and both slot reports; final
pinned2cases1.15s and complete report/save equality against the prior fixture.
Default balance profile matches default outputs; changed Bold healing affects
actual output; invalid bounds rejected; omitting profile clears inherited JSON
and restores default report/save exactly. First dev-balance build53.75s; warm
profile build0.04–0.06s. No tuning values changed in the repository.

Unavailable and normalized duplicate choices, hours0/337 (including direct
native337), missing arguments and existing output refusal checked; source
bytes unchanged and rejected new-output paths absent. Example clippy and JS
syntax/diff checks pass. Real WASM rebuilt as required after Rust example edit,
5,236,591bytes/SHA256ae07e9503204d65d236072dbe0183b8fd8608d0376dfd70f6411fc41528047c6
unchanged. No runtime library/UI/deployment changes or balance-gate claim.
Evidence: scratchpad/choice-check-release-20261006/ and original browser results
in scratchpad/earned-tactic-comparison-20261006/.
