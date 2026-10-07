# Cut 61 — progression and pacing review

Review the earned first session, first-week walls, class training, Legacy and
harder descents. Preserve manual construction, optional tactics and all numeric
gates. No grants, automatic purchases or additional reset layer. No deployment.

## Findings

| Area | Evidence | Decision |
| --- | --- | --- |
| Opening | Eight-seed baseline: Scout after 3 sends, median 9.3 minutes, worst 10.9; Porter median 9.3, worst 10.6. Depth 8 and stance/content targets pass. Separate baseline depth-8 check passes all 16 seeds. | Preserve opening cadence. |
| First-week wall | Baseline seed 8 holds depth 20 from day 2 through day 7; reaches depth 28 on day 8. Five stagnant days fail the unchanged four-day limit. Gold, class training and drills still progress. | Improve safe recovery at deeper floors. |
| Gunner | Six fresh continuations from public earned 16/24-hour unlock camps: both weapons reach depth 13 within 8 hours after switching. First clears take 16–56 hours; mastery takes 96–136 hours. | No blanket class XP acceleration. Samples do not certify every class. |
| Endgame | Earned campaigns clear the first descent; the existing app offers the next numbered, harder descent for the same bloodline. | Preserve explicit progression choice. Higher-tier variety remains a content priority. |
| Legacy | Twelve-node tree has at most 216 purchased points. Mature saves can accumulate surplus, including repeated first-depth earnings for new heirs. | Queue deeper meaningful choices/sinks; preserve existing earnings in this fix. |

## Change

Steady level 3+ recovers to 60% health in safe gaps once historical best reaches
D18. Before D18 its threshold remains 50%; level 2 remains 40%. Attack precedes
rest. Heal, hurt/dry exits, telegraph responses and rest safety are unchanged:
no recovery with visible enemies, poison or a hazardous tile.

Rust owns the change in `packages::stance_rows` and `balance::DEFAULT`.
`tuning/balance.json` matches the shipping default. Class XP, completed-run
training thresholds, Legacy earnings, spending and elapsed time are unchanged.

## Trials and rejection criteria

All existing thresholds remain unchanged. A candidate must preserve depth 8 on
day one, first-week training, pre-D23 stall ≤4 days, no idle King clear,
chosen-tactics advantage ≥1.5× and ≥90% outpace pairs, growing check-ins,
content-stage cadence and send-stall limits.

- Global 60% recovery reduces seed 8's stall from five days to two, but delays
  median stance level 5 to 7.5–8 days, past the seven-day limit. Rejected.
- Global 55% passes the targeted stall/training trial, but the wider check
  puts seed 11's first D8 at 32 hours. Rejected; baseline D8 passes all 16.
- D18-gated 60% passes all 13 targeted rows across 16-seed configurations in
  202 seconds. Universal idle and send-stall rows complete 16/16. Fail-fast
  medians/ratios can settle before all remaining results; outpace settles at
  94–100% with one seed left. This is a targeted check, not an exhaustive audit.

Accepted seed 8 trace: D16 day 1 → D21 day 2 → D28 day 5; longest pre-D23
stall five → two days. Its individual stance level 5 still arrives on day 8;
the unchanged gate specifies the 16-seed median, which passes. The detailed
trace stops at the selected milestone on day 8, not a full fortnight.

An earlier direct Rest55 trace accidentally used `RIDDLE_BALANCE_FILE`, which
only the native-dev wrapper reads, and therefore replayed the baseline.
Correct direct trials install `RIDDLE_BALANCE_JSON` with `dev-balance`.

## Final validation

- `tools/verify.sh --full`: green in 435 seconds. 678 Rust tests pass, one
  existing ignored test; TypeScript, 1,736 copy literals, 14 tooling tests,
  clippy, shipping WASM, web build, current metrics, ten wire seeds and all
  18 current-player fortnight cases pass. Broad 272-case migration/system
  removal audit was not run.
- First sandboxed verification stops at the native-host child-process test.
  Isolated unrestricted rerun passes 2/2; full unrestricted rerun is green.
  No test assertion changed. Existing web chunk-size warning remains.
- Native/WASM exact parity: 27 bridge replies across three earned camps,
  including forecast, options, eight-hour catch-up and resulting raw saves.
- Fresh local origin :5426: earned early L2 and trained late L5 saves show
  correct 40%/60% rows; mobile 400 and desktop 1440 checks preserve raw save,
  with no overflow or page errors. Screenshots shown. Late earned cleared
  save opens the ascension screen, not home; its wrapper's app-only run counter
  was initialized to zero, so that display is not a campaign run-count claim.
  Initial UI assertion mistakenly expected L3 behavior from an L2 camp;
  corrected fixture/threshold check passes. Temporary preview stopped; :5219
  retained.

Raw logs: `/tmp/riddle-pacing-*.log`; preserved artifacts under
`scratchpad/pacing-cut61/`, including rejected profiles, baseline binary,
fresh Gunner continuations, parity hashes and UI captures. Scratch artifacts
are not staged. No gameplay/UI gates weakened; no deployment.

## Next content priorities

1. Meaningful deeper Legacy choices or sinks for mature bloodlines.
2. Higher-descents encounter/loot variety and earned cold-start pacing samples
   for the other classes. No all-class or unlimited-tier balance claim yet.
