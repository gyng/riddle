# Shared game UI materials

`web/src/chrome.css` extends the packed skin, using the existing tablet/panel art
with CSS bevel fallbacks. Keep component identity separate from its material:
`log-tab game-control`, for example, retains all log routing and accessibility.

| Material | Usage | Treatment |
| --- | --- | --- |
| Management window | `openWindow` in ui/sheet.ts | Centered framed parchment; title plate; respects top bar/console; stacked Back/Close |
| Command | `.game-control` on a button | 44px minimum, carved tablet, bevel fallback, gold selected edge, focus/press response |
| Interactive record | `.game-inset` within a game window | Recessed parchment row; retain text/portrait hierarchy |
| Editable field | `.sheet-body .name-input`, `.ta` | Square recessed dark field; bone text; keyboard semantics unchanged |
| Packed command card | `.cmd .tile` | Existing tile art and uniform control type scale |
| Portrait/gem/stud | Existing component | Circular framed art is intentional; keep readable silhouette and separate hit target |

Ordinary `.chip`/`.btn`/`.card` controls in management windows and anchored
inventory panels already receive the shared command
material. Use explicit `.game-control` for controls with specialized names outside
that selector. Scoped material selectors must beat older component borders and
minimum sizes. Do not add another bespoke gradient for the same material.

Map hits/foundations, gauges, chart marks and plain combat-log text have their own
presentation; applying a button frame to them would obscure the scene. Item and
trait silhouettes remain unboxed. Rule tokens inside an existing tablet and
inline disclosure arrows remain compact parts of their containing component.

Cut50 audited actual earned Hero, Class, Tactics, Forge, Workers, Settings,
Chronicle, Runs, Appearance, Savings and Quest at400/1440. All22 have no page
horizontal overflow/browser errors and preserve full engine save during
inspection. Existing home/watch/report/death command frames, cloth/banner,
scroll, charts and token popovers reviewed in source/current playthrough artifacts;
those are not22 additional freshly captured windows. Supplies and Stored Gear
anchored panels also captured at400/1440 (four more views): same controls and
compact title plates, lit rarity colours on dark materials, panels above map labels.
Companions is not built in the earned fixture; its panel reviewed in source, not
claimed as an earned capture. Ascension number input
already has a recessed square field; native numeric/validation semantics stay.

When extending UI, use a shared material, inspect320/400/1440, measure settled
animation geometry, and check real routing/save behavior. See CUT50 for the
bounded regression evidence; this audit does not certify every hidden state or
replace an independent fun rating.

Cut54 extends the same command recipe to companion selection/Breed/Hatch/rules/
dismissal, death Details/Send again/Morgue, patch Others and next-worker shortcut.
Companion portraits stay outside the tablet; subordinate text uses the readable
secondary ink on dark materials. Diagnostic companion/death320/400/1440 captures
and interactions pass. These fixtures do not claim independently earned pets.
Inline ledger records and forecast chart bars remain quiet; retired runlane
capsules are inactive. Current built seed5 home/watch/return walk passes; seed7
Warlord screenshot captured before a retained stale Edit harness failure.
