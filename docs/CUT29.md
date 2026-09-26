# Cut 29 — The curve keeps opening; a tap is a decision

*Contract, 2026-09-27. Cohort 24 (build 9720ff7): 75.1 · 71.4, α 0.877, mean 73.25 (+7.55 over the
control). Design: `docs/PROGRESSION.md` (measured 14-day timeline + re-tier; chart page
https://claude.ai/artifact/Lt3hD2tNR81Kj6AwmZVkhc), the owner's requests (`scratchpad/queued/cut29-inputs.md`),
`docs/PLATEAU.md` "Cohort 24". Traits are Cut 30 (`docs/TRAITS.md`, `docs/CUT30.md`).*

## Design

### 1. Progression re-tier (core + client) — `docs/PROGRESSION.md` §3–5
- **Marks income**: the frontier mark (◆1 per bank at ≥ best − 1; 78 % of rater marks — it pays for the
  same D13 bank every run) is removed; ◆1 per day whose absences brought a send home.
- **The catalogue in tiers T1–T6**, opened by meeting band bosses; every price ≤ ◆8; rows monotonic
  (5–10: 3 · 4 · 5 · 6 · 7 · 8); condition words free (they never moved a forecast, 0/18); automations move
  to gold; cards that never mattered either change (§3 of this cut) or leave the catalogue.
- **Late sinks**: oath draws (◆2, repeatable), oath slots 2–3, an oath reward pool of its own (route 2,
  heir pick, party slots, a rule set per band) so the Cut 28 sink doesn't empty with the catalogue.
- **The dayplayer's two bars become hard gates** (days with an unlock ≥ 10/14; best-depth stall ≤ 3) —
  `gates.mjs` drops the "informational until M7" carve-out. The known strength wall (dayplayer D17,
  7–12 days) is attacked by experiment E1 (the plateau search's best one-row edit offered at a wall); if
  E1 does not break it, a Crypt content ramp; record the result, never loosen the bar.
- Gate: metrics rows from `examples/progression.rs` (unlock days ≥ 10/14 mean over rater sets and the
  dayplayer; marks unspent ≤ 8 at the worst check-in; purse ≤ 1.5 days' net; stall ≤ 3 on ≥ 13/18 rater
  lineages); the Cut 25 lever row; bots unchanged.

### 2. Systems open one at a time (core + client) — `docs/PROGRESSION.md` §6
- Each system opens at its trigger moment (the table in §6: reorder at the first plateau, the vs line at
  the first plateau, the divergence at the Warlord met, the forge at the Warlord slain, the D5 route at
  the fork seen twice, automations at the Lich met, …); no tutorial text; the reveal glint on arrival.
- Bots get the full system set (their gates unchanged); old saves open what they have used.
- Gate: tests; ui.mjs (a fresh lineage's day-0 surfaces; each trigger opens its system); the per-day
  system counts reproduce `proposal.json`'s `systems_by_day` within ±1.

### 3. Diagnostics meters (core + client)
- **Core stats** per fight, per run, per night: damage dealt/taken per second by attacker, foe and pet;
  healing per second by source; time split (fighting · travel · chores · rest); per-rule fires and share of
  actions; supplies used; gold per minute; hits taken by hero vs pets. Pure reads of the event stream
  (determinism unchanged).
- **Client**: a toggleable compact meter on the watch; the fight's breakdown on the death screen; per run
  and per night on the report; a two-run comparison in camp. Copy per `docs/COPY.md` (units always; rules
  named by their words).
- Gate: qa.rs (meter totals equal the event stream's sums); ui.mjs.

### 4. A tap is a decision (core + client)
- The keep sheet appears only when a find beats something in the vault; otherwise it settles by
  preference with one line (`kept leather +1`).
- `quartermaster` behaviour free by default; insuring automatic when the purse covers it.
- The restock adds a kind a rule references (`+ fire · for throw fire`).
- Vault preference, cage, start, repeat, insure consolidated into one standing-orders panel.
- Rule editing is fast: drag or long-press to reorder any distance; a chip change shows its first pass
  ≤ 1 s (AW: one ▲ per tap, 5–12 s waits).
- Gate: a scripted player replaying the cohort rule sets averages ≤ 1 non-decision tap per send; QA tap
  logs measured the same way.

### 5. Desktop and mobile IA (client) — if not already delivered by the gfx/ui eval agent
- ≥ 1024 px: rules | well | forecast + meters, the console across the bottom. Mobile: the console always
  visible; nothing covers the primary gem.

### 6. Seams (cohort 24)
- A companion's death is a named beat and a report line (AX: Greth the tamed ogre, L5, gone with only
  `party −1 ogre`); a tamed foe's grudge closes as `tamed`, never `avenged`.
- Pack 4 bought → supplies 4/4 (AX saw 3/3); a D12 bank of $81 against a ~$260 forecast (AX) — find why.
- The watch survives a WebGL context loss (AW: 7 minutes frozen) — owned by the gfx/ui eval agent.

## Gates

| Gate | Bar |
|---|---|
| Unlock days ≥ 10/14; marks unspent ≤ 8; stall ≤ 3 (hard, dayplayer + rater sets); purse ≤ 1.5 days' net | metrics (progression rows) |
| Systems open per the §6 table; bots full-set | tests, ui.mjs |
| Meter totals equal the event sums | qa.rs, ui.mjs |
| ≤ 1 non-decision tap per send | scripted player, QA logs |
| Lever, bots, dice, stalls, dances, lanes, divergence, oaths, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 25 (three absences): mean ≥ 76; pacing or progression 0.8 from both; α ≥ 0.80 | two blind cards |
