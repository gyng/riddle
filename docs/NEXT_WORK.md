# Owner queue — 2026-10-06

Local UI iteration-speed and compact-depth work are implemented and pushed.
Continue with hero presence/tactic observability, then the content/checkpoint audit. Push approved
changes; do not deploy until the owner requests it. Show real app screenshots at
UI checkpoints. Keep simulation truth in Rust and manual town construction.

1. **Compact desktop depth column — implemented2026-10-06.** Replace the long right-side floor ladder
   with a concise view of the watched/current floor, next important boss or
   checkpoint and nearby progress. Keep the full depth/forecast reference
   accessible when wanted. Inspect actual early/established/late states before
   choosing the layout; preserve forecast uncertainty and mobile access.
   Verify desktop spacing, watch/camp identity, tooltips, overflow and snapshots.
2. **Rich hero presence.** Active hero entries should communicate an immediate
   status such as D13 · In combat, exploring, collecting loot, heading home or
   ready in town, using the existing portrait/thumbnail and a concise indicator.
   Audit available engine snapshots/events first; avoid inventing live facts
   from stale data or polling expensively. Watched replay and current hero status
   must remain distinguishable, including paused/earlier footage and inheritance.
   Verify multiple bloodlines, away catch-up, exit/death, town and phone views.
3. **Remaining content and checkpoint audit.** Inventory what is actually
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

This records requested work; none of these three items is certified complete.

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
