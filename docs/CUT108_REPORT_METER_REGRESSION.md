# Cut108 — keep the report regression aligned with approved scope

Current broad browser run has one Cut29 assertion expecting this night15m,
while approved Cut105 correctly names Completed runs and separates camp rest.
Update the expected title only; preserve15m, gold$20/min, fight13%, travel67%,
and add exact meter data-seconds900. No app/core/WASM or copy changes.

ScopedCut29 meters and next complete client acceptance must pass. Preserve
original broad failure as evidence; do not call an unfinished/mixed run green.
The frozen production candidate remains ad71e72, unchanged.

ScopedCut29:meters61064terminal1/1PASS5.2s(10unchangedfunctionalchecks,
plus exactseconds900 within report assertion). Original broadrun retains
reportedfailedoldtitle. App unchanged. Nextwholecurrentclientqueuedafter
nativefreshstress; default7pool/heavy1, allassertions/timeouts unchanged.
