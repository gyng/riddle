# Cut 31 — numbered ascensions

Implementation contract, 2026-10-06. Owner direction and complete intended scope
are ENDGAME_ASCENSIONS.md: endless harder descents, affixes/elites/boss changes,
then branching Legacy and classes. This cut implements the endgame loop first.
Partial checkpoints below do not certify the whole cut.

Status: A implemented and verified as below; B/C/D remain open. Higher tiers
are not exposed in the WASM/native app bridge or UI and currently have no
encounter modifiers. This is not a playable numbered endgame yet.

## Save, unlock and reset contract

Each bloodline owns selected difficulty `tier`, highest selectable `unlocked`,
and highest completed `cleared`. Difficulty zero is the existing dungeon.
Clearing tier N unlocks N+1 with checked arithmetic; repeating a completed tier
cannot increase unlocks again. Storage is constant size, not one record/tier.
A run snapshots its tier at send and saves it in history/replay copies.

Old saves with no numbered state remain difficulty zero, including old challenge
ascensions; never infer harder clears from their ascension counter. An old ended
save grants tier1 eligibility on the next explicit new-descent action. Loading
alone does not reset, alter live difficulty, consume RNG or spend currency.

`begin_descent(tier)` is a new path, separate from old `ascend(variant)`. Initially
available at an ending, it validates the entire request before mutation. Invalid,
premature or locked requests preserve the full save exactly. The selected
surviving hero begins the next descent; retain identity/heir/class, all currency,
Legacy, equipment upgrades, unspent supplies, vault/party, town/workers, facts,
trained tactics and chosen policy. Recompile generated tactics against the new
record immediately; authored policy is retained. Reset selected depth/checkpoints/freshness,
record-linked cache, recent departure counters and old run/replay presentation.
Other bloodlines/live runs remain unchanged. Retain global clocks and run IDs.
No automatic send or construction; scout continues existing automation.
Existing challenge API preserves its verified reset behavior while new UI is
being built. Do not expose a selectable higher tier until its mechanics exist.

## Delivery checkpoints and gates

A. Core progression, validated separate reset, old-save compatibility and run
snapshot. Tests: five ordered clears unlock1–5; replay0 does not unlock6;
three selected slots preserve full other games and shared wallet; premature,
locked and overflow refusals preserve exact saves; tier0 gameplay/RNG/hash
unchanged. Old live tier0 saves retain their replay and RNG. No client claim.
B. Three actual affixes, two elites and one boss alteration, bounded arithmetic,
modifier metadata in snapshots/tooltips/death attribution. At most two affixes
and one elite ability; deterministic choice independent of gameplay RNG.
Tests include spawn/summon/ally distinctions, counter effects, replay/save
faithfulness and tier0 parity. Specify each mechanic and its counter before
implementation; no hardcoded finite difficulty list.
C. WASM/native bridge and chunky ending/tier selection, read-only consequences,
320/400/1440 controls and actual earned screenshots; class/Legacy retained.
Five actual consecutive clears demonstrate loop and durable progression.
D. Fixed-seed tier1/3/5 tuning, meaningful build comparisons and normal required
checks. Median equal-work native tick overhead target ≤10%, report timings.
Whole/sliced/reloaded absence parity for one/three bloodlines. Preserve all
current tier0 balance gates, manual construction and no absence punishment.

A–D must all pass before calling numbered ascensions implemented. Affix balance,
higher tier completion, the Legacy tree and class specializations remain explicit
work, never implied by progression-only tests. No deployment until requested.

## Checkpoint A verification — 2026-10-06

`endgame.rs` owns constant-size Progress, validation/idempotent checked unlocks,
read-only eligibility for old endings, and a separate `begin_descent` reset.
Optional serialized state keeps old challenge counts distinct and leaves old
save fields absent. Run difficulty snapshots survive saves/history/capsules;
forecast keys distinguish positive selected tiers without changing tier0 keys.
Normal ending tick records the played run's tier. Old challenge API returns to
its unscaled tier0 and retains numbered unlock progress if present.

Eight new focused tests within the full workspace run:578 PASS/1 existing
ignored in37.01s. Covers five sequential progress clears/repeat/locked/overflow,
old count37 not implying harder clears, all three selected bloodlines carrying
wallet/Legacy/class/kit/supplies with full other games untouched, live-run saved
tier, tier0 simulation/events/RNG/key parity, synthetic ending hook reading
played tier, and full-save busy/premature/locked/invalid refusals. Invalid saved
progress rejects load. Synthetic boundary fixture does not prove a King kill.

Earned King seed3/heir33 source from UX_ASCENSION_CONSEQUENCES.md: actual new
core reset to tier1 for single and legally added two other live bloodlines;
keeps heir33, gold2850(single)/450(multi, after their real send costs), classes,
Legacy and kit; full other runtime games unchanged. Complete result reloads
exactly. First diagnostic exposed stale generated record guard D35 becoming D2
only on reload; corrected by immediate recompile and covered in regression.
Diagnostic Rust source copied to scratchpad and temporary example removed.

Rebuilt real WASM load/save for both numbered native results byte-identical.
Six previous-WASM earned ending/old variant/multi saves: rebuilt WASM produces
byte-identical entire loaded saves,1s advance event/snapshot responses and
entire advanced saves. These exercise existing gameplay, not new difficulty
mechanics. Scripts, raw integer-preserving JSON and SHA256 records retained in
scratchpad/endgame-foundation-20261006/. Native/WASM numbered equality is a
save/reset proof; no harder encounter or tier completion claim.

Final clippy all workspace/all targets with warnings denied PASS; fast WASM
5,258,334B, package real directory. Web build/typecheck PASS (existing bundle
size advisory); copy-lint1630/0 violations and diff clean. Full statistical
balance gates, higher-tier profiling/completions, modifier/replay UI and UI
screenshots remain for B/C/D. No deployment or claim of full-cut completion.
