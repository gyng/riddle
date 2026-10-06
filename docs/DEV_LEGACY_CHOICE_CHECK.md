# Direct saved-town Legacy comparisons — 2026-10-06

Extend choice-check with `--upgrades health,armour,damage,all`. Each requested
path independently finishes its selected bloodline's current ranks through
actual Rust offers/purchases. Cross each path with the original loadout and
requested package choices, including unchanged controls. Preserve all slots,
supplies and shared town; no browser preparation, forged points or gold.

Validate every preparation and package choice before any simulations/output.
Reject unknown/duplicate paths, away or unaffordable purchases. Already complete
paths are unchanged controls. Keep old invocation's version1 case shape and
filenames; expanded comparisons use version2 with path/spend/rank provenance.
Keep input/executable/profile/output hashes, source tester dump metadata and
non-overwriting outputs. Validate exact reports/raw saves for all30 cases in
the previous earned Legacy/WASM audit; test partial/capped paths, insufficient
points and selected-only multihero ownership. Measure three warm expanded
runs against the old15 separate invocations. No gate/balance changes or deploy.

Verified: all30 expanded reports and raw final saves exactly equal prior
real-WASM Legacy audit outputs across seeds1/3/5. Original invocation's
version1/no upgrade fields/0-baseline and1-bold filenames and exact results
preserved. Final10case verification repeated after error-context change.
Native example3tests cover real costs, partial/capped paths, source unchanged
on insufficient/unknown/away refusal, and selected-only purchases for both
selected1 and selected2. CLI2tests reject malformed/duplicate/missing paths
before loading/building and preserve existing output. Actual unaffordable
negative fixture refuses with named path and no output/source mutation.
Clippy/syntax/diff clean. Required WASM rebuild hash exactly unchanged:
47e7a8969d90638d091060b37902c353d086b177023b977a490d79ed449143ad.
No library/player UI changes, so no new app screenshot or balance gate claim.

Three paired warm seed3 ten-case timings (seconds; alternating order):

| Pair | Five separate invocations | One expanded invocation |
| --- | ---: | ---: |
| 1 | 5.55 | 5.36 |
| 2 | 5.74 | 5.34 |
| 3 | 5.84 | 5.40 |

Median5.74→5.36s (~6.6% lower); a modest native wall-time improvement, not a
simulation speedup or full-gate claim. Main workflow gain: five command/build
setups become one and no browser is needed to prepare four legal paths.
First modified-example build took15.33s/20.12s total; warm runs do not include
that rebuild. All cases still run, with no result-cache reuse or smaller seed
counts. Evidence: scratchpad/native-legacy-choice-20261006/ includes30 verified
cases, immutable manifests, prior-output comparison and paired timing.json.
Diagnostic only; no shipping balance/gate changes or deployment.
