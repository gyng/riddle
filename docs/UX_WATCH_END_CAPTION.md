# Watch outcome caption — 2026-10-06

A real completed Bold run still displays HIT WEAKEST SPECTRAL above the
hero's death scene. Clear the current-action caption when playback reaches
the hero's death or any run exit. Enemy deaths must retain the hero's action
caption. Preserve outcome/cause callouts and plain combat history. Clear only
at the viewer's event clock, never when the engine runs ahead.

Numeric checks: live action visible before death; no action caption at hero
death, death/bank/return exits, or later queued rule events; enemy death retains
caption. Seek back to an alive action tick and loading another floor restore
normal caption behavior. Real WASM final frames at400/1440 show Run ended,
end cause and history, no stale action, no overflow/errors. Targeted watch
checks, build/typecheck/copy lint. No Rust/state-wire/gameplay/deployment changes.

Implemented a presentation-only end flag in ReplayState, reset by load/seek.
Hero death and every exit clear the action caption; later rules cannot restore
it. Enemy deaths and non-action callouts are unaffected. New19 checks cover
live/enemy/hero/late events, all three exits, future queued exits, seeking to
living/dead ticks and new-floor reset. Four targeted suites pass28.9s, including
frame63checks and existing log/status regressions. Build/typecheck/copy1570
pass; existing bundle advisory remains.

Actual headed WASM earnedseed3 legalBold run66 diesD6 at400/1440: debugText
contains no captions, Run ended remains visible, cause/history have8px gap,
no errors/overflow. Watched2nd heir stays while engine becomes3rd, gold8443.
Used actual fast-skip control; held only verdict presentation for final capture,
then released to real death screen. Initial capture retained Speed sheet; closed
it and recaptured unobstructed screenshots. Artifacts scratchpad/watch-end-
caption-20261006/. WASM unchanged. No deployment.
