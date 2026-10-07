# Earned D19 progression follow-up — 2026-10-07

The actual coordinator's saved town is
`scratchpad/qa-cut78-interactive/earned-d19-trained-engine.json`, extracted
read-only from its played `checkpoint-d19-trained.json`. Best D19, Fighter L4,
$1951, 4 unspent Legacy; health/damage/armour rank3, Clear lungs/Restoration rank1.
Guarded, boss focus, corridor fighting and Skittish are equipped. No new points,
gear, unlocks or elapsed time were granted to the input. SHA256
`5d7fa3f963de1d42192426e0db62d281ed1f4d89a1c36e0e2584bc72ba2589c6`.

Independent native eight-hour replays from that same input:

| One changed choice | Runs | Deepest | Gold home | Deaths | Legacy earned |
|---|---:|---:|---:|---:|---:|
| Current configuration | 7 | D19 | $4880 | 1 | 26 |
| Steady stance | 7 | D21 | $9061 | 1 | 28 |
| Kite archers in second tactic slot | 6 | D22 | $7897 | 1 | 28 |
| Boss focus selected again | 7 | D19 | $4880 | 1 | 26 |
| Light hands temperament | 7 | D21 | $7153 | 1 | 28 |
| Unbowed temperament | 7 | D19 | $4440 | 1 | 26 |

These are legal owned choices. An initially requested unowned Pack break was
refused before simulation; it is absent from the result. Baseline's completed
runs have no stuck return: three bank/no-heals, three hurt returns, one ogre
death on D18 after drain removed47maxHP, leaving6. Steady's replay does include one
stuck return on D19; do not claim all potential stalls are removed. Other
sampled deaths name iron golems. The evidence supports trying a different
stance or tactic at this wall; it does not establish a broad balance result or
that the player has achieved the simulated progression.

Native input and executable are pinned in
`scratchpad/qa-cut92-d19-eight-hour-choices/manifest.json`; complete report and
save per case are retained. Native wall timings include the diagnostic's final
verdict and are not compared with the browser quick path.

Shipping-WASM `runOfflineQuick(8h)` was profiled separately on the same input
in the isolated WSL headed browser, with compilation/warm-up excluded. Three
fresh repetitions took1.956/1.987/1.941s, identical complete report and save
hashes. The profile includes loading/serialization, while those wall times
cover the engine call only. QA browsers were active, so these are diagnostic
samples, not a quiet paired optimization benchmark. WASM SHA256
`55f724843bc57e68dc8970e5af72f0e3889d12fa13d16efe3e8c694f04a2d20e`;
evidence `scratchpad/qa-cut92-d19-profile/summary.json` and CPU profile.

Shipping symbols are stripped: hottest leaf function67 accounts for17.25% of
samples, but its name has not been established for this exact artifact. Do not
reuse older WASM function-number maps. Next performance work should profile a
separate symbolized current artifact, verify its outcomes against shipping,
and then compare any candidate on multiple earned saves with exact outputs.
No engine optimization was landed from these samples; no fun score or new
played progress is claimed. Independent QA continues on immutable396f6ed.
