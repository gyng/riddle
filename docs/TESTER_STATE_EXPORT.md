# Tester debug export — 2026-10-06

Owner request: testers can dump state so we can debug. Status: implemented
and verified locally, not deployed. This extends Settings' existing save/rules copy export; it is a
local downloadable diagnostic file, with no upload or deployment.

## Tester flow

Settings → Debug export → download `riddle-debug-<build>-<timestamp>.json`.
Keep the existing save import/export and rules sharing available. Show a busy
state during capture and a readable error with retry if capture fails. Provide
copy/manual-save fallback when downloading is unavailable on mobile. No need
for a tester to open developer tools or assemble several files.

## Bundle contents

- Versioned diagnostic schema, capture timestamp, application build/commit,
  engine kind/version and diagnostic completeness status.
- Fresh complete Rust engine save plus the existing importable SaveBlob,
  including all bloodlines, active hero, rules/loadout and saved rule origins.
  Verify actual engine save coverage rather than assuming the visible hero is
  the whole town. Include UI editing state separately when not yet committed.
- Current screen and relevant report/death/run IDs, watch mode/replay position,
  selected bloodline and game preferences that affect the reported behavior.
- Bounded recent application errors and diagnostic actions, browser/viewport
  and renderer information useful for UI bugs. Do not dump unrelated browser
  storage, full URL query strings, cookies or credentials.
- Snapshot provenance: when engine state was obtained and whether optional
  context was unavailable. Include the previous checkpoint only as an explicitly
  labelled fallback, never describe a stale save as a fresh capture.

## Implementation notes from the current code

`web/src/ui/settings.ts` already copies save/rules to a textarea/clipboard.
`App.exportSave()` currently serializes `lastSave`; `App.flush()` fetches
`engine.save()` but catches failure. A diagnostic exporter must explicitly
handle fresh-save failure and capture at an ordered worker boundary. Merely
calling flush and then exportSave can silently return stale state. Avoid mixing
fresh engine data with UI context sampled from a different hero/mutation.

Reuse existing save format and worker serialization. Do not perform forecasts,
verdict replays or catch-up simulations merely to export. Capture bounded logs
with negligible steady-state work; this debugging feature must not slow play.
Add a local developer reader/reproduction path that extracts the save and
context without requiring a tester to understand the bundle structure.

## Acceptance for implementation

1. One action produces one parseable versioned file at desktop and mobile
   widths (1440/400), with download and fallback paths exercised.
2. Actual shipping Rust save survives extraction/import and round-trip; all
   bloodlines, rules, inventory and progression match the captured state.
3. Export during watch, immediately after an edit/purchase and after offline
   catch-up is coherent. Export performs zero simulation queries and changes
   no game state; subsequent deterministic outputs remain identical.
4. Failed engine capture is explicit, retry works, and optional diagnostic
   failure does not discard a valid save. No silent cached-save success.
5. Build/screen/hero context and bounded error history are present; unrelated
   browser data is absent. Schema migration and malformed-file behavior are
   checked in the developer reader.
6. Tsc/copy/build plus scoped export/import and real-engine checks pass;
   checkpoint screenshots shown. Push authorized; deployment remains manual.

Priority: tester-support feature ahead of further Tactics query optimization.
The performance/design queue remains active after it.

## Implemented behavior and evidence

Settings has Debug export and Copy dump, using the existing chunky sheet and
textarea. A fresh ordered engine save is required; no flush/cached-checkpoint
fallback. Editing/hero/engine changes during the request retry up to three
times, then fail explicitly. A stalled engine request times out after 30s.
The UI exposes failure/retry; a download failure leaves the complete text and
copy control available. Other save/rules exports clear diagnostic-only status.
Capture does not write the save or invoke gameplay/simulation methods.

Schema 1 embeds the existing save envelope and exact Rust Session save, which
contains the selected game plus the other bloodlines. UI drafts/over-budget
rules are separate from engine truth. Screen/report/death data, watch dataset
including available replay frame, and live context are sampled at request time,
explicitly labelled; they are not claimed to be the same frame as the later
worker snapshot. Save receipt and request times are both included. Build commit
includes a dirty suffix when applicable; source archives fall back to unknown.

Diagnostics keep 64 entries of at most 2000 characters: screen transitions,
rule edits, global errors/unhandled rejections, worker errors and export errors.
They cover this page session, not errors from before launch or every console
message. URL credentials/query/hash are stripped from recorded HTTP URLs.
Environment uses an explicit preference allowlist, viewport, browser and canvas
dimensions; renderer timing/GPU profiling is outside this feature. Optional
environment failure is labelled and does not discard the valid save.

`web/tests/debug-export.mjs` passes at 400/1440 using the actual shipping WASM
engine: two bloodlines and selected hero retained, engine save byte-exact after
capture and import, subsequent 30-minute offline reports/saves exact, capture
calls save only, immediate Forge purchase and offline report captured exactly,
in-flight run captured and restored exactly. Failed saves, changing drafts,
continual changes, unavailable optional diagnostics, bounded history, unrelated
storage exclusion, URL sanitization, download/copy fallback and retry pass.
The complete export gate takes about 5s headless. Existing Settings stack and
report gates pass; tsc/production build/copy lint1492 pass. No Rust edits or
full simulation/balance rerun. Headed 400/1440 screenshots/downloads are in
scratchpad/debug-export-20261006/. Layouts were tested in Chromium, not on a
physical phone/Safari.

## Developer reproduction

Run `node tools/debug-export.mjs dump.json --save /tmp/repro-save.json`.
It writes an importable save and `/tmp/repro-save.json.context.json`, preserving
build, capture provenance, UI context and diagnostic notes. Paste the save file
into Settings' save import in the matching engine/build. Start with saved state;
importing opens camp, so the context file describes the original screen/replay
and drafts to reproduce. It does not automatically replay past UI actions.

The reader accepts schema1 with legacy v1/current v2 save envelopes, rejects
unknown schemas, cached provenance, malformed engine JSON and invalid save
fields, and refuses to overwrite existing output files. Full semantic engine
validation is performed by the actual game loader, not this JSON reader.
Actual captured-file extraction and live-save loader round trips pass.

