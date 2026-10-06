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
