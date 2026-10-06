# Owner queue — 2026-10-06

Latest owner priority: escalating endgame after dungeon clear, deeper bloodline
Legacy and class progression. ENDGAME_ASCENSIONS.md records the proposed loop
and delivery order: tier/affix/elite/boss architecture, compact Legacy tree,
then two class specializations. CUT31_ASCENSIONS.md now specifies the runtime
contract; A progression/reset, B encounters and C tier selection/elite marking
implemented. Five actual consecutive clears on one earned build; real UI and
native/WASM/durable-save parity verified at320/400/1440 including two other
live bloodlines. D build comparisons, one/three-slot absence parity, catch-up
profiles and routine18-case gates now verified with the documented unchanged-
preparation deviation; broad exhaustive balance remains uncertified. CUT32_LEGACY_TREE.md is now implemented/verified:
twelve nodes, actual effects, historic depth access, free home respec, exact
one/three-slot8h parity and real earned desktop/phone screenshots. Nine paid
Tier5 diagnostic builds clear in8–16h, final current18-case gate passes, scoped
equal-work overhead stays below10%. Old save/ranks retained, no deployment.
CUT33_CLASS_STYLES.md implemented/verified2026-10-07: Sentinel Riposte and
Hexbinder Hex, Rust-owned offers and real-worker choice UI, transparent icons,
presence/Hex tips,320/400/1440 earned screenshots and exact8h one/three-slot
native/WASM whole/sliced/reloaded reports/saves. Actual earned replay JSON
matches, survives later camp removal. Twelve paired Tier5 cases clear8–24h;
Sentinel seed5 slower than base, all selected cases within48h. Equal-work
microbenchmark3.28–6.82% overhead against a39e4d7, current18case gates pass.
631core/14tool tests, build/TS/copy/clippy/codegen green. No deployment.
CUT34_LEECHING_ELITES.md now implemented/verified: Tier6+ Leeching recovers
up to2HP on actual melee damage, poison/range counter, played metadata,
replay HP/log/preview and320/400/1440 real-worker checks. Six fixed earned-build
Tier6 diagnostics clear16–24h, eligibility synthetic. Native/WASM8h exact
one/three-slot whole/sliced/reloaded reports/saves, original tier0 baselines and
all twelve older Tier5 campaign saves remain exact.636core/14tool tests,
clippy/build/codegen/TS/copy and routine18case gates pass. Event208→80bytes,
positive-hit microbenchmark overhead1.12% after fixing initial23% failure.
No deployment; optional exhaustive/statistical audit remains uncertified.
Owner requests Gunner2026-10-07: CUT35_GUNNER.md is next before proposed
Rogue/Ranger styles. Fifth base class with actual long/short gun equipment,
chambers and timed reload commitment, inherited XP/Legacy, named automatic
class tactics from first send, meaningful distinct range/armour/close-spread
matchups. Prepared transparent gun icons via built-in imagegen and existing
packer. Per-item saved chambers, absolute20/15tick reload commitment and
long/short weapon definitions now implemented;643core/14tool quick checks
and all-target fast clippy pass. Actual fire/reload/close-burst and quiet-batch deadline integration now
implemented;17focused tests/653core quick PASS. Played GunSnap, native/WASM
whole/sliced/reloaded61tick controlled combat fixtures exact. Class/unlock,
forge/default tactics, remaining ladder, portraits/watch/replay UI, earned
progression and full perf/gates remain pending. No playable Gunner yet.
Preserve old drop RNG, authored rules and saved state; do not expose inactive
classes. Subsequent class-training audit and further boss/loot variety remain.

D checkpoint2026-10-06: exact earned solo Tier1/3/5 and three-bloodline8h
whole/sliced/reloaded full-state/report parity now passes after guide-quote,
partial-rest worker and fractional-clock continuation fixes. Fast-WASM8h
medians0.81/0.69/1.00s, three identical outcomes each. Unchanged Tier5 diagnostic
fails all three seeds14days; spending54 existing earned Legacy changes all nine
tier/seed cases to8–16h clears. Failed bar retained, explicit justified deviation
in CUT31; no claim of broader balance. Final quick603core/14side tests green,
clippy/build pass. Routine18-case full gate terminalPASS; four fixed-combat
comparisons pass. See CUT31 for the King's extended-window missed-hit amendment.
Cut32 branching Legacy is complete within its scoped verification; next class
specializations, with distinct actions and counters alongside the inherited tree.

