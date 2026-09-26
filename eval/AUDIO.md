# Audio — the owner's listening pass

Agent raters cannot hear (`eval/RATING.md`: audio is "unassessed" unless a human note is attached). This is the one pass a person
does with headphones, ~15 minutes, after a sound change. The objective numbers (loudness, peaks, clipping, spectral spread, jitter)
are in `docs/JUICE.md` §6; this checklist covers what numbers can't: does it sound good, does it tire you, does it say the right thing.

Dev build (`tools/dev.sh`, `http://localhost:5219/`), unmuted (settings → mute off). Sound starts after your first tap.

## 1. Audition every cue (2 min)

In the browser console, on any screen after one tap:

```js
const q = [["strike",{kind:"goblin"}],["strike",{kind:"skeleton"}],["strike",{kind:"bloat"}],["strike",{kind:"iron_golem"}],["strike",{kind:"wraith"}],
  ["hit",{dmg:3,kind:"goblin"}],["hit",{dmg:20,kind:"ogre"}],["slay",{kind:"goblin"}],["slay",{kind:"skeleton"}],["slay",{kind:"bloat"}],
  ["slay",{kind:"iron_golem"}],["slay",{kind:"wraith"}],["rule"],["telegraph"],["boss_in"],["boss_break"],["boss_down"],["level"],["unlock"],
  ["buy"],["click"],["edit"],["verdict"],["fold"],["scene"],["scene_end",{up:true}],["scene_end",{up:false}],["exit_bank"],["exit_return"],["exit_death"]];
q.forEach(([n, o], i) => setTimeout(() => { console.log(n, o ?? ""); __audio.cue(n, o); }, i * 900));
```

Then the same strike ten times (`for (let i = 0; i < 10; i++) setTimeout(() => __audio.cue("strike", {kind: "goblin"}), i * 250)`) — it
should sound like ten blows, not one sample looped.

## 2. Listen in place (10 min)

| Where | How to get there | Listen for |
|---|---|---|
| Camp | `?seed=3001&fresh=1` | the pad under the camp (quiet, not a hum you notice); tile clicks and chip taps (soft, not clicky-harsh); a purchase (two coin notes); an edit's divergence scene (a swoosh per branch, a ring on each end: the "lives" ring resolves, the "dies" ring doesn't) |
| Watch, each biome | send; `?speed=1`; the Burrows D5–8, the Fens D9–13, the Crypt D14+ (a deep lineage: `?absent=8h`) | the ambience bed: is each biome a *place* (Warrens drips, Burrows scratching, Fens bubbles, Crypt wind and a far bell, Foundry clanks and crackle, the Deep's sub and slow drips, the Sanctum's chimes) — audible when fights are quiet, gone under them |
| Watch, a fight at 1× | any fight frame | the hero's blows by what they hit (goblin flesh thud · skeleton clack · bloat squelch · golem clang · wraith hush); the blows he takes (a low thud that deepens with damage); the kill; the rule tick; after two minutes: fatigue? |
| A boss | the first boss floor (the goblin warlord) on a set that reaches it | the entrance (a sub drop), the break (a steel crack), the fall (a boom that falls away) — do they feel bigger than a normal kill, without being loud |
| A fold | a lineage whose shallow floors are solved (`?absent=8h`, send) | the fold line's soft whoosh-and-chime |
| Death | a set that dies | the verdict's stamp as the word slams; the low death note |

## 3. Verdicts (fill in, one line each)

| Question | Verdict (good / fix / bad) | Note |
|---|---|---|
| Nothing clips, crackles or pops (cue onsets, bed loops, stacked hits) | | |
| Levels: fights clearly over the bed; clicks under fights; the boss moments the loudest | | |
| The five families are tellable apart with eyes closed | | |
| No repetition fatigue after 2 min of a fight at 1× | | |
| The biomes sound like different places | | |
| The chrome sounds (click, edit, buy) are pleasant, not fussy | | |
| The Cut 27 sounds (fold, scene, scene end) fit the moment | | |
| Mute silences everything, and stays muted after a reload | | |

Raters may cite this file's filled verdicts as the audio evidence (`eval/RATING.md`).
