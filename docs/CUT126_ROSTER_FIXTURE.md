# Cut 126: keep the roster fixture coherent across system acknowledgement

2026-10-11. Complete Cut125 acceptance finished155/156 in1596.0s, exit1.
Only `live-roster` failed: `send starts one slot read before viewer`.
The entire result is retained; no raters released.

Controlled BEFORE reproduction waits2100ms in the real camp (its new-system
acknowledgement threshold is1800ms). The test supplies two hero slots only on
`app.lineage`; the unmocked fake engine's `seenSystems()` returns its original
lineage with no hero slots. The watch correctly adopts that response, so its
subsequent slot read is skipped: diagnostic fixtureSlots2, returnedSlots absent,
jobs0. Same assertion fails, exit1. The original full-run frame was not traced;
this establishes a deterministic route to its observed failure.

1. This mocked roster scene must provide one coherent authoritative lineage
   through both `seenSystems()` and its existing controlled `lineage()` reads.
   Force a pending acknowledgement and assert it happens exactly once.
2. Preserve every original gate: one initial slot read before viewer, pending
   Ready, read deduplication, live/rest/wait updates, untouched other slot and
   XP/Legacy, no clock advance, late view/selection reply rejection, retry after
   failure, camp no-read, and bounded layout. Keep the100×30ms polling budget.
3. Both400/1440 viewport cases pass; then complete current156-group acceptance.
   No application, simulation, resources or numeric gate changes. Preview stays
   b6df4f2; no blind raters before terminal whole acceptance.

Verification: BEFORE exit1 with acknowledgement replacing both fixture slots;
AFTER exit0,20 checks each at400/1440 after the same2100ms camp delay.
Focused runner PASS1/1 in2.9s. Original assertions and polling budget unchanged.
Source/application freeze unchanged. Complete156-group rerun still required.
