# Cut 34 — Leeching elites

Contract,2026-10-07. Expand actual higher-ascension build decisions, beginning
with one elite mechanic rather than more scaling. Tier0–5 birth selection,
catalogue, events and saves stay unchanged. No deployment.

## Mechanics and compatibility

Ascension6+ keeps the existing deterministic1/8 elite birth rate and one elite
per natural non-boss enemy. Eligible selection chooses Shielded, Frenzied or
Leeching from the existing birth hash, without consuming RNG. Lower tiers keep
the original two-way selection exactly. Summons, bosses, pets, captives, nests
and strays retain existing exclusions; splits inherit once, taming removes it.

A living hostile Leeching enemy restores at most2HP on a positive direct melee
hit on the hero. Cap by its missing HP and actual hero HP lost (no overkill
healing). Require adjacency and a non-ranged source. Poison suppresses healing.
Hazards, reflection, ally attacks, zero damage and mirror-intercepted hits do
not heal. Hex/resistance/Legacy/Riposte reductions happen first; healing occurs
before a surviving hero's Riposte counter. Normal stun/Slow reduce opportunities
through their existing attack scheduler, not a new rule or grant.

An authoritative Recover event records id, amount, resulting HP and source.
It updates replay entity health and plain text combat log with existing healing
colour. Keep hostile recovery separate from player healing meters; existing
Heal events and tier0 event semantics stay unchanged. The played modifier
catalogue explains the effect and Poison/range counter; existing elite map
marking applies. No new monsters, per-tick scan, RNG, timers or history arrays.

## Acceptance

1. Tests prove exact lower-tier deterministic selection, new elite occurrence,
   unchanged RNG, one-elite limit, exclusions, split/tame and saved metadata.
2. Real damage paths prove10HP hurt→2HP recovery,1→1, missingHP cap, no
   overkill, poison/Hex/Riposte/mirror interactions, ranged/distant/allied/
   dead/hazard/reflection exclusions. Recover events match actual health;
   save/reload and death attribution preserve the played Leeching identity.
3. Native/WASM played events/save parity, whole/sliced/reloaded absence parity
   including Tier6 and one/three bloodlines. Old tier0–5 raw baselines remain
   unchanged. Actual campaign evidence is separate from controlled fixtures.
4. Real-worker watch/replay health and combat log,320/400/1440 tooltip detail,
   keyboard/tap and no overflow; show screenshots of actual played encounters.
5. Fixed earned preparation seeds1/3/5 Tier6 clears<=48game-hours; eligibility
   synthesis and any failed builds remain explicit. Poison counter comparison
   proves less actual enemy healing, not merely displayed metadata.
6. Equal-work overhead<=10% against e2e74c7, positive-hit mechanic covered;
   quick/core/client checks, clippy, rebuilt WASM/build and routine gate pass.

Partial core checkpoints do not certify the complete content cut. Subsequent
boss/loot variety and class-default training work remain in the owner queue.

## Verified content checkpoint — 2026-10-07

Leeching is implemented, with Rust-owned Tier6+ preview/catalogue, saved elite
identity, real on-hit recovery, death metadata, replay HP and green plain-text
combat log. Tier0–5 selection/catalogue stays exact. Five new meaningful tests
cover birth hash/exclusions, real10→2/1→1/missing-HP/overkill, poison/Hex/
Riposte/mirror and hazard/ranged/ally exclusions, saved/split/tame identity.
Existing absence test additionally covers Tier6. Full final quick636core PASS/
one ignored plus14tool tests, TS/copy1705 green160s under competing work.
All-target fast clippy warnings denied, codegen check, build and real fastWASM
5380527B pass; existing bundle advisory. Routine18case gate source
fccfd9ac8d76acb4 terminal0, no cached legs: wire50.2s, metrics84.7s,
dayplayer203.5s/all selected bars PASS. Current-source recheck confirms the
same compiled-runtime key and all three genuine completed cached legs. Test-
only trace assertion dereference preserves the complete comparison.

Six Tier6 cases use the same actual earned parent XP/owned kit/policy and162
paid Legacy from Cut33, synthetic seed1/3/5 and tier eligibility, no campaign
grants/edits. Sentinel clears16/24/24game-hours,72/133/146 recoveries,
75/145/168 enemy HP restored. Hexbinder16/16/16,53/75/38 recoveries,
61/87/42 enemy HP restored. All reachD34 within48h. This is fixed-build
viability, not legally earned Tier6 progression or every higher-tier balance.
Permanent examples/leech_check.rs; artifacts scratchpad/leeching-20261007/.
A separate normal stepped D19 encounter from that diagnostic preparation
contains a Leeching iron golem recovering1HP after an actual1HP strike; no
arena/encounter/HP overrides. Entire native/WASM step/snapshot and before/
after save bytes equal the original recorded checkpoint after optimization.

Final real-worker tests320/400/1440 prove actual recovery, whole-step/save
reload equality, poison counter, replay HP including seek, green log entry,
three-elite Ascension6 preview/cancel full-save read-only and tooltip details.
Actual headed GPU watch screenshots at all three widths load the full source
exactly, show the normal recovery log, have no errors/overflow and preserve
existing elite marking. History entry was explicitly scrolled into view for
final images; earlier screenshots had it outside the visible three-line history.
Temporary5482 and browser cleaned up, shared5219 retained. Capture pre-compaction
JSON remains byte-exact under the final engine, so played screenshots represent
identical event/state behaviour. Artifacts scratchpad/leeching-qa-20261007/.

Final native/WASM8h whole/half-hour/uneven-reloaded entire reports/saves match
both selected classes with one/three legally funded bloodlines; each slice
response/save exact. Six original tier0/challenge loaded/advanced complete
WASM baselines exact. All twelve older Tier5 style/base seed1/3/5 full campaign
start/after saves remain exact, including8/16/24h outcomes. These verify the
specified fixtures, not an exhaustive migration/system-removal audit.

Initial positive-hit microbenchmark failed23.12%; allocation-free recovery
source alone still failed21.62%, both results retained. Spawn Entity and exit
Trace payloads are now boxed, preserving JSON while reducing native Ev from
208 to80bytes. Rare spawn/exit allocations replace excess storage on every
ordinary event; recovery source is a bounded enum, not a per-hit String.
Final equal-work500tick/damage calls,32foes,101samples,9counterbalanced rounds
versus e2e74c7: median ns/call tier0 none517.400→510.860 (−1.26%);
tier5 none540.020→534.920 (−0.94%); tier6 none546.740→535.560 (−2.04%);
positive Leeching537.080→543.120 (+1.12%). All meet10%, each500-hit positive
sample restores exactly1000 extra enemy HP. Same9hero resets included; parsing/
map generation excluded. No full catch-up/FPS claim. Permanent leech_perf.rs;
source/binary hashes and full interleaved rounds retained in perf.json.

Cut34 is complete within these acceptance boundaries. No deployment; optional
exhaustive/statistical/historical audits remain separate. Owner now explicitly
requests Gunner with distinct long/short guns: CUT35_GUNNER.md is next priority
before Rogue/Ranger styles. Contract and transparent equipment icons prepared;
Gunner gameplay/unlock/class/portraits/forge/reload UI are NOT implemented yet.
Keep-going goal remains active. mine.bars and scratchpad remain unstaged.
