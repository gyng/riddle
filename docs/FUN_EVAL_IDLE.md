# Fun evaluation for idle games — a specialisation of the Fun Framework v4

*Written 2026-09-15 for Riddle. Extends `../eval-fun/docs/FRAMEWORK.md` (read that first;
nothing there is repeated here unless it changes). Data: `eval/presets.json`. Template:
`eval/SCORECARD.template.json`. Scorer: `eval/score.sh` (wraps eval-fun's scorer).
Research inputs: `research/idle-attraction.md`, `research/comparables.md`,
`research/roguelike-and-spectating.md`.*

## 0. Why the universal rubric is not enough for idle games

The v4 framework evaluates a *played session*. An idle game is mostly *not played*: the
unit of experience is the **check-in** (arrive, read what happened, decide, leave) and the
**absence** between check-ins. Three things the universal protocol cannot see:

1. **The absence is content.** What the player comes back to is the game's main
   deliverable. No universal axis scores it. The idler preset in eval-fun leans on
   `progression` and `pacing`, which is how Cookie Clicker scores 74.7 as "fun" while
   everyone agrees it is compulsive rather than fun. The rubric needs to separate *what
   you return to* from *what grew*.
2. **Someone else is playing.** In an auto-played game the agent acts and the player
   configures. Whether the player experiences the agent's wins and deaths as *their own*
   is the whole difference between "I played" and "I watched a spreadsheet". The
   universal `expression` axis (is the build yours) and `failure` axis (does the loss name
   its cause) both touch this but neither asks the question.
3. **Shape Gacha's failure mode.** The previous idle attempt scored well on progression
   hallmarks (two currencies, offline catch-up, prestige, multipliers) and was still not
   fun, because it had no engine axis: tension 0, decisions were a maths puzzle with no
   stakes, nothing produced a story. The v4 rule "progression can never rescue a weak core"
   was correct; the idle-specific rubric must make the *core* of an idle game explicit
   rather than let the retention layer carry the score.

So this specialisation keeps the fourteen axes, their anchors, gates, kill criteria and
reliability rules, and adds: **two idle axes**, **idle boundaries on five existing axes**,
**three idle kill criteria**, **an absence-aware protocol**, and **idle bot proxies**.

## 1. The two idle axes

| # | Axis (id) | Layer | Definition | Rests on | Bot proxy (cap) | Human probe |
|---|---|---|---|---|---|---|
| 15 | Return payoff (`return`) | idle-core | Every absence produces something worth reading **and** a decision worth making; the payoff scales with absence in a way the player can predict, and active play is rewarded without absence being punished | Pecorella's active/idle ratio; "a present when you return"; Zeigarnik; Loop Hero's expedition summary | offline yield ÷ active yield per hour (band 0.25–0.5); decisions pending at return ≥ 1; time-to-first-meaningful-action after return < 30 s | "What did you come back to? What did you decide because of it? Would you have rather stayed?" |
| 16 | Attribution (`attribution`) | idle-core | The player describes the agent's play in the first person; wins are credited to their configuration and losses are traced to a rule they wrote or did not write, and they can say which | Meier (players credit wins, blame the game for losses); Dragon's Dogma pawn inclinations; FF12 gambit discovery; B&W creature-training failures | share of deaths whose trace ends in a player-authored rule (≥ 70%); share of clears where a non-default rule fired on the decisive turn; default-rules-vs-edited win-rate gap | "Tell me how the last run ended. Whose decision was that?" Count first-person pronouns. |

Boundaries (no double counting):
- `return` counts the *content and decision* of a check-in. The amount that grew is
  `progression`; whether the check-in was quick is `pacing`.
- `attribution` counts the player's *ownership*. That the rule set is recognisably theirs
  is `expression`; that the death names a cause is `failure`. A death can name its cause
  perfectly ("killed by an ogre at 3 HP") and still score 0 on attribution if the player
  cannot connect it to anything they configured.
- A pure incremental with no agent scores `attribution` at weight 0 (see `idler` preset).

## 2. Idle boundaries on existing axes

| Axis | Count here in an idle game | Do not substitute |
|---|---|---|
| `decisions` | Configuration choices with foreseeable trade-offs made *at check-in*: which rule to add, which upgrade, which risk to carry. A decision gap exists only if two configurations are both defensible across several runs | "Buy the cheapest thing" sequences; upgrades that are strictly ordered; edits the player makes because the UI nagged |
| `tension` | Stakes the player chose to carry into an absence or a watched run (loot at risk, a rule that trades safety for depth), resolved when they look | Number-go-up anticipation; timers; watched fights whose outcome cannot change anything the player owns |
| `pacing` | Report three loops separately: **in-run** (events per minute when watching at 1x, dead time), **check-in** (arrive → read → decide → leave; should be bounded, 2–10 minutes, and end on a chosen stopping point), **cadence** (do absences of 20 min, 4 h and overnight each produce a distinct experience). Judge the whole | A fast check-in hiding that watching is dead; a rich watch hiding that check-ins are chores |
| `failure` | The agent's death is legible *to the configurer*: cause, margin, and the rule (or gap) that produced it, in one screen. Recovery is cheap: the next run starts on its own | A death message alone. "Killed by X" without "because rule N fired / no rule matched" is at most 0.3 in an auto-played game |
| `progression` | Declare all three layers: within-run growth (kit, floor), between-run persistent growth (unlocks, vocabulary, slots), and **automation growth** (things the player no longer needs to do). Automation-as-reward is a genre hallmark; record its cadence | Prestige as a flat multiplier; unlocks that add numbers but no new decisions or vocabulary |
| `surprise` | Situations in the *chronicle* or the watched run that changed what the player configured | Random number variance in yields; cosmetic events |

## 3. Idle kill criteria (in addition to the four universal ones)

Any one fails the build regardless of score:

1. **Editing is pointless.** The shipped default configuration clears/earns as much as any
   configuration a tester writes in an hour. (Proxy: default-vs-edited win-rate gap < 5
   points over 30 seeds.)
2. **Unfair deaths above 10%.** More than one death in ten is caused by something no
   available rule vocabulary could have addressed. (Proxy: death-cause audit.)
3. **Empty return.** An overnight absence returns the player to a screen with nothing new
   to read and nothing to decide. (Probe: the `return` human probe answered "nothing".)

`darkPatternInLoop` in the universal list is read strictly for idle: any wait the game
created that can be skipped for money, and any absence that *punishes* (decay, loss of
banked gains) beyond stakes the player explicitly chose.

## 4. Protocol changes: absence is part of the horizon

The universal tiers assume continuous play. For idle games the **evidence horizon must
state the absences**, and the rater must actually take them.

- **Tier 1 — one hour plus one absence.** 40 minutes cold, then leave for ≥ 3 hours
  without the app open, then 20 minutes on return. Answer the sixteen probes after the
  return, not before. Screenshot: first minute, first agent death, the return screen, the
  first edit made after returning.
- **Tier 2 — three days, natural cadence.** Precommit the contract (build, save, tier,
  absences: at least one of ≤ 30 min, one of ~4 h, one overnight). Play as the target player
  would: check in when you want to, log every check-in (arrival time, minutes spent, what
  you read, what you decided, why you left). Reach at least five agent deaths and one
  complete unit (a clear, or the declared long-goal cadence). Carry at least one earned
  persistent gain across an absence. Score provisionally before telemetry. Next-day
  delayed recall: retell the best run without notes; if it has no turn in it, `story` ≤
  0.3. Then run the bot proxies as caps.
- **Tier 3 — one week, five players.** As universal, plus a **check-in diary** per player
  and the extra criterion probe "did you open it today without meaning to?" (1–7).

Anti-bias moves specific to idle: do not score `return` on the same day as the absence
began; record whether you *wanted* to come back before you saw what was there; count
pronouns in your own run retelling before awarding `attribution`.

## 5. Bot proxies that are specific to idle (caps only)

All are per-engine work; Riddle's sim exposes them from Rust (`PLAN.md` §9). Each is a
cap: green at or above bar, amber near, red below. None awards.

| Proxy | Axis capped | Bar (initial, revise after calibration) |
|---|---|---|
| Offline yield ÷ active yield per hour | return | 0.25–0.5 green; > 0.8 amber (absence dominates); < 0.15 red |
| Decisions pending at return (rules to edit, unlocks affordable, deaths to read) | return | ≥ 1 green after any absence ≥ 20 min |
| Time from return to first meaningful action | return, pacing | < 30 s green |
| Default-vs-edited win-rate gap (30 seeds, 1 h of edits) | attribution, decisions | ≥ 15 pts green; 5–15 amber; < 5 red (kill) |
| Share of deaths whose trace ends in a player-authored rule | attribution, failure | ≥ 70% green |
| Unfair-death share (no vocabulary could address) | failure | ≤ 5% green; ≤ 10% amber; > 10% red (kill) |
| Death-cause entropy over 100 runs | surprise | top cause < 35% green |
| Rule-set diversity across clears (distinct rule multisets ÷ clears) | expression | ≥ 0.5 green |
| Rules-per-death trend across sessions | mastery | declining or plateau after rising = green (learning then stability) |
| Events per minute when watching at 1x | feel, pacing | ≥ 6 green; dead stretches > 20 s red |
| Check-in length distribution (Tier 2 diary) | pacing | median 2–10 min green |
| Automation cadence (new "no longer need to" per hour of active play) | progression | ≥ 1 per 2 h in the first 10 h |

## 6. Calibration for the idle presets

Score these on the idle presets before trusting a Riddle score (cards in
`eval/calibration/`): **Cookie Clicker** (`idler`), **Melvor Idle** (`idle-rpg`),
**Loop Hero** (`idle-roguelike`; the nearest existing thing), **Progress Quest**
(`idle-rpg`; the reference "watch only" dud-that-is-a-joke), and a **mediocre idle RPG
composite** (`idle-rpg`). Pass condition as universal: Loop Hero should land in *fun* or
above, Progress Quest and the composite in *not yet*, Cookie Clicker where eval-fun put it
(fun, low 70s). If Loop Hero is gated, the axis or weight is wrong. Re-anchor on Loop Hero
in the same sitting whenever the rubric is used after a gap.

## 7. Using it on Riddle

1. `cp eval/SCORECARD.template.json eval/cards/<build>.<rater>.json`, fill the horizon
   block honestly (absences taken, runs watched, runs offline, rule edits).
2. Answer the criterion and the sixteen probes in writing, then award on the anchors.
3. `eval/score.sh eval/cards/<build>.<rater>.json [--profile=achievement]`.
4. Run the proxies (`make probes` once the sim exports them) and set caps.
5. Fix the lowest-weighted-award gated axis first. `return` and `attribution` gates are
   the ones that say "this is an idle game that is not yet a game".
6. Reliability as universal: second blind card on the same immutable build and the same
   absences before any change is believed.
