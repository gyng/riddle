# Expressive combat style and trait emblems — 2026-10-06

Replace generic equipment silhouettes for four combat styles and four personality
traits with expressive creature emblems. Steady calm bear; Guarded armoured
tortoise; Bold charging lion; Hunter focused hawk. Skittish startled hare;
Unbowed battered badger; Light hands coin-stealing magpie; Iron gut curious goat
with potion. Creature emblems describe a disposition, not the hero's class/look.
Keep names/descriptions and Rust behaviour exact. No added controls or copy.

Acceptance: all eight distinct subjects/silhouettes readable in the existing
38px sockets; transparent backgrounds, grounded painted fantasy palette,
consistent bold outlines. Store non-destructive generated masters and exact
built-in imagegen prompts. Pack through the established UI asset pipeline;
existing IDs/fallbacks stay intact. Inspect desktop/mobile Tactics captures,
run scoped UI geometry, copy and build. Push, no deployment.

Implemented: eight built-in imagegen masters in art/ui/icons/pkg_<id>_v2.png,
exact per-asset prompts in art/prompts/style-personality-v2/. Existing originals
are retained. tools/ui-skin.py packs96px transparent PNGs and lists them in
skin.json; packageIcon prefers the listed v2, then old icon/action/glyph fallback.
No CSS dimensions, names, rules, unlocks or equip behaviour changed. Production
serves only the packed assets (~164KiB for all eight), not the large masters.

Validation: packer accepts every new transparent master with no unusable assets;
all eight decode in desktop/mobile menus at their existing28px equipped-mobile
or38px chooser sizes. Headed all-unlocked visual fixture at400/1440, no horizontal
overflow; screenshots inspected/shown. The visual fixture exposes all four
traits together, not a claim that a real fresh hero owns them. Existing selection
28 checks and geometry184 checks pass. Tsc/build, copy1492 and diff check pass;
built bundle includes new manifest entries. No Rust changes/full balance audit
or deployment. First parallel generation attempt completed four styles and
failed on a trait request; separate trait retry completed all four.

Local artifacts: scratchpad/style-personality-20261006/{400,1440}-{styles,traits}.png
and icon-preview.mjs; scratchpad/style-personality-{pack,preview,tests,build}.log.
