# Cut95 — identify the boss on a repelled result

The earned QA-a Warlord D8 result has no portrait. Death rendering explicitly
excludes drive-offs, even though their wire record names the exact boss id.
The owner requested large framed enemy portraits above centered defeat text.

Use `line.driven.boss` for identity art and enemy details on drive-offs, retaining
the recorded display cause and actual HP. Reuse the shared framed112px portrait
and existing centered banner layout. Unknown boss ids keep primitive fallback;
ordinary death and non-enemy stall behavior stay unchanged. Do not guess boss
identity from display title. No core/progression/save changes.

Gates: driven fixture with display cause Warlord resolves exact goblin_warlord
art on320/400/1440, portrait is framed and above its text, spacing at least4px,
horizontal centers within2px and no overflow. Ordinary death art remains.
Actual hover details use recorded boss id and say Seen/Alive or Unavailable,
never Defeated just because the hero returned. Existing defeat checks pass;
capture completed-animation phone/desktop screenshots.
