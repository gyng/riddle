# Comparables: auto-playing RPGs, auto-battler roguelikes, and "program your hero" systems

Research date: 2026-09-15. Review counts are Steam `appreviews` API totals (all languages) unless noted; "%" = positive share. Quotes are from the most-helpful Steam reviews of the last 365 days, Reddit threads, wikis, or developer interviews as linked. Confidence marked **[high/med/low]** where evidence is thin. Web search budget was constrained; primary sources were fetched directly (Steam API, wikis, interviews, Reddit mirrors).

Target design, for reference: real roguelike (procedural dungeon, permadeath, items, monsters) + hero is an AI the player programs FF12-gambit-style + idle pacing + metagame/unlocks + equipment and skills.

---

## Part 1 — Auto-hero games

### Loop Hero (Four Quarters, 2021) — the reference case
**Loop:** Hero auto-walks a loop and auto-fights; you place terrain/enemy cards to shape the loop, equip drops, and choose when to retreat. Between runs, camp buildings unlock cards/classes/mechanics.
**Numbers:** 36,543 reviews, 93% Very Positive (API); 500k copies in 7 days ([Inverse interview](https://www.inverse.com/gaming/loop-hero-interview)); Metacritic 82 ([Wikipedia](https://en.wikipedia.org/wiki/Loop_Hero)).
**Why the split worked (dev framing):** "You just control the adventure, not the hero"; "it's not an idle game at all"; the hook is "when you feel that you have found the right balance, and your hero can handle any problem, but then you change one item, everything breaks down" ([GameRant interview](https://gamerant.com/loop-hero-interview/)). Players describe the role identically: "You don't actually control the hero, you build the world around them"; "You're tuning enemy density, scaling, and build synergies yourself, balancing pressure and payoff, then deciding how far to push before pulling out alive" (Steam, 39h).
**Retreat/death model [high]:** retreat at the campfire keeps 100% of resources, retreat mid-loop 60%, death keeps 30% ("lose 70%" per Wikipedia). Death is a resource tax, not a wipe; the meta (camp) is never lost. Discovered gear is per-run. This makes death the *least* efficient exit rather than a punishment, and turns "when to retreat" into the core decision.
**Complaints (very consistent):** (1) grind gate on the camp: "after the first few cheap buildings, everything takes several maps to afford" (r/LoopHero); "the back 80% once you hit the grind is really boring and it requires just enough attention that you can't just enjoy it as an idler game" ([r/patientgamers](https://www.reddit.com/r/patientgamers/comments/1tb1ko1/)); (2) low decision density: "I feel there is almost no meaningful decisions to make, and I just end up passively staring" (r/LoopHero); "tiles combinations are 0-2 layers deep and honestly the player just stacks them all in one place" (Steam); (3) unreadable failure: "You can lose a run cause monsters steal your gear and you don't notice"; "lose the entire run unpredictably"; (4) abandoned content ("Belt and Gloves" slots never implemented; no updates in 3+ years) is the #1 recent negative.
**Steal:** the three-tier exit (safe exit > risky exit > death) with meta never lost; camp unlocks that add *new mechanics/classes*, not just numbers; unlocks can be toggled off so the pool never bloats (praised explicitly); hero-inventory autoplay with a small number of high-leverage manual decisions.
**Avoid:** resource caps per run + expensive buildings (forces re-running solved loops); decision density that drops to zero mid-run; unreadable causes of death; shipping placeholder slots.

### Progress Quest (Fredricksen, 2002)
**Loop:** Zero-player parody; roll a character, watch progress bars kill monsters, loot, level. Nothing the player does after creation matters.
**Reception:** "the first example of ... the 'idle RPG'"; influenced Fallout Shelter per Todd Howard ([Wikipedia](https://en.wikipedia.org/wiki/Progress_Quest)). Interviewed in the CHI PLAY idle-games paper below.
**Steal:** the comedic loot/quest name generators; the fact that watching a bar can carry affect if the *text* is good.
**Avoid:** zero-input as the whole game; it is a joke that lands once.

### Godville (2010, web/mobile)
**Loop:** Your hero autonomously quests, fights, and writes a diary; you are its god with sparse levers (encourage/punish, "voice of god" text commands, prayers). Mostly community-authored content.
**Reception:** reviewers surprised by longevity ("playing continuously for nearly four years"); praised writing; engagement compared to "monitoring email" ([Wikipedia](https://en.wikipedia.org/wiki/Godville)). Player counts not verifiable here **[low]**.
**Steal:** the hero's diary as the "why did my character do that" surface — narrated intent is cheap and beloved; the god/hero relationship makes non-control feel thematic rather than absent.
**Avoid:** levers so weak that outcomes feel unattributable (works for a satire, fails for a roguelike).

### Idle Champions of the Forgotten Realms (Codename, 2017)
**Loop:** Place champions in a formation grid; adjacency buffs and click/auto damage push waves; swap champions per "variant" objective. F2P with paid champion packs.
**Numbers:** 15,051 reviews, 77% Mostly Positive.
**Praise:** formation puzzles are "surprisingly deep"; "the game has a huge 'iceberg' of content" (Steam, 7,548h).
**Complaints:** it stopped being idle: "changed from a fun idle game where you could set up a team and let it run ... to a babysitting game where you have to check the team every 2 minutes" (Steam, 1,161h); "commits one of the cardinal sins of idle games: it demands your attention" (Steam, 9,866h); monetization dominates the negative reviews.
**Steal:** per-run "variant" constraints that force a different loadout each time (cheap replayability); adjacency/formation as a loadout puzzle.
**Avoid:** drift from set-and-forget to attention tax; FOMO monetization.

### Melvor Idle (Games by Malcs, 2021)
**Loop:** RuneScape-in-a-spreadsheet; combat is fully automatic once you pick a target; dungeons run unattended; deaths cost a random equipped item; Hardcore deletes the character.
**Numbers:** 16,216 reviews, 90% Very Positive.
**Death rules [high]:** "Upon death a random equipment slot is selected, any Equipment within the selected slot will be lost forever (unless the Protect Item prayer is active)"; idling is "safe" iff your Auto Eat threshold ≥ the monster's max hit; Auto Eat tiers heal to 40/60/80% ([wiki](https://wiki.melvoridle.com/w/Combat)). The community built a Combat Simulator mod to answer "can I idle this?"
**Praise:** "boss fights actually need planning"; "I can take a short break, check my progress ... and go back to work" (Steam). The thing players value is *pre-combat certainty*: you know before committing whether the AI can survive.
**Complaints:** online-server requirement for a single-player game (top negatives); lost Hardcore accounts when servers went down.
**Steal:** a single legible safety inequality (threshold vs max hit) as the idle-combat contract; an in-game simulator/forecast; a "protect one item" prayer as a paid insurance slot; Hardcore as an opt-in mode rather than the default.
**Avoid:** any online dependency for a single-player idle; death penalties that can be caused by the game (disconnects) rather than the player's plan.

### NGU Idle (4G, 2019)
**Loop:** Adventure mode auto-attacks through zones; bosses gate zones; everything else is numbers-go-up with many prestige layers. Death in adventure just respawns you (no loss) **[med]**.
**Numbers:** 12,978 reviews, 96% Overwhelmingly Positive. Extreme retention: multiple 10,000-25,000h reviews ("It took me THREE YEARS").
**Complaints:** "Disrespectful of players time. Making things take longer doesn't equate to a better idle game"; offline progress excludes the auto-equip loop; CPU load.
**Steal:** humor as the retention glue; visible "you are X% through" progress; automation itself as the reward (auto-equip, auto-boss).
**Avoid:** offline/online asymmetry; time-walls presented as content.

### Trimps (2014 web, Steam 2022)
**Loop:** Build a colony, send trimps to fight zones automatically; portal (prestige) resets with perks; automation (AutoFight, AutoStorage, etc.) is unlocked gradually as *rewards*.
**Numbers:** 1,498 reviews, 90% Very Positive.
**Complaints:** "you don't get automation until hundreds of hours in"; "10-15 days PER milestone of content".
**Steal:** automation features as unlocks (players earn the right to not click); the community expectation of exportable saves.
**Avoid:** locking basic automation so late that the pre-automation game is a chore.

### Legends of Idleon (Lavaflame2, 2021)
**Loop:** MMO-styled idle with multiple characters farming while offline; huge system count.
**Numbers:** 29,799 reviews, 77% Mostly Positive; top negatives (400-630 helpful votes each) are all monetization/FOMO/"weekly paychecks" and dev conduct.
**Steal:** multi-character parallelism (several heroes on different jobs) is the strongest idle hook here.
**Avoid:** everything about the monetization trajectory; a dev who moderates his own criticism.

### Idle Slayer (Pablo Leban, 2020)
**Loop:** Auto-runner that auto-attacks; you jump/collect; portals to minigames; prestige layers.
**Numbers:** 10,862 reviews, 85% Very Positive. Complaint cluster: "You can't consider this game an idler when you are punished for idling"; active-only quests; alleged rigged odds.
**Steal:** nothing unique.
**Avoid:** advertising idle while gating progress behind active minigames; fighting player automation/macros in a single-player game.

### Nomad Idle (2025) — a near-miss on the target design
**Loop:** Auto-shooter hero (Vampire-Survivors-like) that plays itself; you buy upgrades and reset characters. Marketed as an "idle RPG-like".
**Numbers:** 883 reviews, 65% Mixed.
**Why it underperformed:** "You really don't feel that your input on the game does anything"; "There's no actual gear, or character specific trees, or real decisions to make ... missing 90% of what made those games interesting"; "every piece of gear you get is always equipped ... you just progress down the path and reset"; "after around 5 hours in, no more mechanics will be introduced". No offline progress. Positive reviews say it's fine as a second-monitor game.
**Steal:** stats/skills modifiable mid-run ("so you can experiment when things get tough") was praised.
**Avoid:** auto-hero with no loadout decisions; linear gating; no new mechanics after hour 5; no offline progress.

### Astronarch (2021)
**Loop:** Party-based roguelite auto-battler; pick heroes, equip items, choose map paths; battles auto-resolve.
**Numbers:** 1,145 reviews, 89% Very Positive; ~30-40h to 100%.
**Complaints:** "The only strategy is to set your heroes up so that your tank will be the first to be attacked ... After that. You watch and maybe use potions"; "runs start feeling repetitive ... once you've seen most of the available units"; "Painfully slow (Desperately needs a speed-up toggle)".
**Steal:** Ascension ladder (A1-A20) as the meta; potions as the one manual lever.
**Avoid:** auto-battles where positioning is the only real decision; no speed control.

### Despot's Game (Konfa, 2022)
**Loop:** Roguelike auto-battler: buy humans, assign classes, place them, watch fights; async PvP at the end.
**Numbers:** 3,356 reviews, 84% Very Positive.
**Praise (best articulation of "responsibility" in an auto game):** "Entire teams wiped out because I trusted one unit too much. Perfect-looking builds collapsing because I stood one tile too far to the left. The game never yelled at me for it. It just let me watch the consequences. And somehow, that was fun."
**Complaints:** "feels more like a very long demo"; PvE gets boring; PvP meta-chasing.
**Steal:** consequences that are visibly traceable to a placement decision; the "puzzle that refused to stay solved" feeling.
**Avoid:** shallow run structure; PvP as the only endgame.

### Backpack Battles (PlayWithFurcifer, 2024)
**Loop:** Buy items, tetris them into a bag so adjacencies trigger; fights auto-resolve against other players' bags.
**Numbers:** 20,701 reviews, 91% Very Positive.
**Praise:** async ("I can get up and do things while playing"); "an excellent replay and recap system ... letting you go back and see how each battle panned out ... to see what you could have perhaps done better" (Steam, 25h); "The only PvP game worth playing - because there's no interaction with other players."
**Complaints:** matchmaking/power creep at high rank; balance churn.
**Steal:** the replay/recap: an auto game earns responsibility by letting you *rewatch and diagnose*; spatial adjacency as the programming surface.
**Avoid:** ranked PvP treadmill for a single-player roguelike.

### Super Auto Pets (Team Wood, 2021)
**Loop:** Buy/position pets whose abilities trigger by position and event; auto-battles vs. async opponents.
**Numbers:** 36,176 reviews, 90% Very Positive. Complaints are pack monetization and rigged-feeling matchmaking; "Keywords are not explained, and don't do what you think they would do half the time."
**Steal:** trigger vocabulary (on faint / on hurt / on buy / start of battle) is a proven, learnable event-programming language for non-programmers.
**Avoid:** ambiguous keyword semantics; content behind packs.

### Vampire Survivors (poncle, 2022) / Brotato (Blobfish, 2023)
**Loop:** Movement-only (VS) or movement+aim (Brotato) input; weapons auto-fire; per-run build from randomized level-ups; meta unlocks via gold/achievements.
**Numbers:** VS 265,738 reviews, 98%; Brotato 118,814 reviews, 96%.
**Design analysis:** "only requires directional controls"; near-miss on every sub-30-minute run; "No run ever feels wasted as players feel a sense of increasing mastery ... even if they only gain a little gold" ([The Conversation](https://theconversation.com/vampire-survivors-how-developers-used-gambling-psychology-to-create-a-bafta-winning-game-203613)). Galante: "balance is completely out of the window. I just want to make stuff that is fun" ([Pocket Tactics](https://www.pockettactics.com/vampire-survivors/interview)).
**Steal:** every run pays into meta; short runs (VS 30 min, Brotato ~20 waves); unlock cadence measured in minutes early on; huge visible power growth inside a run.
**Avoid:** Brotato's unlock bloat into the pool (Loop Hero's toggle-off was praised specifically as the fix).

### Path of Achra (Ulfsire, 2023) — minimal-input traditional roguelike
**Loop:** Grid roguelike where you mostly hold auto-move and let the build's on-hit/on-move triggers do everything; "broken build sandbox"; ascension cycles.
**Numbers:** 3,256 reviews, 98% Overwhelmingly Positive.
**Praise:** "Can be played with one hand"; "a theorycrafting game"; the joy is authoring a trigger cascade and watching it go.
**Complaints:** "you'll spend most of the game holding the auto-move button watching sprites disappear"; "you'll die for reasons you don't know or couldn't have prevented"; only a few builds work at high cycles.
**Steal:** trigger-based skills ("on kill", "on step", "on hit") as the real "programming"; gods/classes as build seeds; permadeath with fast restart and no meta-grind gate.
**Avoid:** opaque death causes; "too much information to determine whether a build is successful ... forced to try and fail".

### Stone Story RPG (Martian Rex, 2019 EA / 2021) — closest shipped comparable
**Loop:** ASCII auto-RPG; hero auto-runs levels; you choose gear and craft; ~3h in you get the Mind Stone and write **Stonescript** ("`?hp < 7` → `activate potion`"; runs 30x/sec; top-to-bottom; indentation = nesting) to automate weapon swaps, potions, abilities, farming ([Stonescript intro](https://stonestoryrpg.com/stonescript/)). Pitch: "If you can type, you can Stonescript--no programming degree required!"
**Numbers:** 996 reviews, 91% Very Positive.
**Praise:** "Scratches that coding itch without being super complicated or tedious"; scripts shared on GitHub and Discord; wiki says scripting is "strongly recommended, if not needed" for regular play.
**Complaints:** "Where is the 'programming' this was tagged with... (you don't get it until ~3 hours in)"; mobile-game bones (dailies, time gates, "replaying the same levels over and over again to farm").
**Steal:** unlock the scripting layer only after the player has felt the pain it solves (they explicitly gate it behind a boss that teaches weapon-switching); one-line `?condition action` syntax; import of community scripts.
**Avoid:** mobile time-gates; letting the script be the whole endgame.

### Auto Rogue (定期的な宝物, May 2025) — "combat programming roguelike", the most direct precedent
**Loop:** Each run you pick skills and attach *conditions* to them ("reduce HP on use but lower cost", "if it lands the killing blow, +max HP"), then press play; enemies with unique traits change what the optimal program is. Store copy: "everything is decided before the battle begins."
**Numbers:** 719 reviews, 91% Very Positive, $6.99.
**What worked:** "the unique build you'll have each run, the unique mechanics of each enemy, and the strict limitations on the size of your program mean that you'll have to come up with a new variation of your program for every single fight" (Steam, 6h); "The game tells you what the result is (probably) going to be" (a forecast).
**What failed:** "Way too easy and way too short" (top review); no endless mode; no mid-run save (60-90 min commitment); "there appears to be only one thing to unlock"; two characters; "builds are much too on rails"; translation. The concept got 91% but players ran out of content in 1-5 hours.
**Steal:** program-size limits as the difficulty knob; enemy traits that invalidate last fight's program; a pre-battle forecast.
**Avoid:** shipping without meta/unlocks, endless mode, or mid-run save; difficulty that is only number scaling.

### Septaroad Voyager (2023 EA) — FF12 clone
JRPG with "Tactics" (condition + action, unlockable slots) and license boards. 34 reviews, 85%; still EA in 2025; negatives cite glacial development. Evidence that "gambits" alone don't sell; gambit players find it by searching for the mechanic ("I discovered Septaroad Voyager when looking for more games that have a 'gambit' feature"). **Steal:** nothing. **Avoid:** cloning the surface of FF12 without a reason for the automation to exist.

### Other "idle dungeon" Steam titles (2022-2026), with reception
- **Lootun** (2024): 1,055 reviews, 94%. Party idle looter, "Melvor without the professions"; praised as "best idle RPG experience"; negatives: "I'm supposed to exclusively be managing the scrapping filter and equipping gear"; "brain dead ... once you get to late game. It's more about clicking the 'auto equip best' button". **Steal:** loot filters as player-authored automation. **Avoid:** auto-equip-best that erases the decision.
- **Legends of Dragaea: Idle Dungeons** (2025): 997 reviews, 87%. First-person Wizardry-style auto-crawler. Negatives: "There are no stakes, nothing. It's a wallpaper simulator"; "lacks the sense of achievement and fine-tuning that define good idle games"; the auto-build system is "flawed" and manual team building is required. **Avoid:** no stakes.
- **Dark Hunting Ground** (2025): 770 reviews, 92%. Minimal ARPG that transitions "from hands-on farming to fully automated looting as you refine your systems"; complaint: runs too similar; grind wall at level 50.
- **Talented** (2024): 1,475 reviews, 91%. Stand-still shooter + randomized talent graph; negatives: "You have to constantly manually aim ... for several minutes between any interesting choices"; "The random tree is so big that everything averages out, no choice is truly meaningful."
- **Dragon Cliff** (2018): 8,200 reviews, 89%. Idle party ARPG with auto-play; long-lived.
- **The Perfect Tower II** (F2P, 2025 1.0): 4,449 reviews, 87%. Incremental with a real in-game scripting "AI"; negatives: "trying to do too much"; tutorials unhelpful.
- **Guild of Dungeoneering** (2015): 2,633 reviews, 77%. Early "you build the dungeon, the hero picks the path" precedent; "plays its whole hand within the first few hours"; grind and RNG.
- **Dungeon Team** (2024): 189 reviews, 79%. Has a 5-setting priority AI; negative: "You can't work out target or skill priorities with specificity and the 5 priority settings you have for the AI are very inconsistent." **Avoid:** coarse AI sliders that are neither predictable nor expressive.
- **Nomad Idle**: see above (65% Mixed).

---

## Part 2 — Behaviour-programming systems

### Final Fantasy XII gambits (Square Enix, 2006; Zodiac Age 2017)
**Structure [high]:** ordered list of (target+condition, action); first true row fires each turn; manual commands interrupt; 12 slots max (2 free, 10 bought on the License Board); gambit *conditions* are bought in shops, so vocabulary is gated by progression ([FF wiki](https://finalfantasy.fandom.com/wiki/Gambits)). Hidden sanity rules: won't recast a buff you already have, won't revive the living, two characters won't double-cast the same buff.
**Design intent (devs):** Ito wanted a "single-player online game" with "independently acting party members who would still act the way the player wanted them to"; prototype came from FFIV's auto-battle AI; inspired by American-football plays ([Wikipedia](https://en.wikipedia.org/wiki/Final_Fantasy_XII)). Kato: "if we added just the real-time aspect to the command based battle system ... controlling everything might be too fast-paced and difficult"; "There's a great feeling of triumph when you defeat a formidable enemy through a fine-tuned setting of your Gambits" ([PlayStation Blog](https://blog.playstation.com/2017/07/07/extended-play-how-final-fantasy-xiis-gambit-created-one-of-the-most-distinct-rpgs-ever/)). Katano: "it was really hard to gauge whether or not what we were doing was going to work nicely or not until the very end."
**"The game plays itself":** the enduring complaint ("Gambits off = uninteresting and clunky battle system / Gambits on = take a nap while the game plays itself", r/FinalFantasy). Defenders: "if you try to auto-pilot only using gambits on hunts, rare game, espers, or bosses, you're just gonna die. The real way to use gambits is as a helper tool" (Steam forum). IGN's line: "gambits do not function without a player." The critique that actually matters for design: "FFXII has an incredibly straightforward roster of spells, abilities and equipment, and close to zero synergies across them, so it mostly serves as a method of automating mundanity rather than an actual vehicle for player expression" (r/JRPG, 64 pts, [thread](https://www.reddit.com/r/JRPG/comments/13zem51/)). Also: "too granular. there are waaaay too many, and i'd guess some 80-90% of them are very rarely used"; gating by shop purchase "just feels like it needs polish".
**What fans loved:** discovery of combos: Reverse+Decoy tank ("Each attack on the character with Reverse will actually heal"), "Foe: HP = 100% → Steal", "Foe: Weak to [element]" auto-targeting boss phase shifts; full gambit sets are traded on Reddit/GameFAQs. Fans "would kill for another game with it" (165 pts).
**Zodiac Age:** 12 job boards, 2x/4x speed toggle, later three saveable gambit sets per character. Speed mode is the tell: once the AI works, the player wants to *watch faster*, not intervene more. 10,261 Steam reviews, 89%.
**Steal:** first-true-row priority list (people understand it instantly); hidden no-redundancy rules; vocabulary that expands with progression; multiple saved sets per situation; speed toggle.
**Avoid:** 80% unused conditions; flat ability roster with no synergies (the automation then has nothing interesting to express); gating basic conditions behind money.

### Dragon Age: Origins tactics (BioWare, 2009)
**Structure [high]:** per-character list of (condition, action, target), top-to-bottom, evaluated continuously in and out of combat; if a row can't execute (no mana, cooldown, unreachable) it falls through; if nothing fires, a coarse *Behavior* preset (Default/Aggressive/Ranged/Passive...) acts; 2 slots at L1, +1 at 3/6/10/15/20/25/30, more via the Combat Tactics skill; class presets ("Healer") ship with 10+ rows; 3 custom presets; conditions include "Enemy: Target of [party member]", "Enemy: Clustered", "Self: Mana < X%" ([DA wiki](https://dragonage.fandom.com/wiki/Tactics_(Origins))).
**Praise:** "multiple tactic slots could be linked up to create a skill with two or more conditional qualifiers ... Origins also had cooldowns though, which gave the AI more variety" (r/gamedesign); "Tactics tab ABSOLUTELY nails this feeling."
**Complaints:** "there are simply not enough tactics slots to use a Mage effectively entirely on AI"; slots cost skill points that compete with combat skills; "the standard presets are less than optimal and the mechanisms are very unclear"; no movement actions ("you cannot use a tactic to make a character flee from Inferno"); sustained-ability bugs; "the never attacked when they should no matter what you told them."
**Community craft:** the wiki's build recipes (Support/Healer/Melee) and the "free sustained abilities via mana thresholds" exploit show players treat it as a programming language with idioms.
**Steal:** fall-through on *can't execute* (not just condition false); the Behavior fallback preset for when no rule fires; "target of X" conditions for focus fire; shipping strong class presets to copy from.
**Avoid:** slots that compete with combat power; no movement/positioning verbs; opaque preset behaviour.

### Pillars of Eternity II: Deadfire AI behaviours (Obsidian, 2018)
**Structure:** unlimited rows of (multiple conditions with AND/NOT, action, target, per-row cooldown), grouped into action sets; scripts are exportable and shared ([r/projecteternity](https://www.reddit.com/r/projecteternity/comments/1ai1pe3/)).
**Praise:** "I've fine-tuned my party AI to be self-sufficient ... my casters ... conserve at least one spell slot in the case of an emergency" (20 pts); "You can make the game fully automated"; "makes it like a whole new game."
**Complaints:** "super daunting ... options and combinations seem endless"; "AI won't cancel actions mid-execution for emergencies"; some actions "just better done manually"; too good ("the game becomes too easy").
**Steal:** per-row cooldowns (prevents spam without extra conditions); exportable scripts; "conserve N resource" style conditions.
**Avoid:** unlimited slots with no onboarding ladder; editor that opens on a blank page.

### Kingdom Hearts party AI / Xenoblade
KH1/2 "Customize" menu gives 4-step frequency sliders per category (Constantly/Occasionally/Rarely/Never) and item-use thresholds ("HP items: only in emergency"); players mostly ignore it: "Kingdom Hearts games have terrible AI" (GameFAQs). Xenoblade series ships no per-party programming (XC3 adds a party-wide Auto-Battle toggle) **[med]**. Lesson: coarse sliders don't create ownership; nobody trades KH configs.
**Steal:** nothing. **Avoid:** frequency sliders as the whole interface.

### NieR:Automata plug-in chips (PlatinumGames, 2017)
**Structure:** passives, HUD elements, and the OS itself are chips with a storage cost (64 → 256 capacity; 1-16 each); fusion compresses duplicates; Easy mode adds Auto-Attack/Auto-Evade/Auto-Fire chips that literally play the game ([analysis](https://saschb2b.github.io/game-mechanics/games/nier-automata/chip-system)). 153,361 reviews, 87%.
**Steal:** a single memory budget where automation, information (HUD), and power compete — "Showing a minimap means less room for damage bonuses"; letting the player remove something load-bearing (OS chip = death).
**Avoid:** fusion-inventory tedium (players made spreadsheets and charts just to fuse efficiently).

### Dragon's Dogma pawn inclinations (Capcom, 2012 / 2024)
**DD1:** nine inclinations stacked in a hidden primary/secondary/tertiary order, set through the Knowledge Chair Q&A and elixirs, plus a bestiary knowledge system; pawns shared online. Reality: "over 70% of people were unaware of how the system worked so you'd see countless mages with Scather" (r/DragonsDogma); pawns learned bad habits from other players' worlds.
**DD2:** four inclinations (Kindhearted/Calm/Simple/Straightforward) plus specializations; reaction: "Compared to DD1's system? It's a god send"; residual complaint that voice is tied to inclination and that descriptions don't predict behaviour ("I want someone that is clever ... not running away D: I don't know what it will do exactly"). DD2 62% Mixed for unrelated reasons (performance/MTX). 
**Steal:** pawn chatter that announces intent ("yelling out useful hints") is the best in-class "why did it do that" surface; sharing your configured hero with other players.
**Avoid:** hidden stacked-priority state edited through indirect UI; behaviour descriptions that don't predict behaviour.

### Carnage Heart (Artdink, 1995)
Flowchart-chip programs on a grid for mechs, looping from top-left; hardware+software loadouts. Reviews: "deep, sophisticated, and well-implemented" but "most gamers would find its complexity overwhelming and the lack of direct control over the mecha frustrating"; EGM: "Game you Need a Ph.D. to Play" with a 60-page guide ([Wikipedia](https://en.wikipedia.org/wiki/Carnage_Heart)). Five sequels in Japan, one localized.
**Steal:** spatial program layout as a puzzle (chip footprint = budget). **Avoid:** requiring a manual to reach the first success.

### Gladiabots (GFX47, 2019)
Drag-and-drop behaviour trees (priority left-to-right; condition nodes with filters; action leaves) for robot squads; campaign + async ranked "ghost" ladder; 913 reviews, 88%.
**Praise:** "programming without having to remember the syntax"; "Simple logic can be very powerful, whereas too much complexity, often has flaws, you cannot put your finger on" (473h).
**Complaints:** "there's no progression? Like you can't upgrade the bots at all, you just change how they think ... once you get a good enough AI setup ... no reason to change anything"; "it feels a bit like work"; "Just trying to guess the order it's going to execute." Dev's own lesson: tutorials are "very difficult to integrate if it wasn't thought through from the start" ([interview](https://www.pointnthink.fr/en/gfx37-no-plan-b/)).
**Steal:** ghost opponents at your level; the AI debugger (step through which node fired).
**Avoid:** pure programming with no RPG progression — players explicitly want *both* "make the bot smarter" and "make the bot stronger".

### Screeps (2016) and Bitburner (2021)
Real JavaScript. Screeps: persistent MMO, "your code is the player" (2,094 reviews, 86%; recent negatives about an unpatched RCE). Bitburner: incremental where scripts hack servers (7,409 reviews, 95%; free). Both are loved by programmers and bounce everyone else: "Anyone telling you that you don't need to know anything about coding to play this game is straight up lying ... there are zero coding basics taught"; "The tutorial assumes the player have a basic understanding of programming that I simply don't have."
**Steal:** Bitburner's arc of *automation compounding* (scripts that buy servers that run scripts); Screeps' persistent world where code keeps working while you sleep.
**Avoid:** text code as the only interface for a general audience.

### Idle Loops (Stop_Sign, 2018) / Increlution (Gniller, 2021)
**Idle Loops:** you are in a time loop with a mana budget; you queue an action list, it replays every loop; knowledge (which pots have mana, explored town %) persists, skill levels reset; you continually re-author the list ([r/incremental_games](https://www.reddit.com/r/incremental_games/comments/ntwrgd/)). "It's the most similar to what made Factorio fun ... you need to constantly optimize your automation." Onboarding failed: "Holy moly this is a confusing game" (25 pts); the fan tutorial is longer than the game's. Top request: "preview or simulate a loop at a faster rate so you can see the mana cost and mana remaining at each stage."
**Increlution:** "a step-by-step queue system ... plan exactly what you want to happen and the game will automatically follow your orders"; the queue pauses the game when empty; two-tier skills (per-life Generation levels vs permanent Instinct levels); each life you die of aging, next generation is stronger; 1,256 reviews, 85%; r/incremental_games Best Game 2021/2022; abandoned in EA (top negatives). "a game about planning and strategizing, not a game about micro-management or clicking."
**Steal:** the *program is the run*; persistent knowledge as the metagame (the hero remembers the dungeon, not just gets stronger); a per-loop simulator/forecast; auto-pause when the plan is exhausted.
**Avoid:** no tutorial; content that never finishes.

### Autonauts (Denki, 2019)
Program bots by *demonstration* (record actions, wrap in repeat loops). 4,966 reviews, 89%. Praise: "Visual programming allows many good solutions to emerge without hand holding." Complaints: "You have to exemplify every action you want the bot to take ... Switching a workflow from a level 2 gadget to a level 3 gadget ... is super annoying"; bots "clog" and need manual rescue.
**Steal:** programming-by-demonstration for the first script; loops as the first abstraction.
**Avoid:** scripts bound to specific item tiers so upgrades break automation.

### Unicorn Overlord (Vanillaware, 2024) — the modern proof gambits sell
Per-unit priority lists with *two* conditions per row and a pre-battle forecast; battles auto-resolve. 1M+ sales by Sept 2024; Metacritic 86-90 ([review](https://www.yomiqo.com/en/unicorn-overlord-review/)). "The mere addition of a second condition to each command elevates it to the granularity required of a strategic RPG"; "the same team, with nothing changed but the order of their skill conditions, can produce wildly different battle outcomes." Onboarding still hard ("blankly staring at the screen ... overwhelming pit of information"). Community discovery moments ("Literally had to in Elfheim. This fool was Guard covering to death" → set Guard to physical-only, 169 pts) show the aha loop works.
**Steal:** two conditions per row, no more; a forecast before commit; enemies whose traits invalidate your default program.
**Avoid:** no in-battle intervention *and* no replay (the one thing critics missed).

### Zachtronics — lessons for making programming feel like play
From the 2019 GDC talk and the Dec 2025 SED podcast ([transcript](https://softwareengineeringdaily.com/wp-content/uploads/2025/12/SED1884-Zachtronics.txt)): every puzzle has "a huge number of solutions"; secondary optimization metrics (cycles/cost/size) "turn one solvable problem into a long-lived possibility space"; histograms instead of leaderboards ("we don't really need to attribute the top hundred people who all copied the best solution ... just show what's the distribution of scores"); the biggest accessibility fix in Kaizen was time-scrubbing: "you can just scrub back and forth and you can edit your program at any point in time ... you turn it into something that they're not going to kill"; the cost of accessibility is the core's disappointment ("the 10% of people who thumbs down our game are almost entirely people who liked our old work").
**Steal:** scrub/replay of the last fight with the program visible; secondary metrics per floor (turns, HP lost, potions used) with histograms; multiple valid programs per enemy.
**Avoid:** one intended solution per encounter.

---

## Part 3 — Synthesis

### (a) Control-granularity sweet spot for a gambit-style system
Evidence converges on a narrow band:
- **Row shape:** *target/condition(s) → action*, ordered, first-true fires, fall-through on can't-execute (DAO), with hidden no-redundancy rules (FF12). Two conditions per row is the demonstrated ceiling for mass appeal (Unicorn Overlord, 1M+ sales; DAO's chained rows praised); unlimited AND/NOT trees (PoE2, Gladiabots) are loved by a minority and "daunting" to the rest.
- **Slot count:** FF12's 12 and DAO's ~8-12 are enough; Auto Rogue found that a *hard program-size limit* is itself the fun ("strict limitations on the size of your program mean you'll have to come up with a new variation"). Never make slots compete with combat power (DAO's skill-point complaint) — make them progression rewards or a NieR-style shared budget with gear.
- **Vocabulary:** ship few conditions, all useful; FF12's "80-90% rarely used" and Dungeon Team's "5 inconsistent priority settings" are the two failure modes (too many / too vague). Super Auto Pets' event verbs (on hurt, on faint, start of battle) and Path of Achra's on-kill/on-step triggers prove that *event triggers* are the more expressive and discoverable half; add them alongside state thresholds (HP<X%).
- **Verbs must include movement/positioning.** DAO's "you cannot make a character flee from Inferno" and Astronarch's "only strategy is tank position" show that a roguelike hero needs retreat/kite/pick-up/descend verbs, not just cast/attack.
- **Coarse sliders (KH) and hidden stacked state (DD1) both fail**; DD2's simplification was called "a god send". Explicit lists beat personalities.

### (b) How successful games keep the player *feeling responsible* for an AI's outcome
- **Traceability:** Despot's Game ("one tile too far to the left ... it just let me watch the consequences"), Backpack Battles' replay/recap, Gladiabots' node debugger, Zachtronics' scrub. The common thread is a post-mortem the player can read. Loop Hero and Path of Achra draw the exact opposite complaint ("lose the entire run unpredictably", "die for reasons you don't know").
- **Forecast before commit:** Melvor's threshold ≥ max-hit rule, Unicorn Overlord's battle preview, Auto Rogue's "tells you what the result probably is", Idle Loops' top feature request (simulate the loop). A forecast converts "the AI failed" into "my plan was wrong".
- **The program must have something to express:** the sharpest FF12 critique is that with no ability synergies gambits only "automate mundanity". Path of Achra/VS/Brotato show that trigger cascades give a program *content*.
- **Announced intent:** Dragon's Dogma pawns shouting, Godville's diary. Cheap, and it converts observed behaviour into legible decisions.
- **Enemy traits that break last floor's program** (Auto Rogue, Unicorn Overlord's elf magic-conferral) force re-authoring, which is where ownership is renewed.

### (c) How death/permadeath was made to feel good in idle or auto contexts
- **Death as the worst of several exits, never a wipe of the meta:** Loop Hero 100/60/30% resource retention; VS "no run ever feels wasted". Players accept losing the *run*; they revolt at losing the *account* (Melvor server-death Hardcore losses, 212-helpful negative).
- **Death must be attributable and pre-announced:** Melvor's safety inequality makes idle death a plan failure; opt-in Hardcore for those who want stakes.
- **Legends of Dragaea shows the inverse:** no stakes → "wallpaper simulator". Stakes are wanted, but per-run.
- **Persisting knowledge, not just stats:** Idle Loops/Increlution make the *program* and *map knowledge* the survivor of death ("Instinct" levels vs per-life levels). This is the natural roguelike analogue: the hero dies, the gambit set and the bestiary survive.
- **Cheap restarts:** Path of Achra restarts in seconds with no camp grind; Loop Hero's camp grind is its most-cited flaw.

### (d) What the metagame looked like in the ones that retained players
- **Unlocks that add mechanics/classes/verbs, not multipliers** (Loop Hero camp; NGU's features; Trimps' automation unlocks). Auto Rogue died on "only one thing to unlock".
- **Toggleable unlocks so the pool never bloats** (Loop Hero praised; Brotato criticized).
- **Per-run constraint variants** (Idle Champions variants, Melvor slayer tasks, Astronarch A1-A20, Backpack Battles ranks) drive re-authoring without new content.
- **Multiple saved programs** (Zodiac Age's three gambit sets, DAO's three presets, PoE2 exports) — and communities *trade* them (FF12, DAO wiki, PoE2 Nexus, Stonescript GitHub). Design for shareable text.
- **Short runs, frequent meta payout, an ending.** The CHI PLAY idle-design study (6 designers incl. Progress Quest, Kittens, Universal Paperclips) recommends "design for plateaus that encourage players to disengage temporarily" and giving an ending "to say 'it's okay, you can stop playing now'" ([paper](https://par.nsf.gov/servlets/purl/10174274)). Idle Champions and Idleon show the retention-through-attention-tax route ends in 77% scores.
- **Fair pricing:** every 2024-26 idle/auto title with F2P+packs (SAP, Idleon, Idle Champions, Idle Slayer, Dungeon Team) carries monetization as its top negative; buy-once titles (Melvor, NGU, Lootun, Increlution) do not.

### (e) Mechanics worth prototyping first, ranked
1. **Gambit list with forecast + replay.** Rows = (target, ≤2 conditions, verb); first-true fires; fall-through on can't-execute; a pre-fight forecast ("survives 92% of sims; dies to Ghoul packs") and a scrubbable replay that highlights which row fired each turn. This is the legibility kill-test; if watching the replay doesn't make players say "my rule was wrong", nothing else matters.
2. **Program-size budget as the difficulty/progression knob** (Auto Rogue, NieR): slots, HUD info, and gear share one budget; unlocks expand it. Prototype with 4 rows and see if runs are still winnable and interesting.
3. **Three-tier exit** (descend/retreat-at-stairs = keep all, retreat mid-floor = keep 60%, death = keep 30%) with meta (gambit sets, bestiary knowledge, unlocked verbs) never lost.
4. **Event-trigger verbs and positioning verbs** (on-kill, on-hurt, when 3+ foes adjacent → retreat to corridor / pick up / use stairs). Test whether players discover combos (the FF12 Reverse+Decoy moment) within the first hour.
5. **Enemy traits that invalidate the default program per floor/biome** (reflects magic, steals gear, only hittable at range) so re-authoring is forced every few floors.
6. **Persistent knowledge as metagame:** bestiary auto-fills condition vocabulary ("Foe: Undead" only appears after you've met undead), Idle-Loops-style.
7. **Shipped presets + community sharing:** 3 class presets to copy from (DAO), 3 saveable sets per hero (Zodiac Age), copy-as-text export from day one.
8. **Secondary metrics with histograms per floor** (turns, HP lost, potions) à la Zachtronics — cheap replayability once the core works.
9. **Announced intent:** one-line hero log ("HP 31% → Potion (rule 2)") streaming during autoplay.
10. **Speed toggle and offline/idle sims** last — they only matter once the program is worth watching.

Lowest-confidence items: Godville/NGU death rules and Xenoblade specifics are from memory **[med/low]**; everything with a Steam number or a linked quote is **[high]**.
