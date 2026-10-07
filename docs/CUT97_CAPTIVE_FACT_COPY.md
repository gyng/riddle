# Cut97 — distinguish a recruitable captive from an ally

Independent QA-a read `captive · ally` in learned facts, then watched attack
captive/fell without an allegiance-change line. Rust's definition gives the
neutral captive an ally tag for learnable vocabulary; only free_captive sets
actual ally=true. Ordinary combat may attack an adjacent chained stair blocker.
No evidence that an actual freed ally was attacked.

Display that learned tag as `potential ally` in report facts and enemy traits.
Keep all persisted fact/tag IDs, rules, target selection and allegiance intact.
Reuse one display helper; ordinary enemy tags retain their current spelling.
This corrects identity copy, not the player's chosen rescue policy.

Gates: report learned captive fact says potential ally, ordinary tag spelling
unchanged; actual enemy tooltip says Potential Ally; exact saved engine stays
unchanged. Regression fails before repair, scoped report/tooltip checks pass.
QA-a original unexplained observation remains in its report, not rewritten.
