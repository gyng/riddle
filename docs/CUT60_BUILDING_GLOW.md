# Cut60 — home building shadow inspection

Owner noticed a box shadow under home buildings. Inspect current earned home
at400/1440: normal hit targets transparent, border0, shadow/filter none. Town
renderer draws no CSS shadows; authored sprite feet remain. Source identifies
4.6s `.town-hit.arrived` rectangular gold box glow. Remove only this obsolete
hit-rectangle styling; existing construction glint and arrival text remain,
keyboard focus outline retained. No sprite edits or gameplay/state changes.

Acceptance: all building hit targets, including explicit arrived state, remain
transparent/borderless/shadowless at400/1440, screenshots and build/diff.
Diagnostic arrived-class styling check does not claim a newly earned construction
playthrough or prove every possible shadow report has the same cause.

Result: explicit arrived-state styling at400/1440 PASS, all hit targets transparent
with border0/shadow none. Screenshot shown. Build/TS and diff PASS. No deployment.
