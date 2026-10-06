# Boss warning words — 2026-10-06

The real Warlord watch says `rallies` in its warning and combat history.
Rust defines that pending action as calling more goblins. Show `calls goblins`
on the hostile nameplate and combat log for the Warlord and Captain, retaining
the core event, timing, flavour callout, counter, and glyph lifetime.
Other actors and unknown warnings keep their original words.

Acceptance: recorded rally renderer checks at400/1000 retain warning priority,
lifetime and visibility; mounted combat log checks at400/1440 preserve delayed
actor identities, raw damage and unknown warning fallback. Inspect headed
phone/desktop screenshots; typecheck/build/copy lint pass.

Baseline evidence: scratchpad/warlord-watch-20261006/. Real WASM loaded an
existing advanced camp, used its legal D5 start, and stepped to D8 before opening
the normal live watch. Both widths saw the Warlord rally7862/7902, break8109,
and die8154; no page errors or horizontal overflow. This is encounter coverage,
not fresh-player balance, performance, or a complete run/reward playtest.
Desktop entrance sprite extends above the canvas at this zoom; investigate
boss framing separately rather than claiming the warning edit fixes it.

Verified: shared display formatter changes only these two known actor kinds;
combat warnings are captured with their batch before delayed release. Renderer
34 checks and combat identity52 checks pass in29.6s. Initial fixture reused an
existing damage target ID; unique warning actors corrected the fixture without
changing production code. Typecheck/build, copy1528/zero violations and diff
check pass; existing bundle-size advisory. Headed400/1440 recorded real rally
snapshot renders `warlord · calls goblins`, screenshots inspected. Recorded
snapshot preview omits the app chrome and is not a new live playthrough.
No Rust, WASM, balance or deployment change.
