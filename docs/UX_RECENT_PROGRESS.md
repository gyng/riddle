# Record and recent runs — 2026-10-06

Actual progressed-camp screenshot showed Progress stopped floor20 above recent
banked runs with none past13. Core lead is the historical selected record;
stall depth is the recent window. Show Record floor20 / Recent best floor13
from those existing payloads, not current live state or another hero. Do not
claim the whole absence returned home or made no new record: earlier runs can
have died or set records before the current plateau window.

Recent runs replaces Runs ended. Known bank/return causes read Collected gold /
Returned home with the original run counts, earlier-run distinction and exact
floor. Unknown/old text keeps its readable fallback. Decision log and pre-pen
gating remain. Report suggestions use plain action/condition labels for known
boss attack, rest, health/floor/carry conditions; preserve unknown syntax and
all thresholds. Patch payload, ordering, drop/buy/apply mechanics unchanged.

Gates: fixtures at320/400/1440 cover record20/recent13, equal floors, unknown
legacy text, absent stall, earlier-only/mixed counts, pre-pen suppression,
boss/health/floor/removal wording and exact patch application. No horizontal
overflow. Update existing qa92 copy assertions to the new truthful contract;
retain its exact forecast/trace/news assertions. Relevant report/patch client
checks plus build/typecheck/copy/diff. Headed real camp-3 screenshot at400/1440
before/after and source values. No Rust, tuning, deployment or new simulation.

Verification: recent-progress60 checks320/400/1440, existing qa92 report8
checks, report-actions32 checks400/1440 all pass in4.0s. Build/typecheck,
copy1523/diff pass; existing chunk advisory. Real rebuilt-WASM camp-3/8h
Details400/1440 shows source record20, recent13, collected4runs and Target
the boss/Boss in sight, no warnings/overflow. Screenshots shown. Mobile
forecast below action avoids three-line title; first screenshot caught this
layout issue and final screenshots/checks repeated after correction. Opening
Details is required for this section; initial screenshot attempted scrolling
to its hidden title and was corrected. Native simulation/WASM artifact unchanged.

Full historical qa92 fails before the report: rough-forecast ellipsis assertion,
obsolete unlock shortfall selector and timeout. Unchanged HEAD test reproduced
the same failures. Added --part=report for independently running its existing
report assertions; default still runs all its original sections. Report subset's
old tile-label expectation updated to current exact order/labels (including
summary stats), then all eight checks pass; no assertions dropped. The old
camp failures remain queued, so this is not a full client-suite pass. Temporary
baseline script removed. New fixture's removal selector initially indexed a
second target despite only one having a target; corrected and rerun.

Source/report payload, patch/drop/buy handlers, old text fallback and threshold
values preserved. No balance/FPS claim/deployment. Earlier forecast optimisation
docs overstated report-actions count as48: corrected to32 (two widths), while
report-hero-upgrade remains57 (three widths).

Follow-up: the historical qa92 camp failures above are now resolved; see
DEV_QA92.md. Full40checks and report-only8checks pass with all sections kept.
