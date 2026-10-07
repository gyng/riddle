# Cut47 — use the compiled rules returned by class changes

Real earned Gunner class-picker → Fighter reproduction: Rust's class reply has
no gunner_tactic; after the client refresh both UI and Rust contain it tagged
player. App.setClass used mutate without adoption, then afterLineage rulesChanged
wrote the previous class's compiled set back into Rust. Equal final UI/core
alone was therefore insufficient proof; capture the class reply before refresh.

Adopt returned sets before afterLineage, using the existing mutate adoption path.
No Rust mechanics, class prices, XP, package/drill rules or save migration changes.

Numeric acceptance:400/1440 actual owned Fighter/Gunner picker round trip, exactly
one engine class call per choice, full app compiled rows equal each returned
Rust class reply before downstream edits; Fighter contains zero generated gun
handling rows, Gunner exactly one class-owned gun handling row. One player rule
and all drills persist. Invalid class refusal leaves complete save and UI rules
unchanged. Flush/reload preserves exact rules/class. Existing class-style, roster,
class XP and report replacement gates, TS/copy/build/diff. Show screenshots.

Related visible navigation defect: the initial Heroes return button is deleted
by paintTabs. Recreate it during tab paints while an active hero is editing,
including before extra set tabs are unlocked. Closing Rules refreshes compact
rows and restores roster; opening/closing must leave the engine save unchanged.
Verify both widths through actual Edit / Heroes controls and screenshots.

Related budget defect: TS isPkgRow excluded class origins while Rust PKG_ORIGINS
includes class. Gunner handling incorrectly consumed one player slot and could
block a valid send. Include class in the existing package-row predicate; assert
exactly one own row across the earned round trip. No Rust cap change.

Accepted 2026-10-07. Real earned Fighter/Gunner round trip passes at400/1440:
returned Rust rows, first UI paint, rendered Gun handling, one own rule,
preserved player/drills, one call per choice, refusal and exact reload.
Actual Edit → Heroes inspection leaves the full save unchanged. Before-extra-tabs
return coverage is a controlled presentation fixture, not earned early pen.

Seven related gates passed before the final budget correction (92.9s). Final
class-switch, patch-overflow, locked-cond, package-lanes and live-roster pass5/5
(27.9s); return gating plus first-plot pass2/2 (24.1s). Final headed screenshot
round trip passes both widths, /tmp/riddle-cut47-final-shots.log. TS/build,
copy1734 literals/zero violations and diff checks pass. No Rust/WASM changes.
Artifacts: scratchpad/class-switch-cut47-20261007. Baseline captures demonstrate
stale gun handling rewritten as player despite final UI/core equality. Retained
initial harness failure required closing the parent Hero sheet; subsequent
screenshot failure exposed the missing Heroes route, fixed in production.

Independent95 score still required; scoped gates do not certify the full suite.
