# Unit identity icons — 2026-10-06

Put a portrait or existing sprite beside seen enemy names in the unit guide,
beside the killer name on death banners, and beside a recorded fallen hero's
name. Use historical hero class, never the currently selected heir. Unseen
reference rows remain anonymous. Reuse packed art with sprite/glyph fallback;
no new simulation, generated assets, or controls. Keep existing hover details,
cause tap-to-trace, and narrow-screen status columns.

Acceptance: icons decode beside known names, undiscovered identities concealed,
historical class ownership, phone320/400 and desktop1440 no horizontal overflow,
existing death/tooltip input regressions; actual WASM screenshots. Build/copy.

Verified: new unit-icon checks at320/400/1440 cover portrait decoding, guide
concealment, missing-art primitive, name adjacency, historical class and bounds.
Death input/identity/action regressions4/4 pass13.1s. Final build/typecheck and
copy1555 clean; existing bundle advisory. Headed actual WASM saved town guide
and Warlord death400/1440 captured without warnings/overflow, screenshots shown.
Evidence: scratchpad/unit-icons-20261006/. Shared icon uses packed portrait,
existing sprite, then primitive; no generated art or Rust change. No deployment.
