# Trait expressions — 2026-10-06

Improve personality in the Unbowed and Light Hands emblems. Preserve the existing
badger and magpie identities, painted ink style, colours and transparent
silhouettes. Unbowed should read as brave defiance; Light Hands as mischievous
loot collecting. Preserve the other six emblems and existing 44–52px sizing.

Acceptance: both sources have transparent corners and pack to96px; all eight
selected style/trait emblems decode, remain unframed and do not intersect labels
or overflow at320/400/1440. Inspect headed phone/desktop screenshots. Existing
package-selection and death-action checks, typecheck/build/copy lint pass.
No gameplay, save, simulation or deployment changes.

Verified: built-in imagegen edits saved as
art/ui/icons/pkg_{unbowed,light_hands}_v4.png, with exact prompts under
art/prompts/trait-personality-v4/. Both1254px RGBA sources have fully transparent
corners; tools/ui-skin.py packed96px runtime files and added only those two
manifest entries. Existing packageIcon v4 preference needs no code change.
The magpie output uses a narrowed sly gaze rather than the requested full wink;
accepted after inspecting its coin-palming silhouette at game size.

Headed all-unlocked fake-engine visual fixture at320/400/1440: all eight selected
emblems decode at44–52px, the two traits use v4, no icon/text intersections,
frames or horizontal overflow. Phone/desktop screenshots inspected and shown.
Package-selection28 and death-actions44 checks pass3.7s; build/typecheck,
copy1524/zero violations and diff check pass. Existing bundle-size advisory.
Evidence: scratchpad/trait-personality-20261006/. No Rust/WASM/gameplay/deployment.
