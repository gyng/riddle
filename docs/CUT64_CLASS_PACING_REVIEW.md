# Cut 64 — broader class pacing review

Extend the current progression review beyond the six Gunner campaigns. Compare
Fighter, Rogue, Ranger and Caster from the same three publicly earned D13 camps,
with paid eight-hour home preparation and no grants or authored rule edits.
Record played depth separately from inherited records, first clear, class
mastery, deaths, and Legacy balances. Preserve complete starting/ending saves
and source identity. This is a diagnostic, not a gate or a universal balance
claim. Do not change XP, prices or caps without a reproduced pacing failure.

Acceptance: each class selection/unlock is a legal player action; preparation
spends only available resources; campaigns stop at 168 hours or earned mastery
after a clear; no automatic advancement to an unowned harder descent. Existing
manual opening and Cut61 recovery behavior remain intact. No deployment.

## Results and decisions

Current runtime f5eafbe; three public earned camps originally took 16/24 hours
to reach the Gunner unlock. Fighter already starts at level 4–5; the other
three classes start at level 1, zero XP. Eight-hour samples measure time after
class selection, not time from a new town. All twelve replay D13 in the first
eight-hour sample. Caster seed3 needs another 16 hours of legal Fighter play
to meet the Lich before its paid unlock; that wait is recorded separately.

| Class | Mastery hours, seeds 1 / 3 / 5 | First clear hours, seeds 1 / 3 / 5 |
| --- | --- | --- |
| Fighter | 88 / 96 / 72 | none / none / 128 |
| Rogue | 96 / 104 / 104 | none / none / none |
| Ranger | 88 / 104 / 88 | none / 160 / 160 |
| Caster | 96 / 96 / 104 | none / none / none |

“None” means not observed within 168 hours after selection, not impossible.
These are paid-upkeep Steady campaigns with the source camp's temperament,
no selected tactics or pen edits, not the IDLE/PICKED/TUNED gate bots. There is
no universal first-clear requirement for this policy. Three seeds cannot
certify class balance. No baseline XP acceleration is justified by this sample.

Two explicitly owned Boss Focus comparisons preserve the same source and
preparation: seed1 Fighter mastery 88→112h, still no clear; seed3 Rogue mastery
104→112h, best D28→D33, still no clear. Longer/deeper runs can slow XP per hour;
Boss Focus is not a universal cure. Retain tactical trade-offs instead of
silently choosing a build for the player.

The strongest remaining progression gap is late Legacy choice depth. Final
base campaigns have 377–981 unspent points, with 162 spent on the selected
root/Recovery/Warding paths. The full current tree's legal maximum is 216,
including Control and one exclusive leaf per branch. Surplus alone does not
justify unlimited stat ranks or changing existing earnings. Prioritize a
separate post-clear contract with mutually exclusive build choices and paid,
earned higher-descent comparisons; preserve opening prices and existing saves.

Retain Cut61's D18-gated safe-recovery change, already verified against the
unchanged progression gates: seed8's five-day early wall becomes two days.
The opening still targets Scout within 15 minutes/5 manual sends; previous
eight-seed results are 9.3m median/10.9m worst, three sends. Fresh built-WASM
seed11 walkthrough confirms empty town→manual house→three manual sends→Scout
and a 20m return with real gold, XP and Legacy. It completes in 37s, zero
console/page errors; this is a scripted diagnostic, not an independent rating.

## Verification and reusable tool

`class_campaign` is an example only; no core library, shipping rates, prices,
caps, UI or scoring change in this cut. Target a seed/class/tactic independently:

```sh
cargo build -q --profile fast -p riddle-core --example class_campaign
target/fast/examples/class_campaign EARNED_CAMPS NEW_OUT_DIR --seed 3 --class rogue --tactic boss_focus
```

Three base runs and two targeted comparisons finish successfully (14 campaigns).
Fast build, scoped Clippy with warnings denied, and diff checks pass. Full
starting/ending saves, per-class results, action lists and unchanged source
bytes are retained. First attempt incorrectly assumed every D13 camp could buy
Caster and stopped after seven completed campaigns; its raw failure remains.
The final harness waits for legal unlocks and records failures without grants.
Per-class result files preserve completed evidence independently of later jobs.

Artifacts: `scratchpad/class-pacing-cut64-seed{1,3,5}/`,
`scratchpad/class-pacing-cut64-focus-{fighter,rogue}/`,
`scratchpad/qa-cut64/walk/`; logs `/tmp/riddle-cut64-*.log`.
Native executable SHA256:
`e768aec30bc43f22b551205964dd17d0ddf01f0f231f8a30423121f90b613070`.
The initial failed campaign is under `scratchpad/class-pacing-cut64/`.
Broad client audit remains a separate running job; no new full-suite,
exhaustive-balance or independent fun-score claim. No deployment.

Earned seed1 Rogue final save inspected in current built real WASM at400/1440:
L10, bestD33, 529 Legacy; selected paths owned, unused Control still affordable.
Opening the actual hero sheet preserves the exact save and has no page errors
or document overflow. Desktop screenshot shown; these UI checks do not claim
the complete tree was purchased or certify all screen geometry.
