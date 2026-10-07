# Cut104 — consistent damage and fewer duplicate decisions

QA-b could not distinguish the recipient in goblin · hero −10hp and found
corridor fighting rule repetitions crowding real combat events. Hurt uses the
same cause/source → recipient direction as attack. A Hero cause uses the
recorded batch's Hero name; other causes retain their actual kind/mechanic,
without guessing an actor ID. Capture the hero identity even when a batch has
only hurt events.

Owned style/tactic/drill rule rows already have live observed icons, counts and
detail sheets. Omit their redundant rule text from the overlay only. Keep all
actual hits, modified/unpaired damage, hazards, healing, pickups, deaths and
floor/exit events. Written/unowned rules remain textual. Disabled observer
(e.g. legacy bail prepend) must stop suppressing rows. Core events, traces,
replays and icon activation counts remain unchanged.

Gates: actual released damage arrows/name/style/cause identities, existing
one-to-one dedup and delayed actor semantics; known-row text omitted while
counts increase only when released, nonowned custom choices remain; all
outcomes retained, observer disabled restores text; existing80-row ring and
unboxed overlay/scroll checks stay intact. No simulation or progression edits.

Validation2026-10-07: final scoped combat-identity/combat-rules/watch-log/
gunner-observer/tactic-observer5/5PASS40.1s; types,copy1765zero,build and
diff pass. Exact attack/hurt dedup counts retained after both now share arrow
wording; hurt-only paused batch survives subsequent hero-name change.
Synthetic custom rows explicitly lack origin; owned corridor row counts only
on release, actual damage remains, disabled observer restores rule text.
Initial regressions exposed old wording assertions and a fixture assigning a
getter-only rules field; corrected fixture uses the actual selected set.
No timeout or numeric gate reduced. Checkpoint screenshots400/1440 in
scratchpad/qa-cut104-combat; synthetic fixture, not a scored playthrough.
Current broad acceptance still required before shipping-WASM freeze/scoring.
