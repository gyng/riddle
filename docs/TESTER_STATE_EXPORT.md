# Tester debug export — queued 2026-10-06

Owner request: testers can dump state so we can debug. Status: queued, not
implemented. This extends Settings' existing save/rules copy export; it is a
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

Priority: next tester-support feature, ahead of further Tactics query
optimization. The performance/design queue remains active after it. This
contract records work to implement; it does not claim an export UI exists yet.
