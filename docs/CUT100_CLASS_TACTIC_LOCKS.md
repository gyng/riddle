# Cut100 — show the requirement that remains

QA-a's Ready class menu hid the core unlock reason despite enough upgrade
tokens. Both QA players saw locked tactics still demand a Warlord kill they
had already completed. Show core class needs outside the disabled button,
shared with the older camp picker. Tactics retain their existing arrival
schedule; derive next-send eligibility from the same pure arrival plan used
by arrive, otherwise identify boss/day progression after Warlord is slain.
No free unlock, change of arrival order, prices or progression.

Gates: class gate/token-shortage reasons visible in phone/desktop actual menu,
buyable/owned class lacks a false lock, save unchanged on read; core tactic
triggers before kill, eligible next send, waiting for progression, directly
owned card, and cadence clear all match actual arrivals; existing class and
package tests remain intact. Rebuild real WASM after Rust changes.

Validation: new core next-send/waiting/direct-card/read-only regression passes;
all686Rust tests pass/one ignored. Complete verify --quick passes69s including
14JS helper checks. Clippy/type/copy1758/diff/webbuild pass. Real fastWASM
rebuilt5521523bytes. Three scoped client suites pass27.4s: class-locks across
320/400/1440, existing class-switch, package18checks. Screenshot exposed pale
requirement text on parchment; changed to existing ink-text palette, unchanged
responsive recheck passes7.2s. Settled fixture screenshots in
scratchpad/qa-cut100-class-locks. Arrival ordering and progression remain the
same; pending_arrivals is the original computation shared with the wire.
Broader current-game and client acceptance remains required before scoring.
