# Cut49 — distinguish hero progress from class training

Home Details currently leads with an unqualified class level; the active roster
shows the separate hero level. Make the menu agree with the roster: named hero,
class, hero level/XP, actual current presence. Label class level explicitly in
folded Class XP details. Shared presence presentation must retain exact existing
run/slot alignment and old-wire fallbacks. Observe onChange/onLive without
advancing, writing rules or spending. Missing hero slots keep legacy fallback.

Acceptance: deliberately different hero/class levels at400/1440 are labelled
correctly; XP/status changes refresh an open menu; switching class and appearance
keeps working and Details expansion survives refresh. Opening/closing actual
earned menu preserves full save. Zero overflow/errors, screenshots, existing
presence/name/appearance/class gates, TS/copy/build/diff. No deployment.

Accepted 2026-10-07. Hero menu72×84 framed portrait and explicit Hero level/XP;
Class level/XP clearly separate in Details, including full progress at MAX.
Shared presence helper extracted without changing its alignment/fallback rules;
menu refreshes through existing onChange/onLive and unsubscribes both on close.
Class and Appearance actions moved above the Legacy tree. Parchment presence
uses dark status colours after screenshot review exposed poor contrast.

Fresh headed first-session walk before edit:31.5s,9 captures, real house/manual
sends/scout/8h absence, no page/console errors; scratchpad/home-cut49-walk.
Final actual earned400/1440 menu screenshots and opening/closing exact full-save
equality; scratchpad/hero-cut49-final, /tmp/riddle-cut49-summary-final.log.
Separate controlled presentation cases use Hero L3/XP144→180, Class L10/XP900,
D13 combat→rest; verify labels, HP detail, open Details preserved, unsubscribe
and zero overflow without modifying engine save.9 checks each width, no errors.
These deliberately unequal values are diagnostics, not earned progression.

Final6/6 hero-summary/presence/names/appearance/class-switch/class-styles pass
26.3s, /tmp/riddle-cut49-client-confirm.log. Earlier concurrent appearance touch
geometry failure retained in client-final.log; same unchanged44px assertions
pass isolated and final concurrent. Added exact geometry to its failure message,
no tolerance/threshold changes. First new test launch used wrong creation path;
subsequent assertion matched roster Chronicle instead of menu Details; scope
corrected to hero-sheet, raw failures retained. TS/build/copy1739/zero and diff
pass. No Rust/WASM inputs changed; no deployment/full audit/95 certification.
