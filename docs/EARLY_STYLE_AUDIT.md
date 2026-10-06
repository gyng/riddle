# Earned early-style comparison — 2026-10-06

Three predetermined seeds (1,3,5), actual shipped WASM new towns: explicitly
build house, send runs by hand, hire only the earned porter/scout, then check
in every8h until Bold/BossFocus/KiteArchers are owned. All reached this at the
32h check-in after three manual sends. No forged gold, unlocks, combat stats,
extra buildings, hero upgrades, or extra hires. These are intentionally sparse
player states, not a full engaging-bot model or a balance gate.

From each exact saved Session, compare the next8h unchanged (Steady) versus
Guarded, BossFocus, KiteArchers or Bold independently. Tactics occupy slot0.
Every sourceSHA/executableSHA/outputSHA bound by choice-check manifests; all15
native reports and exact raw final saves independently match current shipped
WASM. Initial records are bestD11/D13/D18, so deepest this absence must not be
confused with the persistent record. Bests below name only new boss victories.

| Seed | Choice | Runs | Deepest this absence | Gold home | Deaths | New boss victories |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 1 | steady | 15 | D11 | $1219 | 0 | — |
| 1 | guarded | 9 | D19 | $9974 | 3 | bloat mother, lich |
| 1 | boss_focus | 14 | D13 | $2278 | 0 | — |
| 1 | kite_archers | 15 | D8 | $941 | 0 | — |
| 1 | bold | 12 | D18 | $5781 | 11 | bloat mother |
| 3 | steady | 14 | D12 | $1267 | 1 | — |
| 3 | guarded | 11 | D13 | $1709 | 3 | bloat mother |
| 3 | boss_focus | 14 | D12 | $1234 | 1 | — |
| 3 | kite_archers | 14 | D10 | $1098 | 1 | — |
| 3 | bold | 12 | D13 | $0 | 12 | bloat mother |
| 5 | steady | 11 | D13 | $2725 | 1 | — |
| 5 | guarded | 8 | D19 | $5054 | 2 | lich |
| 5 | boss_focus | 12 | D13 | $1881 | 1 | — |
| 5 | kite_archers | 11 | D13 | $2976 | 1 | — |
| 5 | bold | 13 | D13 | $0 | 13 | — |

Bold's high attrition repeats:11–13 deaths and no gold in two towns. It still
wins Mother in seeds1/3; seed1 brings$5781 home. Guarded brings gold in all
three and wins a new boss in all three; it exceeds the baseline's depth in
seeds1/5. This supports inspecting recovery/rest and supplies as actual levers,
not declaring Bold universally broken or Guarded universally superior. Stance
levels, previous scars, progression and terrain differ between these towns.
BossFocus/KiteArchers have state-dependent outcomes; a tactic need not improve
every saved town. Do not alter content or weaken gates from this sample.

Warm native five-case batches1.99/1.35/2.88s including build0.04–0.06s. These
are diagnostic fixture observations, not a general performance bound. Current
WASM SHA7554108a1f234dc3597d9473e5d5ca5b2fb552319541fb28786c065ae4fc1acd.
Source saves, chronological reports, hashes, raw outputs and verification:
scratchpad/early-style-audit-20261006/. Actual earned seed3 Guarded/Bold report
screenshots confirm the already-added boss victories section alongside gold
(the deaths remain under Details); clean page errors. No player UI/core/balance/WASM/deployment change.

Next content experiment: compare recovery/supply changes on these immutable
states and broader early walls before any balance edit. Use targeted full-seed
requirements for the affected stance; this audit does not certify fortnight
progression, all stance niches, or the exhaustive272-case audit.
