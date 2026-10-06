# Earned Legacy choices — 2026-10-06

From immutable earned seed1/3/5 towns, compare the same next8h with no upgrade,
health rank3, armour rank3, damage rank3 and all three rank3. Use only actual
upgradeHero offers/purchases and existing bloodline points; preserve gold,
supplies, hired workers, town and all slots. Each individual path costs18
Legacy from rank0; all three cost54 and is a different investment, not an
otherwise equal comparator. No forged resources or changed caps/prices.

Run shipping-default Steady/Bold for each prepared save, retain native
manifest input/executable/output hashes, and compare complete reports and
raw final saves against current real WASM. No-upgrade results must reproduce
the existing baseline exactly. Confirm point/price/rank accounting and source
immutability. Show the actual upgrade sheet and a resulting report. Record
fixture timings and observations, without treating three towns as a balance
verdict or altering shipping content/gates. No deployment.

Completed:15 prepared-state comparisons, each with Steady/Bold. All30 full
native reports and raw final saves exactly match current real WASM; all30
native case hashes and input hashes verified. No-upgrade controls reproduce
all six previous early-style results exactly. Initial points83/88/164; real
purchases consume18 per single capped path or54 for all, without gold spending.
Source towns remain byte-identical. Packs, buildings and workers unchanged.

| Seed | Legacy path | Cost | Steady depth / gold / deaths | Bold depth / gold / deaths | Bold new bosses |
| --- | --- | ---: | --- | --- | --- |
| 1 | none | 0 | D11 / $1219 / 0 | D18 / $5781 / 11 | boss: bloat_mother |
| 1 | health | 18 | D10 / $1048 / 0 | D16 / $3648 / 12 | boss: bloat_mother |
| 1 | armour | 18 | D13 / $2017 / 1 | D13 / $1143 / 12 | boss: bloat_mother |
| 1 | damage | 18 | D13 / $2108 / 1 | D19 / $7546 / 11 | boss: bloat_mother, boss: lich |
| 1 | all | 54 | D13 / $2316 / 1 | D19 / $6267 / 11 | boss: bloat_mother, boss: lich |
| 3 | none | 0 | D12 / $1267 / 1 | D13 / $0 / 12 | boss: bloat_mother |
| 3 | health | 18 | D13 / $1683 / 0 | D13 / $0 / 13 | boss: bloat_mother |
| 3 | armour | 18 | D13 / $1300 / 1 | D13 / $0 / 12 | boss: bloat_mother |
| 3 | damage | 18 | D13 / $1680 / 0 | D18 / $4660 / 11 | boss: bloat_mother |
| 3 | all | 54 | D13 / $1896 / 1 | D18 / $6170 / 9 | boss: bloat_mother |
| 5 | none | 0 | D13 / $2725 / 1 | D13 / $0 / 13 | — |
| 5 | health | 18 | D15 / $3815 / 0 | D13 / $0 / 13 | — |
| 5 | armour | 18 | D15 / $2745 / 2 | D13 / $0 / 12 | — |
| 5 | damage | 18 | D15 / $3949 / 0 | D18 / $0 / 12 | — |
| 5 | all | 54 | D19 / $6666 / 0 | D20 / $2377 / 8 | boss: lich |

In these states, damage is the useful single-path change for Bold in seed3:
D13/$0/12deaths becomes D18/$4660/11deaths. Health/armour still yield$0.
Seed5 damage reachesD18 but still returns$0; all upgrades reachesD20/$2377
with first Lich victory and8deaths/9runs. Seed3 all upgrades reachesD18/$6170
with9deaths/10runs. Seed1 damage improves toD19/$7546/Mother+Lich, while all
upgrades brings less gold ($6267). These investments matter; higher investment
does not guarantee a better deterministic trajectory. Bold remains high risk.
Steady full upgrades improves depth/gold in all three; individual paths vary,
and health in seed1 is lower than its unchanged control. No universal ranking,
low-attrition promise or general stance-balance verdict from three towns.

Warm native two-case batches0.54–1.29s, build0.04–0.08s. Actual results and
provenance under scratchpad/legacy-choice-20261006/; unchanged shipping WASM.
Real upgrade sheet and report screenshots captured. The sheet audit led to
UX_LEGACY_UPGRADE_FEEDBACK.md: clearer Rank/Next and distinct accessible actions,
plus refresh of points/offers/away status without waiting for a new hero.
No core/price/cap/balance/gate changes. Next: broader affected-stance checks
before any content edit; inspect away sustain-spending behavior against the
owner's absence contract, preserving spending protections until explicitly
justified by measured gameplay. No deployment.
