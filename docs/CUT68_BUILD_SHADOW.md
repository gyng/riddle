# Cut 68 — Build plaque silhouette shadow

Owner requests the Build building button's shadow follow its actual border,
rather than the DOM rectangle. Replace its outer box shadows with alpha-based
CSS drop shadows. The painted button skin has no box shadow; retain the fallback
skin's inset bevel. Hover, press and busy brightness must compose with the
shadow rather than replacing it. Preserve placement, construction rules,
keyboard focus and minimum 44px target. No asset/core/deployment changes.

Validation: production web build/TypeScript passes. Fresh real-WASM empty towns
at 400 and 1440px show button.png border art, no painted-skin box shadow and
composed drop shadows at rest and hover. Keyboard focus outline and minimum
44px height pass; exact saved state unchanged and no page errors. Desktop
screenshot shown. Capture/results: scratchpad/qa-build-shadow; raw logs:
/tmp/riddle-build-shadow-{build,ui}.log. Existing build chunk warning remains.
