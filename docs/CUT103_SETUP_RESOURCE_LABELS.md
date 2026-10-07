# Cut103 — distinguish setups, upgrades and the forge

QA-b misread a saved fighter setup as the current Rogue/Ranger class, the
blacksmith gear shortcut as a worker hire, and Legacy versus upgrade tokens as
conflicting rewards before resolving them. Label the existing tab group Saved
setups without renaming any set. Label the building forge consistently with
its gear menu. Report Legacy names Hero upgrades; token plaque names Classes
and styles. Keep all amounts, ownership, choices and gameplay unchanged.

Validate types/copy/build and existing earned Legacy/report XP/town checks;
inspect rendered labels/spacing on phone and desktop. No new unit tests for
these reversible copy edits. Resource purpose fragments must fit existing
copy budgets; no tutorial paragraphs or additional controls.

Validation: existing earned Legacy/report XP/town-save/UI-frame suites pass
4/4 in26.8s; types/copy1765/diff/build pass. Read-only actual earned real-WASM
inspection at400/1440 keeps saved names [fighter,null,null] while actual class
is Gunner; building accessible label forge. Mobile scroll to actual tabs shows
Saved setups visibly, with400px scrollWidth; desktop keeps its still-closed
editor hidden rather than manufacturing an unlock for a screenshot. Actual20m
return in each context yields1run/+28tokens/+1Legacy with the distinct purpose
labels. Visual review found token/full-haul labels falling back to stair icons;
map them to existing mark/gold assets. Corrected screenshots inspected; final
build passes. Evidence scratchpad/qa-cut103-copy and
/tmp/riddle-cut103-inspect.log. These diagnostic returns are not scored play
or new broad acceptance. No Rust/gameplay/amounts/persisted names changed.