Local UI iteration-speed and compact-depth work are implemented and pushed.
Hero presence, tactic observability and the content/checkpoint audit are implemented.
Catch-up slice semantics are fixed and verified within PERF_OFFLINE_SLICE_BOUNDARIES.md.
Readable progress milestone/reward is implemented in UX_PROGRESS_GOAL.md.
Direct useful preparation is implemented in UX_WALL_PREPARATION.md.
All43 item icons are covered in UX_ITEM_ICON_COVERAGE.md; the ending audit's
unsafe refusal fallback is fixed in UX_ASCENSION_REFUSAL.md.
Next: earned King/ending presentation and explanations for all four ascension
variants, verifying actual carry/reset semantics including multiple bloodlines.
Carry/reset audit now verified in UX_ASCENSION_CONSEQUENCES.md: 12 variant/slot
transitions, six exact-save refusals, existing four variant behavior checks.
Selected slot resets; shared spendable gold clears with its ledger adjustment;
Legacy, town/workers/Savings and other heroes persist. Synthetic API fixtures
do not prove an earned King victory. Ending UI now implements framed explained
choices and centered pre-reset review;39checks across phone/desktop/landscape,
frame63/QA41 pass. Diagnostic screenshots preserve real-WASM save exactly.
Earned ending now verified: normal TUNED seed3 first King on day4, exact
check-in export; all four actual WASM ascensions match native and durable full
saves. Hunted with two other live bloodlines preserves both games. Headed
400/1440 earned ending/review/camp screenshots shown, no display overrides.
See UX_ASCENSION_CONSEQUENCES.md; long-term variant completion and full balance
suite remain uncertified. Next existing-content variety and optional-reset proposal.
Post-ascension audit/fix complete in UX_PERSISTENT_CAMP.md: workers were
accessible; their shortcut mislabeled an unrelated curriculum fallback. Now
Workers/count; class offers moved out of topbar, Details choices retained.
Actual earned400/1440 fullsave read-only proof. Existing Cut305 day0 surface
assertion fails identically on previous UI source; next audit first-home fixture/
above-fold visibility without weakening manual-build or density invariants.
Then the audit's impact-ordered variety and optional-reset gameplay queue. Push approved
changes; do not deploy until the owner requests it. Show real app screenshots at
UI checkpoints. Keep simulation truth in Rust and manual town construction.

1. **Compact desktop depth column — implemented2026-10-06.** Replace the long right-side floor ladder
   with a concise view of the watched/current floor, next important boss or
   checkpoint and nearby progress. Keep the full depth/forecast reference
   accessible when wanted. Inspect actual early/established/late states before
   choosing the layout; preserve forecast uncertainty and mobile access.
   Verify desktop spacing, watch/camp identity, tooltips, overflow and snapshots.
2. **Rich hero presence — implemented2026-10-06; tactic observability implemented2026-10-06.** Active hero entries should communicate an immediate
   status such as D13 · In combat, exploring, collecting loot, heading home or
   ready in town, using the existing portrait/thumbnail and a concise indicator.
   Audit available engine snapshots/events first; avoid inventing live facts
   from stale data or polling expensively. Watched replay and current hero status
   must remain distinguishable, including paused/earlier footage and inheritance.
   Verify multiple bloodlines, away catch-up, exit/death, town and phone views.
3. **Remaining content and checkpoint audit — completed2026-10-06.** Inventory what is actually
   implemented versus PLAN/CUT30_5 and subsequent owner amendments: areas,
   enemies/bosses, classes, traits/tactics, items/forge, buildings/workers,
   bloodlines/Legacy, checkpoints, milestones and endgame. Trace real earned
   player states and progression gaps. Compare checkpoint pacing, rewards,
   progression visibility and meaningful choices with a documented selection
   of leading idle games. Research current primary game/documentation sources
   before comparisons; justify the comparison set rather than claiming an
   unsupported ranking. Distinguish missing, unfinished, implemented-but-hard-
   to-read, and verified content. Produce an impact-ordered implementation queue
   with concrete evidence and scoped acceptance checks. Preserve Riddle's
   autonomous roguelike identity and absence invariants.

Compact depth and presence are implemented and verified within their documented scope.
The audit is recorded in CONTENT_CHECKPOINT_AUDIT_20261006.md; its implementation queue remains open.

Initial code pointers: desktop right column is wideCols in ui/frame.ts, using
renderShaft in ui/forecast.ts (currently up to9 notches and a folded range).
Its watch-column opener currently does nothing, so full-reference access needs
an actual navigation path when compacting it. Hero rows are ui/heroes.ts.
HeroSlot.live currently carries only run_id/heir/depth/start/hp/max_hp/turn:
it does not expose combat/exploration phase. Rich presence for every bloodline
needs authoritative snapshot data or a small Rust-owned phase field; HP changes
or the selected hero's replay must not stand in for other heroes' activities.

