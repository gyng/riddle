# Cut101 — retain the report being read

QA-a had to reopen report details after every historical death. Keep expanded
state and scroll position per report object and app, in UI memory only. A new
report starts folded. Back from its death card retains the actual absence flag
as well as the same report. No persistence, game state or rewards changes.

Gates: actual verdict/back cycle twice retains expanded details, scroll and
absence flag on phone/desktop; manual collapse retained; a different report
starts folded; exact engine save unchanged; existing report actions/death
controls remain valid. Restore after mounting; cancel stale restoration on
disposal.

Validation: regression fails before with Back from death collapsed report
details. After repair320/400/1440 pass two actual verdict/back cycles, scroll
within2px, absence flag, explicit collapse, different-report default and exact
save. Four scoped suites pass8.9s: report-reading, existing report-actions,
death-actions and report-exit-depth. Type/copy1758/diff/webbuild pass.
UI memory uses weak keys per app/report and cancels stale restoration on
disposal. No Rust change since Cut100; current broader acceptance is pending.
