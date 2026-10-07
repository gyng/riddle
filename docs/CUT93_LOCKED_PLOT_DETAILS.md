# Cut93 — explain a locked town plot on tap

Two independent QA players on396f6ed reproduce “next plot” taps after
house/forge/storehouse that leave the screen unchanged. The handler currently
flashes text already on the map. Hover/long-press carries details, but ordinary
taps provide no persistent explanation.

Open the existing chunky game window with the next building's name and actual
wire unlock condition. Apply the same action to the plot target and its map
label. Ready plots continue requiring the manual Build action; clicking a locked
plot must never build, spend or mutate game state. Preserve the map label and
existing build stages/target counts.

Gates: existing Cut30town numeric/placement/build checks unchanged; stage2
locked target opens a visible dialog with its building/condition and Escape
closes it, exact saved engine state unchanged. First plot/manual persistence/
construction dedup tests pass. Mobile and desktop screenshot spacing checked.
QA remains on immutable396f6ed; this repair is reviewed on a separate build.
