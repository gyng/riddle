# Roguelike story generation + spectating/raising an AI hero

Research brief for an auto-played roguelike (procedural dungeon, permadeath, items, distinct monsters; hero runs on player-authored gambits; idle pacing; camera follows hero; metagame unlocks). Two questions: **(A)** what a roguelike needs to be a good story generator when no human makes the moves; **(B)** what makes watching / raising / teaching an agent fun. Every finding ends with a **So what** line.

Sourcing note: web search budget was exhausted early in this session; findings rest on fetched primary pages (cited inline) plus well-known talks/essays cited by venue. Confidence marked where I am working from recollection rather than a fetched page. Sites that refused fetch (RPS, Game Developer, roguebasin, fandom, crawl wiki) are cited by title only.

---

## Part A — Roguelike design

### A1. Berlin Interpretation: what is load-bearing today

The 2008 Berlin Interpretation lists nine high-value factors (random generation, permadeath, turn-based, grid, non-modal, complexity/emergence, resource management, hack-and-slash, exploration/discovery incl. unidentified items) and five low-value ones (single character, monsters behave like players, tactical challenge, rooms-and-corridors, numeric status) ([Wikipedia: Roguelike](https://en.wikipedia.org/wiki/Roguelike)). Twenty years of practice has sorted these differently from how Berlin ranked them:

| Factor | Community verdict | Evidence |
|---|---|---|
| Permadeath | Load-bearing, but as *stakes*, not as *punishment*. Non-permadeath modes are now common (Tangledeep "Adventure mode", Andrew Aversa's RLC 2020 talk "The End of Permadeath"). | [RLC 2020 program](https://roguelike.club/event2020.html); Josh Ge notes Cogmind deliberately makes the PC "quite resilient... even if naked and item-less" so death is a *stretch* not a *snap* ([Designing for Mastery](https://www.gridsagegames.com/blog/2025/08/designing-for-mastery-in-roguelikes-w-roguelike-radio/)). |
| Procedural generation | Load-bearing only along gameplay-relevant axes. Brian Walker's RLC 2016 talk ("Proc gen and gameplay") asks "how can each play... feel distinct" to veterans and answers: variance in *items and situations*, not map shapes. | [RLC 2016 program](https://roguelike.club/event.html); talk on the [RLC YouTube channel](https://www.youtube.com/@roguelikecelebration) (recollection, confidence medium-high). |
| Turn-based | Vestigial as a *format*; what matters is that every state is inspectable and every consequence attributable (Jupiter Hell, Rift Wizard keep turns but strip waiting). | [Rift Wizard Steam page](https://store.steampowered.com/app/1271280/Rift_Wizard/): "low randomness, high player agency". |
| Resource management / hunger | Hunger as a *resource* is dead: DCSS removed food outright in 0.26 (2021); RLC 2020 had a whole talk on the "evolution of roguelike hunger mechanics" (Chapman). Hunger as a *clock* survives in other clothes: DCSS's Zot clock (0.27), Cogmind's alert level, Brogue's guaranteed-schedule food. | [RLC 2020 program](https://roguelike.club/event2020.html); Ge: clocks exist to "prod the player along and impose a cost on spending too long in one area" ([source](https://www.gridsagegames.com/blog/2025/08/designing-for-mastery-in-roguelikes-w-roguelike-radio/)). DCSS 0.26/0.27 facts from recollection, confidence high. |
| Emergent complexity | The single most-cited source of stories: NetHack's TDTTOE culture ("the DevTeam thinks of everything"), cockatrice-corpse-with-gloves as canonical example. | [Wikipedia: NetHack](https://en.wikipedia.org/wiki/NetHack); [Wikipedia: Roguelike](https://en.wikipedia.org/wiki/Roguelike). |
| Identification | Still valued, but only when the gamble is *interesting* (Brogue's detect-magic "benevolent/malevolent" hint; DCSS's price-ID). Pure guess-work was pruned. | DCSS manual philosophy ([crawl_manual.rst](https://raw.githubusercontent.com/crawl/crawl/master/crawl-ref/docs/crawl_manual.rst)). |
| Grid, ASCII, single character, rooms-and-corridors | Vestigial. | Berlin itself rated most of these low. |

**So what for an auto-played roguelike:** Keep permadeath-as-stakes, item-driven run variance, a forward-pushing clock, and a small set of deeply interacting systems; drop everything that exists only to give a human something to fiddle with (hunger accounting, ASCII purism, rooms-and-corridors dogma).

### A2. Designers on what generates anecdotes vs grind

**Brian Walker, Brogue.** Brogue has no XP: the character is entirely what the dungeon gave it (enchant scrolls, potions of life/strength, allies). The dungeon is an ecosystem: gas, fire, deep water that sweeps items, chasms, bog, brimstone; monsters that interact with it (bloats pop into caustic gas; pink jellies split; monkeys steal and flee; goblin conjurers summon; ogres shamans buff; captives can be freed as allies who level with you). Monsters telegraph intent and flee at low HP; stair-dancing is defused by monsters following through stairs. Vaults present a *choose one of N* cage. Every consumable has a throw/dual use (incineration, darkness, creeping death, confusion). Walker's RLC 2016 thesis: distinctness comes from *situation variety*, so build the item and monster sets for interaction density, not size. (Talk on [RLC YouTube](https://www.youtube.com/@roguelikecelebration); wiki confirms only the "simplicity and beauty" reception — [Wikipedia: Brogue](https://en.wikipedia.org/wiki/Brogue_(video_game)). Detail from recollection of the game and talk, confidence high.)

**Tarn Adams, Dwarf Fortress.** "Losing is fun" began as "a throw-away joke from the game manual" meant "to create comfort with the concept of permadeath"; the game is explicitly an "open-ended story generator"; "one of the things we set out to do is to get people to write these narratives about their game" ([Wikipedia: Dwarf Fortress](https://en.wikipedia.org/wiki/Dwarf_Fortress)). The wiki's own gloss: conservative, non-losing play "tends to become very boring" ([DF wiki: Losing](https://dwarffortresswiki.org/index.php/DF2014:Losing)). Legends mode records every historical figure, site and event and exports XML so players can *reconstruct* what happened after the fact ([DF wiki: Legends](https://dwarffortresswiki.org/index.php/DF2014:Legends)).

**Josh Ge, Cogmind.** Legibility by *telegraphing*: a surveillance bot that alerts unseen allies "shows an animation and message to that effect" so the player can respond. Stories come from forced adaptation: "roguelike design is at its strongest when players are forced to adapt... MacGyver your way out of awkward or deadly situations"; "Rebuilding is a frequent topic of discussion in the community, and a noticeable way to get one of those 'gaming highs' in the middle of a run." Balance is read off the community, not the dev inbox: "what players say or show to one another is more important than what they say directly to a developer"; "'it depends' is an incredibly common refrain" = healthy balance. Cogmind's post-run scoresheets (hundreds of stats, uploaded to leaderboards) back this with data ([Designing for Mastery](https://www.gridsagegames.com/blog/2025/08/designing-for-mastery-in-roguelikes-w-roguelike-radio/); scoresheet detail from recollection, confidence medium-high).

**DCSS.** The manual's philosophy section: "challenging and random gameplay, with skill making a real difference"; "meaningful decisions (no no-brainers)"; "avoidance of grinding" (grind = "low risk, take a lot of time, and bring some reward"); no-brainers are "a horrible lost opportunity for fun"; and "all tedious, but necessary, chores should be automated" — autoexplore `o`, autotravel `G`, autofight `Tab` ([crawl_manual.rst](https://raw.githubusercontent.com/crawl/crawl/master/crawl-ref/docs/crawl_manual.rst)). How much of DCSS is already automated: in practice a run is `o`/`Tab` until the game *interrupts*. The interrupt list is the developers' opinion of where a human is needed: autoexplore stops on a monster coming into view, an item, an altar/shop/portal/branch stair, HP or status change; autofight refuses below an HP threshold (`autofight_stop`). The community meme "DCSS is an o-Tab simulator" is answered by bhauth's RLC 2020 talk "What makes DCSS a good game": the game is about what you do when the automation *stops* (recollection, confidence medium-high). The DCSS bot **qw** is "the first bot to win DCSS with no human assistance"; it is a rule/goal list in an rcfile ("D:1-11, Lair, D:12-D:15, Orc"), has won 3- and 15-rune games, "does not have a high winrate", leaves most spells unused, and is spectated on public servers with an `action delay` option built in for watchers ([github.com/crawl/qw](https://github.com/crawl/qw)).

**Other rule-based agents that prove the premise.** Rog-O-Matic (CMU 1981–83) had "a higher median score than any of the 15 top Rogue players at Carnegie-Mellon" and won; its architecture was a priority-ordered set of "experts" (melee, ranged, retreat, explore) plus persistent cross-game monster memory — a gambit list with a metagame ([Wikipedia: Rog-O-Matic](https://en.wikipedia.org/wiki/Rog-O-Matic); architecture from the paper, recollection, confidence medium). BotHack ascended NetHack in 2015; the hardest parts were item identification (needed logic programming over prices/observations) and combat/inventory heuristics ([github.com/krajj7/BotHack](https://github.com/krajj7/BotHack)). Angband ships a Borg that people run as a screensaver (recollection, confidence medium).

**Shattered Pixel Dungeon.** Small, legible system set (5 hero classes, item ID by use with `?`-marked unknowns, a handful of gas/fire/water/grass interactions) beats big NetHack-style lists for a mobile audience; Evan Debenham's RLC 2021 talk was on community-driven development ([RLC 2021 program](https://roguelike.club/event2021.html)). Confidence medium (blog unreachable).

**Caves of Qud.** "Deeply simulated physical and political systems" and procedural history "based on historical accounts such as word of mouth and ancient texts, allowing for bias and conflicting perspectives" ([Wikipedia: Caves of Qud](https://en.wikipedia.org/wiki/Caves_of_Qud)); Grinblat's RLC 2016 talk generated in-game lore books with Markov chains ([RLC 2016](https://roguelike.club/event.html)). The lesson is that *retold* history (biased, partial) reads as story where raw event logs do not.

**Rift Wizard / Jupiter Hell.** Rift Wizard strips randomness from combat so every death is a build/tactics error you can read ([Steam](https://store.steampowered.com/app/1271280/Rift_Wizard/)). Jupiter Hell keeps turns but compresses them to real-time-feeling action with action queuing and cover ([Wikipedia](https://en.wikipedia.org/wiki/Jupiter_Hell)).

**Which systems create anecdotes vs grind (synthesis):**

| Creates anecdotes | Only adds grind |
|---|---|
| Monster–monster and monster–terrain interactions (Brogue ecosystem, NetHack) | Large monster lists that differ only by numbers |
| Dual-use consumables (throwable potions, read-vs-keep scrolls) | Consumables that are only stat sticks |
| Allies that can die, betray, level, be sacrificed | Allies as DPS with no state |
| Identification gambles with partial info (benevolent/malevolent) | Blind ID of dozens of item flavours |
| Environmental hazards that hit *both* sides (gas, water, fire, chasms) | Damage-floor tiles |
| Clocks (Zot clock, alert level) that force diving | Hunger bookkeeping |
| Vault "choose one" cages, altar/god commitments | Shops that buy items (DCSS removed this) |
| Near-death recovery (Cogmind naked core; Brogue flight) | Instant-death traps with no tell |

**So what for an auto-played roguelike:** Build ~30 monsters and ~40 items chosen for *interaction density* with a handful of terrain hazards; the hero's rules will then produce different stories from the same rule set. Grind systems are doubly bad here — the hero grinds them silently and the watcher sees nothing.

### A3. The human's decisions in a classic roguelike, and where they belong

Criterion: Sid Meier's "interesting decision" (trade-off, situational, personal — GDC 2012 "Interesting Decisions", [GDC Vault](https://www.gdcvault.com/play/1015756/Interesting-Decisions), recollection, confidence high) is the mirror image of DCSS's "no-brainer" rule. **If a decision has a dominant answer, automate it silently (DCSS). If it is a trade-off that expresses a play style, it belongs in the gambit layer.**

| Decision | Rule-able? | Interesting? | Placement |
|---|---|---|---|
| Fight vs retreat | Yes — qw uses HP%, enemy count, "danger" estimate, distance to stairs | Yes: thresholds *and* exceptions are personal ("never flee from X", "always fight when cornered") | **Gambit layer** (core) |
| Corridor vs open ground | Yes ("if ≥2 melee adjacent, back into corridor"); qw does it | Medium: one toggleable tactic | Gambit, as a named tactic card |
| Use consumable now vs save | Yes with thresholds; but scarcity makes the threshold the whole game | Yes — the classic roguelike decision | **Gambit layer** (core) |
| Identify-by-use | Yes ("read unknown scrolls when safe, HP full, no monsters"); qw and BotHack both do this | Medium — the gamble itself is fun to *watch* | Gambit as policy dial (cautious ↔ reckless), execution automated |
| Rest | Trivial | No | Automate silently; couple to the clock |
| Dive vs explore | Yes (qw goal sequences; "descend on finding stairs" vs "clear level") | Yes — strategic, personal, big story lever (under-leveled dives = tension) | **Gambit layer** (strategic tier) |
| Target priority | Native to FF12 gambits | Medium ("casters first", "lowest HP first") | Gambit |
| Equipment swaps | Yes by score; interesting only when trading resist vs damage | Low–medium | Auto-equip with a preference dial |
| Escape items (teleport, blink, digging) | Yes ("HP<20% and no path to stairs") | Yes — the highest-drama moment | **Gambit layer** (panic rules) |
| Ally handling (free captive? sacrifice? leave behind?) | Yes with a stance | Yes — moral/personal | Gambit stance |
| Throw vs drink, use terrain (door, water) | Yes | Yes — the "clever play" the watcher wants to see | Gambit tactic cards, unlocked over the metagame |
| Vault/altar/god choice | Yes by preference | Yes (Brogue's "1 of N") | Gambit preference or rare **direct interrupt** |

Bots' documented gaps are your design budget: qw leaves most spells and abilities unused; BotHack's hardest problem was item ID; both are weak at *improvised* play. That is exactly the space the player's rules should fill, and exactly where the watcher will see the hero look smart or dumb.

**So what for an auto-played roguelike:** The gambit editor should expose ~8 decision families (retreat, consumable thresholds, escape, dive/explore, targeting, ally stance, ID policy, terrain tactics) and hide the rest. Every death should trace back to one of those eight.

### A4. Death: legible and productive

**Making death legible (classic roguelikes).**
- NetHack: tombstone with "killed by ...", DYWYPI inventory dump, dumplogs (identified inventory, kills, conducts, dungeon overview, last messages) that players paste into YASD/YAAD threads; the community even sorts deaths into *stupid* (pilot error) and *annoying* (unavoidable), and "Such deaths are considered part of learning to play" ([Dumplog](https://nethackwiki.com/wiki/Dumplog); [YASD](https://nethackwiki.com/wiki/Yet_Another_Stupid_Death); [Tombstone](https://nethackwiki.com/wiki/Tombstone); [Wikipedia: NetHack](https://en.wikipedia.org/wiki/NetHack)). Bones files let your corpse haunt a later game.
- DCSS: morgue file = death message + *auto-generated notes timeline* ("Reached XP level 5", "Entered Lair", "Killed X") + message log + action counts. The notes are a highlight reel the game writes for you (crawl wiki unreachable; recollection, confidence high).
- Cogmind: scoresheet with hundreds of stats, uploaded to leaderboards, plus a per-run "history" of depth/route (recollection, confidence medium-high).
- Brogue: one-line epitaph "Killed by a goblin conjurer on depth 4 with 120 gold", seed shown so the run is replayable; monsters telegraph, so the player can usually name the mistake.

**Making death productive (roguelites).**
- Rogue Legacy: death → choose one of three heirs with visible traits; gold spent persists; design goal "relatively forgiving and accessible, while also allowing permanent progression"; critics: "[rides] the line of frustration and fun" ([Wikipedia](https://en.wikipedia.org/wiki/Rogue_Legacy)).
- Hades: death is a scene change; ~10 hours of dialogue gated on run events; roguelike structure "allowed them to tell these branching stories... over the course of multiple playthroughs" ([Wikipedia](https://en.wikipedia.org/wiki/Hades_(video_game))).
- Dead Cells: cells bank only at section exits — "if the player dies before then, they lose all collected Cells"; blueprints must be extracted; Motion Twin wanted permadeath to feel "rewarding" ([Wikipedia](https://en.wikipedia.org/wiki/Dead_Cells)).
- Loop Hero: the hero auto-walks and auto-fights; retreat keeps resources, death loses 70%; a critic was "hooked on an RPG that plays itself" ([Wikipedia](https://en.wikipedia.org/wiki/Loop_Hero)). Retreat is a *decision about when to stop watching* — the exact lever an idle game has.
- Dungeon of the Endless: heroes act "autonomously" between waves and permadie; the crystal's Dust is the persistent stake ([Wikipedia](https://en.wikipedia.org/wiki/Dungeon_of_the_Endless)).
- Darkest Dungeon / XCOM: graveyard and memorial wall list each dead hero with cause and deeds (recollection, confidence high).

**Patterns for a death you did not personally cause to feel fair and interesting:**
1. *Rule attribution*: name the rule that fired (or failed to fire) in the last N turns — "Retreat rule did not trigger: HP 41% > threshold 40%". Sid Meier's GDC 2010 "Psychology of Game Design" ([GDC Vault](https://www.gdcvault.com/play/1012186/The-Psychology-of-Game)) documents that players blame the game for losses at 3:1 odds; you defuse this by making the cause a *rule the player wrote* (recollection, confidence high).
2. *Telegraph before doom*: Brogue/Cogmind-style intent display so the watcher saw it coming before the hero did.
3. *YASD/YAAD split*: classify each death as "rule gap" (fixable) vs "dice" (RNG); show the split on the lineage screen so the player sees their rule set improving.
4. *Auto-notes timeline*: a DCSS-style chronicle written during the run so the death has a story before it.
5. *Persistent trace*: bones, graveyard entries, heir traits, seeds — the run leaves residue.
6. *Bank on exit, not on death* (Dead Cells): create a retreat decision so the player chooses risk.

**So what for an auto-played roguelike:** Death is the main authored moment. Ship a death screen that shows the last five rule firings, the telegraphs that preceded them, a YASD/YAAD verdict, and a one-click "patch this rule" button.

---

## Part B — Spectating and raising an AI

### B5. Why watching an agent is compelling

**Tynan Sylvester (RimWorld).** "The whole value of a game is in the mental model of itself it projects into the player's mind"; "anything in the Game Model that doesn't copy into the Player Model is worthless"; players supply emotion via *apophenia* — Sims players ascribe jealousy and grief the sim never models. Story-richness needs (1) minimum representation, (2) primal values at stake ("life/death, alone/together, wealth/poverty"), (3) simple, intelligible systems, not a "gigantic hairball" ([The Simulation Dream](https://tynansylvester.com/2013/06/the-simulation-dream/)). RimWorld is "a story generator" run by storytellers: Cassandra (rising/falling tension), Phoebe (more downtime), Randy (random) — pacing is an *authored* layer over the simulation ([Wikipedia: RimWorld](https://en.wikipedia.org/wiki/RimWorld)).

**Dwarf Fortress.** Stories are retold *after* the fact from Legends (see A2); Boatmurdered spread because it was a *narrated* play-by-post, not a log. Graham Smith: results are "often hilarious, occasionally tragic, and always surprising" ([Wikipedia: DF](https://en.wikipedia.org/wiki/Dwarf_Fortress)).

**The Sims.** Autonomy ("free will") plus needs the player can read; scholars treat Sims stories as "a variety of writing akin to scrap-booking" ([Wikipedia: The Sims](https://en.wikipedia.org/wiki/The_Sims)) — players narrate what the agent did.

**Godville / Progress Quest.** Godville: zero-player; the hero writes a diary, "occasionally needs a sign of the god's existence"; player influence is "rewards and punishments, and sometimes direct communication"; a reviewer compared checking it to "checking his email or Twitter feed"; players enjoy "witnessing the hero's personality develop independently" ([Wikipedia: Godville](https://en.wikipedia.org/wiki/Godville)). Progress Quest: players persevere "owing to an emotional attachment with the character"; credited by Todd Howard as the inspiration for Fallout Shelter ([Wikipedia: Progress Quest](https://en.wikipedia.org/wiki/Progress_Quest)). Both work because the *voice* of the hero carries the absurdity — the narration is the product.

**Twitch Plays Pokémon.** 1.17M participants, 120k peak concurrent, 55M views; the community invented lore (Helix Fossil, Bird Jesus, False Prophet) out of noise; Anarchy vs Democracy became an ideological fight ([Wikipedia](https://en.wikipedia.org/wiki/Twitch_Plays_Pok%C3%A9mon)). Lesson: watchers manufacture meaning from *visible struggle* and *named recurring characters*.

**Blaseball.** A text-only baseball sim, weekly seasons, betting with fake coins, votes on rule changes, players "incinerated" by rogue umpires; fans built art, sportscasters, unions; a stats community (SIBR) gave an RLC 2021 talk on "why data accessibility matters" ([Wikipedia: Blaseball](https://en.wikipedia.org/wiki/Blaseball); [RLC 2021](https://roguelike.club/event2021.html)). Lesson: expose stats and let fans narrate; give the audience *votes* that visibly warp the sim.

**Marble races.** Jelle's Marble Runs: sports commentary "as though they were athletes", named teams (Savage Speeders, O'rangers), fan loyalty, 204M views ([Wikipedia](https://en.wikipedia.org/wiki/Jelle%27s_Marble_Runs)). Lesson: commentary + persistent named competitors turn physics into drama.

**SaltyBet / AI-vs-AI.** MUGEN AI fights with fake-money betting; fights are watchable because the bet gives stakes and the roster has recurring characters with known quirks (recollection, confidence medium). Claude Plays Pokémon (2025): viewers watched the model's visible reasoning more than the game; being stuck in Mt. Moon for days and inventing a "faint to teleport" strategy became the story; viewers were endeared *and* frustrated by dumb mistakes (recollection, confidence medium).

**Football Manager.** The manager never touches the ball; match output is presented as *highlights* (key/extended/full) with commentary and a post-match analysis of individual player actions; 19M players for FM24 ([Wikipedia](https://en.wikipedia.org/wiki/Football_Manager)). Lesson: the pacing control ("key highlights only") is *the* idle-spectator UI pattern.

**Auto battlers.** Prep phase → units "automatically battle each other, typically without player input"; depth is composition and positioning ([Wikipedia](https://en.wikipedia.org/wiki/Auto_battler)). Path of Achra (2023) applies this to a roguelike: combat resolves automatically as you move; the build is the game (recollection, confidence medium-high). Carnage Heart (PS1, 1995) is the direct ancestor of "program then watch": flowchart-programmed mechs fight without input (recollection, confidence medium). Majesty (2000): heroes have autonomous personalities; the player only places bounty flags — indirect control with legible incentives (recollection, confidence high).

**Story sifting (academic).** "The problem of story sifting involves the selection of events that constitute a compelling story from a larger chronicle of events... a profusion of events, many of which are relatively uninteresting"; sifters are "mixed-initiative creativity support tools that help players narrativize their play experiences by surfacing sites of potential narrative interest as they emerge"; The Sims 2 already nudges the sim toward completing recognised "story trees" ([Kreminski et al., Felt, ICIDS 2019](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf); follow-ups Winnow 2021, StU 2022 on *unexpected* outcomes: [publications](https://mkremins.github.io/)).

**Principles extracted:**
1. *Legible intent* — the hero says what it is doing and which rule fired (Dragon's Dogma pawns shout; Claude's thought bubbles; qw's plan line).
2. *Visible near-misses* — HP bars, "barely", telegraphs; drama needs the watcher to see the alternative outcome.
3. *Narrated causality* — a diary/commentary voice (Godville, marble commentary, RimWorld event letters), not a log.
4. *Attribution to the player* — the story must route through something the player authored (rules, votes, heirs).
5. *Pacing controls* — speed, skip-to-next-event, "key highlights only"; the storyteller layer decides cadence.
6. *Highlight reels* — sift the chronicle (Felt-style patterns: comeback, betrayal, first-kill-of-X, near-death) into a post-run reel and a lineage history.
7. *Named recurring characters* — monsters and allies with names and grudges; watchers attach to individuals.
8. *Stakes the watcher holds* — bets, votes, or the rule set's reputation.

**So what for an auto-played roguelike:** The hero needs a voice (diary + rule callouts), a storyteller that controls cadence, and a sifter that writes the highlight reel. Without these three, an auto-played roguelike is a log scroller.

### B6. Raising and teaching a companion AI

**Dragon's Dogma pawns.** Your main pawn is yours; two more are borrowed from other players. Pawns "yell out useful hints and strategies" and "provide information about enemies"; sharing was designed as "casual multiplayer elements similar to watching a bulletin board system" ([Wikipedia: DD](https://en.wikipedia.org/wiki/Dragon%27s_Dogma)). Inclinations (Scather, Medicant, Mitigator, Challenger, Utilitarian, Guardian, Pioneer, Nexus, Acquisitor) are learned three ways: a Knowledge Chair questionnaire, imitation of the player (pick up lots of items → Acquisitor rises; heal often → Medicant), and command usage (Go/Help/Come). Bestiary knowledge accrues per monster (stars) and travels with the pawn; hired pawns come home with a star rating, a comment and gifts from strangers. Failure modes: inclinations *drift* unintentionally (the famous "my pawn became a Guardian and just stands next to me" or grabs flowers mid-fight), so players bought elixirs to reset them; DD2 cut nine inclinations to four plus explicit Specializations for legibility, yet still drew "poor AI for allies" criticism, and the Dragonsplague mechanic (an infected pawn silently disobeys, then massacres a town) became the canonical *opaque behaviour* backlash ([Wikipedia: DD2](https://en.wikipedia.org/wiki/Dragon%27s_Dogma_2); inclination details from recollection, confidence high; Dragonsplague, confidence medium-high). What made pawns *theirs*: they carry your knowledge, other players rate them, and they talk back.

**Creatures (Steve Grand).** Norns have neural-net brains, biochemistry and inheritable genomes; learn by drive reduction; players teach words via a learning computer or by naming objects; a disembodied hand tickles (reward) or slaps (punish); "creatures could choose to obey or refuse"; full life cycle with death; players "couldn't force creatures to obey, creating an impression that Norns possessed independence and personality"; community swapped Norns and bred lines; Dawkins called it "a quantum leap"; frustration: "considerable patience" required, "virtual fish tank" ([Wikipedia: Creatures](https://en.wikipedia.org/wiki/Creatures_(video_game_series))). Known failure mode: Norns doing dumb things opaquely (starving next to food, the C3 "One Hour Stupidity Syndrome" genome bug) — bonding survived it because the creature was *vulnerable and individual* (recollection, confidence medium).

**Black & White.** Reinforcement (slap/stroke), imitation of the player's acts, and three leashes (attention, good, evil); BDI architecture by Richard Evans — beliefs as data structures, desires as perceptrons, opinions as decision trees; the creature learns "what", "when" and "how" separately; visible morphing (glowing eyes vs purple glow) made learning legible; reception: "revolutionary", but later "poor use of the much-lauded creatures" ([Wikipedia: B&W](https://en.wikipedia.org/wiki/Black_%26_White_(video_game))). Molyneux's postmortem: players wanted the creature to learn faster and could not tell *why* it did things; teaching by punishment was mis-attributed (Game Developer postmortem unreachable; recollection, confidence medium). The lesson everyone drew: implicit learning is charming for an hour and opaque for a hundred.

**Tamagotchi / Nintendogs / Monster Rancher / Digimon World.** Attachment comes from reciprocal care: "when the cared-for object thrives and offers us its attention and concern, people are moved to experience that object as intelligent" ([Wikipedia: Tamagotchi effect](https://en.wikipedia.org/wiki/Tamagotchi_effect)). Nintendogs: the dog learns *its name in your voice* and tricks by mic; needs shown as plain words ("Hungry", "Famished"); neglect degrades via the real clock ([Wikipedia](https://en.wikipedia.org/wiki/Nintendogs)). Monster Rancher: monsters generated from *your* CDs; "Loyal monsters are more likely to listen to commands, while disloyal monsters might refuse"; lifespan ends in retirement; retired monsters combine ([Wikipedia](https://en.wikipedia.org/wiki/Monster_Rancher)). Digimon World: "The player cannot control the actions of an unintelligent Digimon, but as it gets smarter more control over its actions is given" — *control itself is the reward for raising*; bad care visibly punishes (Numemon); critics still hated "the largely uncontrollable nature of its combat system" ([Wikipedia](https://en.wikipedia.org/wiki/Digimon_World)).

**Extracted:**
- *Ownership signals*: created by you, seeded by you (CD/name/voice), carries your knowledge, rated by others, comes home with stories.
- *Legible learning*: explicit stats over implicit weights (DD2 > DD1, Nintendogs word-states, B&W morphs). The FF12 gambit is the *most* legible form: the "learning" is literally text the player wrote.
- *Partial obedience is a feature*: Creatures, Monster Rancher loyalty, Godville. A hero that can refuse (fear, greed, fatigue traits) feels alive — but only if the refusal is announced and attributable.
- *Failure modes*: opaque drift (DD1 inclinations), stupid-looking deaths with no explanation (Norns, B&W), control withheld too long (Digimon World), uncanny stupidity where the agent ignores obvious danger the watcher can see.

**So what for an auto-played roguelike:** Make the gambit list the hero's legible "personality" and let a small trait layer (greedy, cowardly, curious) add announced deviations; never let behaviour drift silently. Ownership comes from the hero carrying the player's rules *and* a bestiary the hero learned across runs.

### B7. Academic framing (brief)

- Sonia Fizek, *Playing at a Distance* (MIT Press 2022) and "Interpassivity and the Joy of Delegated Play in Idle Games" (ToDiGRA 2018): play is displaced to the machine, not removed; drawing on Pfaller/Žižek's interpassivity (the object acts/enjoys on our behalf); the idle player oscillates between spectator and manager (pages unreachable; recollection, confidence medium-high).
- Alharthi et al., "Playing to Wait: A Taxonomy of Idle Games" (CHI 2018): idle games span an interactivity spectrum; waiting is itself play; check-in loops ([ACM DL](https://dl.acm.org/doi/10.1145/3173574.3174195), unreachable; confidence medium).
- Björk & Juul, "Zero-player games" (Philosophy of Computer Games 2012): setup-only games, solved games, AI-vs-AI — your design is a "setup-only" game with a long setup (recollection, confidence medium).
- Kreminski & Wardrip-Fruin, "Generative games as storytelling partners" (FDG 2019) and Ryan's dissertation *Curating Simulated Storyworlds* (2018), which coined story sifting; Eladhari, "Re-tellings: the fourth layer of narrative" (ICIDS 2018): retellings by players are the evidence a story generator works ([Felt](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf) cites all three).
- Jesper Juul, *The Art of Failure* (2013): failure is what makes games meaningful; players want failure to be attributable (confidence high).
- Bogost on Cow Clicker / Keogh & Richardson "Waiting to play" (Games & Culture 2018): idle games as background labour — a warning about pacing with nothing to watch (confidence medium).

**So what for an auto-played roguelike:** The literature says delegated play is real play *if* the player's authorship is visible in the outcome and the game returns the story as a retelling. Design for retellings: what would a player paste into a YASD thread?

---

## Roguelike systems to ship, ranked by story-per-implementation-cost

1. **Rule-firing trace + death attribution screen** (last N rule firings, telegraphs, YASD/YAAD verdict, "patch rule" button). Cheap; makes every death the player's.
2. **Auto-notes chronicle + Felt-style highlight sifter** (comeback, first-kill, near-death, ally lost, item gamble). Cheap once events are structured; produces the retelling.
3. **Monster intent telegraphs + hero callouts** ("the archer draws", "Retreat: HP 38%"). Cheap; supplies legibility for the whole spectator layer.
4. **Fight/retreat/escape gambit family with scarce escape consumables** (teleport, blink, dig). Medium; the highest-drama decisions.
5. **Dual-use consumables** (throwable potions, read-vs-keep scrolls) with benevolent/malevolent partial ID. Medium; every unknown item becomes a mini-lottery to watch.
6. **Terrain hazards that hit both sides** (gas, fire, deep water, chasm) with ~10 monster interactions (bloat, jelly split, thief, summoner, shield-buffer). Medium; this is where "the DevTeam thought of everything" moments come from.
7. **Allies with state** (freeable captives who level, can die, can be abandoned by rule). Medium-high; the single best source of tragedy.
8. **Forward clock** (Zot-clock/alert-level style, not hunger). Cheap; keeps an idle hero moving and makes dive/explore a real gambit.
9. **Lineage metagame** (heirs with traits, graveyard with deeds, bestiary knowledge carried across runs, seeds). Medium; makes death productive and the hero "yours".
10. **Storyteller pacing layer + spectator controls** (speed, skip-to-event, key-highlights mode). Medium; not story-generating but decides whether anyone watches.
11. **Vault "choose 1 of N" and god/altar commitments as rare direct interrupts**. Cheap; the one moment a human hand is welcome.
12. **Named recurring monsters with grudges** ("the goblin that killed Heir #3"). Cheap on top of lineage; watchers attach to individuals.
13. **Shareable run records** (morgue-style text, seed, rule set) for retellings. Cheap; the community engine.

Skip or defer: hunger accounting, shops, large flavour-only monster/item lists, blind full-ID, instant-death traps, XP grinding.

## Ten principles for making the player feel they played the run

1. **Every death names a rule.** The cause is something the player wrote or failed to write, never "the AI".
2. **Say what the hero is about to do before it does it.** Intent first, action second; the watcher must be able to wince.
3. **Automate no-brainers silently; surface trade-offs.** DCSS's rule is your split between engine and gambit layer.
4. **Scarcity makes rules matter.** Thresholds are only interesting when the potion might be the last one.
5. **The hero has a voice, not a log.** Diary, callouts, commentary; narrate causality in words.
6. **Sift, then show.** A highlight reel and a lineage chronicle, not a message scroll.
7. **Let the hero refuse — loudly.** Traits produce announced deviations; nothing drifts silently.
8. **Bank on retreat, not on death.** Give the player a choice of when to stop risking (Loop Hero, Dead Cells).
9. **The hero carries the player forward.** Bestiary knowledge, heir traits, the rule set's win rate: what the player taught persists.
10. **Design for the retelling.** If a run cannot be pasted into a YASD thread in three sentences, the story layer is not done.
