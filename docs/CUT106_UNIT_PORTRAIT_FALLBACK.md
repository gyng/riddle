# Cut106 — loaded portrait fallback

Original QA-b screenshot shows goblin killer portrait present, but Corin's
historical hero frame empty. Exact original request state is unavailable;
do not claim a proven root cause. Current unitIcon returns an image for listed
art with no error fallback. A failed portrait request can therefore leave an
empty frame despite the known class and an existing atlas primitive path.

Reproduce with failed hero_rogue portrait request on an actual death component.
On image error replace only that icon with the existing same-unit atlas crop/
primitive. Do not use the selected current hero's class or another unit; keep
frame size, historical identity and adjacent text. Healthy art stays untouched.
Unknown atlas art keeps the visible primitive. No game/save changes.

Gates: normal historical rogue decode; aborted portrait request produces same
hero_rogue atlas fallback; absent atlas retains primitive; existing unit/death
identity and frame/spacing checks320/400/1440; exact save unchanged.

Failed-before95867 terminal0/1: failed image left no same-unit atlas fallback.
After repair72728 terminal4/4PASS10.1s /tmp/riddle-cut106-client.log: new failed
portrait regression plus existing unit-icons/death-hero/death-actions. Healthy
rogue image decodes before abort; same-unit hero_rogue atlas after error;
unknown identity retains visible primitive. Frame64/text/save exact. Types/
build35682PASS/copy1768zero/diffPASS. Diagnostic screenshots320/400/1440 in
scratchpad/qa-cut106-portrait;phone inline. Original root cause unproven;
robustness repair addresses confirmed lack of error fallback, not a claim
that original outage caused this image. Broad acceptance still required.