Owner follow-up: **tactic observability without text flooding**, integrated into
hero-presence work. Keep a small set of existing tactic/style icons alongside
presence; light the owning tactic on actual rule activation, using throttled
or merged feedback rather than blinking on every default swing. Tie target/item
emphasis to actual action events. Hover/tap details can show the condition and
recent activation count; the existing plain combat history holds the trace.
Audit row-to-bundle ownership/available event metadata before implementation;
do not invent attribution or alter combat results. Avoid another caption feed,
per-frame layout reads or a new always-visible rule table. Verify early-player
simplicity, mobile access, repeated activations, default attacks and real tactics.


Compact depth verification: UX_COMPACT_DESKTOP_DEPTH.md, all132 UI checks plus
compact/status/console9jobs pass40.7s; actual earned WASM1440/1920 camp/watch/
reference and400camp screenshots shown. Full forecast navigation and read-only
watch hints now work. Phone ladder retained; paused picture depth stays stable.
Next presence audit should reproduce the current real screenshot: watchedD4
but hero-roster label still LiveD1. Treat that as stale presentation evidence,
not a simulation-progress conclusion; inspect slot/snapshot update ownership.

Presence checkpoint: UX_HERO_PRESENCE.md; core owns combat/exploring/returning
for every live slot. Selected roster receives engine snapshots without added
RPCs; pause does not freeze live presence. Exact4case8h raw save/report parity;
actual combat/multi/phone captures;132 legacy UI checks plus focused tests.
Tactic audit: PackagesWire.rows RowSource.label names each compiled rule's
origin, with shadowed_by; rule events carry row and verb. packages::compile
assigns origins before deduplication/truncation. Existing desktop lightRow
looks for rule tablets, which are usually folded under the hero roster.
Next: light the owning existing package icon at the watched event clock;
merge repeats, keep default swings quiet, put condition/action/count on
hover/tap. Snapshot current-run rule/source metadata rather than reattributing
old replay events to newly selected heroes/equipment. No extra caption feed.

Tactic observability implemented: UX_TACTIC_OBSERVABILITY.md. Snapshot exact
row origins/catalog at mount, small fixed-order icons at picture clock, merged
850ms cues with1200ms cooldown; routine swings quiet, preserve last meaningful
choice. Hover counts/condition, tap read-only existing parchment. Known-row
floating captions replaced; literal/unknown paths retained, history intact.
Actual matched attack destination uses existing contact ring (Canvas2D ellipse),
never non-attack/enemy/ended/dying/remembered guesses.13jobs/all132UI checks
pass; final28observer checks each320/400/1440 +end/spacing pass14.1s. Actual
owned Guarded early400/1440 and Hunter3bloodlineD8/ranged surviving archer,
pause holds counts; screenshots shown. No gameplay/WASM/deploy changes.

Content/checkpoint audit checkpoint: CONTENT_CHECKPOINT_AUDIT_20261006.md.
Inventory7biomes/37unit defs/6band bosses/43item defs/4classes/14behaviors/
11workers/3bloodlines/Legacy/D34 +4ascensions. Months reset layers reserved,
not built. Primary comparisons Melvor/Idleon/Stone Story fetched2026-10-06.
Fresh no-choice3seeds longcalls D21 by2weeks; standard targeted idle-d23
3seeds1/2/3 PASS10s median6.33d, not full gate. Exact same8h earnedseed3 save
+160h: wholecallD21/heir20 vs client30min slicesD28/heir47. Wall168.59h both,
simulated turns169.23h vs305.52h. Root: offline finishes past budget on EVERY
internal slice, rest clamp/reveal budget also restart each call; last flag
currently only skips plateau verdict. Fix this before further content: one
past-budget finish/reveal budget per actual absence, preserve intermediate
state/rest/day/worker/quest semantics, clock and report merge,1/3bloodlines,
real-return convenience, manual construction and unchanged balance bars.
Then one useful next milestone/reward, direct existing preparation, remaining
gold/bones/trap glyph icons and ending UI, existing biome/class variety, single
scoped opt-in early reset proposal (no simultaneous expedition/era/dynasty).
Actual400/1440 empty/day1/longcallweek2/client-sizedweek1 captures/artifacts
scratchpad/content-audit-20261006/. No gameplay/deploy changes in audit.
