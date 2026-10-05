# Distinct bloodline family names — 2026-10-06

Real two-slot away screenshots show two active Nialls. Preserve existing given
name derivation; add a deterministic family name tied to persistent slot id:
Ash, Thorn, Flint for the three supported bloodlines. Every heir in a slot
inherits the same family name; distinct slots have distinct complete names
even when given names coincide. Unknown imported ids get a unique numbered
family fallback. No RNG consumption or simulation change.

Core current roster, hero lifetime wire and newly captured deaths use the same
full identity. Already stored nonempty historical names stay verbatim; empty
lifetime names derive their family identity on cloned wire reads. Old unnamed
deaths remain unnamed. Given names still cycle after24 heirs; no custom naming.

Acceptance: all three slots are distinct for matching given names and changing
heir ordinals; names stable through selecting/reloading/adding a third slot;
death captures run heir rather than replacement and old death names stay fixed;
identity queries do not mutate raw save. UI full names fit320/400/1440, hero
menu/Chronicle use core source. Fast Rust tests/clippy, rebuilt WASM, build/copy,
targeted UI and headed real previously-colliding saved camp/screens. No deploy.

Verified:554 fast workspace tests pass38.55s/1 pre-existing ignored; clippy,
rebuilt WASM5241995bytes, build/typecheck/copy1511/diff pass. Existing gameplay
replay hash retained. UI hero identity30 checks320/400/1440 pass4.4s including
actual child bounds for full name/class/level in320px menu; historical death27
checks pass7.7s initial batch. Historical captured slot id/heir and stored old
name compatibility covered by Rust tests; no backfill of old unnamed deaths.

Headed real savedseed15camp/legal second bloodline/two12h slices reproduces
79 runs and two Nialls: Niall Ash, Niall Thorn. Exact load and2→1 selection
retain identities; legal third purchase yields Bryn Flint without renaming
existing heroes. Core source names match roster/menu400/1440; no warnings or
horizontal overflow, screens inspected/shown. No new real death-screen walk;
core/UI test evidence covers death naming. Same raw saved camp and runtime as
previous training proof; wall clock workers may spend during headed UI checks,
so screenshot gold differences are not a name-related simulation change claim.
Artifacts scratchpad/hero-families-20261006/{400,1440}-{roster,hero}.png,
*-real.json, real-preview.mjs and test/build logs.

Complete generated names are distinct across supported slots, even arbitrary
heir ordinals, because family strings are disjoint. Given names may still match
and cycle after24 heirs. Family names currently repeat across different towns;
no town-global registry. Stored explicit historical names remain unchanged.
