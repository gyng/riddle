# Ascension consequences — 2026-10-06

Before replacing the ending chips, verify the current core contract rather than
reuse the pre-bloodline description. No balance or reset behavior change in
this checkpoint. Synthetic completed states prove API semantics, not an earned
King victory or successful long-term variant progression.

Acceptance: all four variants × all three selected slots (12 transitions);
complete-save equality on premature and unknown-variant refusal; unchanged
unselected games immediately after transition; switching slots propagates the
zero shared wallet without resetting other heroes. Check persistent Legacy,
class progress, facts, trained packages, town/workers/Savings and forge salvage.
Existing variant tests must still prove rest, row limits, vault and hunter rules.

## Source audit

`Session` dereferences to its selected `Game`; the bridge's ascend call therefore
restarts only that Game. `town_from` propagates shared gold and town state on
selection and background advancement. Other saved wallet copies may be stale
until synchronization; they are never authoritative spendable gold.

| State | Current transition |
| --- | --- |
| Selected bloodline | Same slot and seed; ascension +1; heir 1, record/start reset |
| Legacy | Points, purchases and persistent slot preserved |
| Class progress, facts, package training | Preserved |
| Town, hired workers, Savings | Preserved; gold ledger subtracts cleared gold |
| Spendable town gold | Zero for the whole town; not just this hero |
| Forge | Salvage progress preserved; purchased kit ranks cleared |
| Descent | Marks, reputation/rank, checkpoints, supplies, graves cleared |
| Other bloodlines | Hero, run, progress, Legacy and equipment retained |
| Companions | Party returns to kennel |
| Vault | Preserved except Bones only |
| Unlocks | Class doors/mastery retained; Short list also retains tactic cards |

Variant explanations must match the existing rules:

- No rest: no in-dungeon rest; camp recovery time halved.
- Short list: at most six rule rows, not six immediately; tactic cards retained.
- Bones only: vault emptied and unavailable; no claim that ordinary loot stops.
- Hunted: deepest previous grudge stalks from floor 3 until avenged/tamed.
  No previous grudge means no hunter; do not promise one unconditionally.

Next UI checkpoint: earned ending evidence, framed readable choices, explicit
selected-bloodline and shared-gold consequences before the irreversible action.
Use the existing chunky parchment component; cancellation must preserve the
complete save and refusal must retain the current safe retry behavior.

UI acceptance: four explained choices, selection opens a centered review without
calling the engine; close/Escape/cancel preserve the save. Explicit Start again
calls once; refusal restores controls, success reaches camp. Verify all variants
and320/400/1440 widths, short landscape, keyboard focus, shared chunky frames,
word budget and screenshots. If a legally earned King state is unavailable,
label layout captures diagnostic and leave earned-ending proof outstanding.

Verification: two new Session tests cover 12 successful variant/selected-slot
combinations and six full-save refusals. Persistent fields use nonempty Legacy,
class XP, facts, package training, forge salvage, vault, kit and Savings fixtures.
Other games compare in full before synchronization and in full after replacing
only shared town fields. All eight focused native tests pass (`ascension`,
`no_rest_removes`, `short_list_caps`, `bones_only_has`, `hunted_puts`, fast profile).
First attempts caught a mistaken ledger expectation and an omitted zero clock
fixture; both corrected without changing runtime behavior. Diff check clean.
Tests/documentation only: no runtime/WASM/UI changes, full gate certification,
earned King claim or deployment. Shared dev server5219 remains HTTP200.

UI implemented: framed ending panel and Mirror King portrait, four explained
challenge choices (Six rules/No vault replace the opaque Short list/Bones only
display names; wire IDs unchanged). Selecting opens the existing centered
parchment window, not the engine. Review names selected bloodline, actual shared
gold→0, carried Legacy/class XP/training/town/Savings/forge progress and reset
heir/depth/supplies/gear upgrades. Multiple heroes get a continuation note.
Cancel receives focus, closing returns it to the choice. Explicit Start again
retains single-flight/refusal protection. Shared centered-window CSS now also
centers windows outside the camp frame. No balance/Rust/WASM changes.

Final39review/cancel/budget/bounds/center/refusal/single-flight/success checks
each320×568/400×900/1440×900/800×400 PASS; shared frame63+QA41 PASS3jobs24.9s.
Build/typecheck/copy1630/diff clean; existing bundle-size advisory. Headed GPU
400/1440 ending/review captures viewed/shown, no errors/overflow, full real-WASM
save exact after review and cancellation. Layout fixture explicitly labelled
simulated ending; source earned D21 town remains unchanged. Run count223 comes
from source's actual exit counters; D34/ended display is a diagnostic override.
First visual review corrected missing portrait key, background contrast and
bottom-aligned review; final art/frame/centering reuse shared components.
Artifacts scratchpad/ending-review-20261006/. Earned King playthrough and actual
successful WASM transition remain outstanding; core variant behavior and
multihero reset semantics are verified separately above. Not deployed.

## Earned ending verification

Supersedes the outstanding earned-ending note above. Normal TUNED seed3 bot
slays Mirror King on day4. Bounded reproduction:
`DP_CHECKPOINT_STOP_AFTER=12 target/fast/examples/dayplayer --seeds 3 --only 3
--bots tuned --days 5 --checkins 3 --resume`. Diagnostic exit75 intentionally
stops at the twelfth complete check-in; not a completed five-day gate. No
balance/seed/state overrides. Snapshot heir33/D34,173 runs,32 deaths,$2850,
204 facts; `ended=true` and `kills` includes Mirror King. Original exact
checkpoint preserved; decode float-bit envelope and retain integer values when
exporting its serialized Game, with no gameplay-field changes. Provenance:
binary SHA256 `bdd7912dda44ce13a528fc4c10b091887c6fd17d093ea9e26a91ac69af5d54f0`,
export SHA256 `a9a3020c3eea7949e19f885e2e61f7f3be91341bd0259e0a3a8a6d7be78da469`.

All four actual browser-WASM choices successfully ascend that earned save;
each complete resulting save is byte-identical to native JSON-bridge execution
and the persisted `riddle.save.engine`. Review/Escape preserve the complete
save. Endings/reviews/new camps captured headed400/1440, no errors/overflow,
viewed/shown without diagnostic display overrides. Hunted also verified after
legally purchasing two extra bloodlines and sending both: other saved games
and live runs preserved, full native/WASM and durable-save equality. Artifacts,
source checkpoint, exported save, full resulting saves and scripts recorded in
scratchpad/earned-ending-20261006/. No runtime/UI/art/balance changes this
checkpoint; no deployment or full-suite claim.

Separate14-day fixed-choice diagnostic from the earned D21 town reached at
mostD23 across six legal tactics/Legacy cases (90.7s total); it did not earn the
ending. Initial60-day tuned search stopped once captured; rerun above pins
the first victory rather than using the later post-victory town. A long-term
variant completion or general pacing verdict is not proved by this checkpoint.
