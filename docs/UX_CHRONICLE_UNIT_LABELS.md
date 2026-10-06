# Historical unit labels in the chronicle — 2026-10-06

Finish the shared unit label pass for archived heroes. Use hero_legacy class
and name when present; otherwise use only the class explicitly printed in the
canonical chronicle header (♟N the trait class ·). Other bloodline entries use
their own printed header. Never infer class/name from the currently selected
hero, and retain text-only fallback for old entries without identity. Preserve
chronicle strings, ordering, existing kept-death links and other-slot actions.
Opening the sheet is read-only. Fixed32px art, shared8px gap, wrapped text.

Acceptance: selected/other ownership and old fallback; kept-death action still
loads original id; gap8±1px, vertical center±1px and bounds at320/400/1440; actual
WASM phone/desktop screenshots and selection/save unchanged on open; build/copy.
No simulation/art/deployment changes.

Verified canonical Rust formatter at engine.rs chronicle_heir_at: header is
♟N the epithet class, followed by · records. Tests at320/400/1440 cover metadata
precedence, other-slot printed class, old text-only fallback, open without
selection, exact saved-death id17, and8px/center±1px geometry. Unit-label/death
identity/report regressions3/3 pass4.6s. Actual headed WASM two-slot save:15
entries per400/1440 capture have correct fighter icons/gap/centering/bounds;
immediate engine save and selected2 unchanged when opening. Screenshots shown.
Final build/typecheck, copy1556 and diff clean (existing bundle advisory).
Evidence: scratchpad/chronicle-unit-labels-20261006/. No deployment.
