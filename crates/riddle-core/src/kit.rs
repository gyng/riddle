//! Cut 23 §1: the forge — the heir's starting kit bought with gold (both cohort-18 raters held
//! ~$2100 after the absence and nothing worth buying). Three ladders (weapon, armour, pack),
//! each step permanent for the lineage: every heir starts with every owned step, a death loses
//! none. Prices come from the lineage (`unit`: its best depth, as the shelf's are), so a step
//! costs about the same share of a night wherever the lineage is; the row slots sit on the
//! same ladder (`row_gold`).
use crate::engine::{Game, LineageState};
use crate::hero::Hero;
use crate::item::Item;
use crate::wire::{KitLadder, KitNext, KitStep};
use std::collections::BTreeMap;

/// The ladders, in the camp's order.
pub const KIT_SLOTS: [&str; 3] = ["weapon", "armour", "pack"];

/// Weapon steps: the class's starting arm, +1 per step, to +3. (Deviation from the contract's
/// 4–5 steps: a fourth, +4, doubled the fighter's blow at depth and broke the D33 wall without
/// its counter — the kitted FULL−D33 passed on 3–5 of 30 seeds, bar 3.)
/// Cut 30 (the idle-first pivot retired the "not engaging fails" rows that capped it: the blacksmith
/// is a multiplier the idle floor lacks): three more steps, each a harder blow (+1 damage).
pub const WEAPON_MULT: [u32; 6] = [1, 4, 9, 16, 25, 36];
/// Weapon steps past this many are damage, not aim.
pub const AIM_STEPS: i32 = 3;
/// Each weapon step past the aim adds this much to the blow.
pub const DMG_PER_STEP: i32 = 2;
/// Cut 25 §1 (AN: the forge moved bank more than any row): a weapon step is aim — this many
/// points on the 80 % to hit — not damage (`Hero::hit_pct`; the arm's blow is the class's own).
pub const AIM_PER_STEP: u32 = 4;
/// Armour steps: (kind, enchant) and their multiples. The top is mail +1 (4 armour): armour
/// subtracts from every blow, and a fifth point (mail +2) made the Deep's lurkers harmless —
/// the kitted FULL−D28 passed the Queen's wall on 11 of 30 seeds without her counter.
/// Cut 30: three more steps (mail +2, plate, plate +1), the forge a multiplier the idle floor lacks.
pub const ARMOUR_STEPS: [(&str, i32); 7] = [("leather", 0), ("leather", 1), ("mail", 0), ("mail", 1), ("mail", 2), ("plate", 0), ("plate", 1)];
pub const ARMOUR_MULT: [u32; 7] = [1, 4, 8, 14, 22, 32, 45];
/// Pack steps: one more supply on the shelf each (`LineageState::supply_cap`).
pub const PACK_MULT: [u32; 4] = [2, 5, 10, 16];
/// The shelf never holds more than this many supplies (the pack's ten slots less the kit and
/// room for what a run finds).
pub const SUPPLY_CAP_MAX: usize = 8;
/// Cut 23 §1: the row slots on the forge's ladder (`row5` … `row10`), multiples of `unit`.
pub const ROW_MULT: [(&str, u32); 6] = [("row5", 2), ("row6", 5), ("row7", 10), ("row8", 16), ("row9", 22), ("row10", 30)];

/// The kit's item ids (never loot: an exit neither salvages nor keeps them).
pub const WEAPON_ID: u32 = 1;
pub const ARMOUR_ID: u32 = 2;
pub fn is_kit_id(id: u32) -> bool {
    id == WEAPON_ID || id == ARMOUR_ID
}

/// The unit a step's price is a multiple of: `100 + 25 × best depth` ($300 at D8). A night of a
/// mostly-home cohort set at D8 nets ~$1000–3700, so the first step of each ladder is under a
/// night and the top steps are two or three.
pub fn unit(best_depth: u32) -> u32 {
    100 + 25 * best_depth
}

/// Cut 24 §3 (AL: leather +1 $1000 → $1100 → $1400, mail $2800 → $3600 as the best deepened —
/// "the gold I banked overnight chases a price that rises with my best"): the lineage's unit,
/// fixed the first time the forge is shown (`lock_unit`: a step owned, or the purse able to buy
/// a first step) — every step's price is then fixed for good; until then it is today's.
pub fn unit_of(l: &LineageState) -> u32 {
    l.kit_unit.unwrap_or_else(|| unit(l.best_depth))
}

/// Cut 24 §3: fix the unit (`LineageState::kit_unit`) once the forge is shown — a step owned, or
/// a first step (a ladder's cheapest) the purse can buy. Called where the purse or the kit moves
/// (an exit, a purchase).
pub fn lock_unit(l: &mut LineageState) {
    // QA on 0c6e126: the row slots' price is shown from the first camp — fixed at the first exit that has a best floor
    if l.row_unit.is_none() && l.best_depth >= 1 {
        l.row_unit = Some(l.kit_unit.unwrap_or_else(|| unit(l.best_depth)));
    }
    if l.kit_unit.is_some() {
        return;
    }
    let first = KIT_SLOTS.iter().map(|s| mults(s)[0]).min().unwrap_or(1) * unit(l.best_depth);
    // QA on 0c6e126 (qaZ: every forge price +10 % after an absence with nothing bought): the forge tile is carved at the first salvage
    // (`Lineage.forge`) and shows the ladders' prices from then — shown is fixed
    if KIT_SLOTS.iter().any(|s| owned(l, s) > 0) || l.gold >= first as i32 || !l.forge.is_empty() {
        l.kit_unit = Some(unit(l.best_depth));
    }
}

pub fn mults(slot: &str) -> &'static [u32] {
    match slot {
        "weapon" => &WEAPON_MULT,
        "armour" => &ARMOUR_MULT,
        "pack" => &PACK_MULT,
        _ => &[],
    }
}

/// Steps owned on a ladder.
pub fn owned(l: &LineageState, slot: &str) -> u32 {
    l.kit.get(slot).copied().unwrap_or(0).min(mults(slot).len() as u32)
}

/// The kit at step `i` (0-based) of a ladder: `sword +1`, `mail`, `pack 4`.
pub fn step_label(l: &LineageState, slot: &str, i: usize) -> String {
    match slot {
        "weapon" => format!("{} +{}", l.class.starting_weapon(), i + 1),
        "armour" => {
            let (k, e) = ARMOUR_STEPS[i.min(ARMOUR_STEPS.len() - 1)];
            if e > 0 {
                format!("{k} +{e}")
            } else {
                k.to_string()
            }
        }
        _ => format!("pack {}", base_cap(l) + i + 1),
    }
}

/// The price of step `i` of a ladder (fixed once the forge is shown: `unit_of`).
pub fn price(l: &LineageState, slot: &str, i: usize) -> u32 {
    unit_of(l) * mults(slot).get(i).copied().unwrap_or(0)
}

/// Cut 23 §1: a row slot's gold price on the forge's ladder (`None` for any other unlock).
pub fn row_gold(l: &LineageState, id: &str) -> Option<u32> {
    ROW_MULT.iter().find(|(r, _)| *r == id).map(|(_, m)| l.row_unit.unwrap_or_else(|| unit_of(l)) * m)
}

/// The night's net `nights` divides by (the last full night's, or tonight's when larger).
pub fn per_night(l: &LineageState) -> i32 {
    l.last_night_net.max(l.night_net)
}

fn s_short(l: &LineageState, price: u32) -> i64 {
    price as i64 - crate::tree::purse(l) as i64
}

/// The shelf's cap before the pack steps (3, or 5 with `supply_cap_5`).
pub fn base_cap(l: &LineageState) -> usize {
    if l.unlocks.contains("supply_cap_5") {
        5
    } else {
        3
    }
}

/// The nights of income until `short` coins are in the purse: the last full night's net
/// (`LineageState::last_night_net`), else none to go on.
pub fn nights(l: &LineageState, short: i64) -> Option<u32> {
    if short <= 0 {
        return Some(0);
    }
    let per = per_night(l);
    (per > 0).then(|| ((short + per as i64 - 1) / per as i64) as u32)
}

/// The ladders as the camp shows them (no sims).
pub fn ladders(l: &LineageState) -> Vec<KitLadder> {
    KIT_SLOTS
        .iter()
        .map(|slot| {
            let n = owned(l, slot) as usize;
            let steps: Vec<KitStep> = (0..mults(slot).len()).map(|i| KitStep { label: step_label(l, slot, i), price: price(l, slot, i), owned: i < n }).collect();
            let short = s_short(l, steps.get(n).map(|s| s.price).unwrap_or(0));
            let purse = crate::tree::purse(l);
            let next = steps.get(n).map(|s| KitNext { label: s.label.clone(), price: s.price, affordable: purse >= s.price as i32, nights: nights(l, s.price as i64 - purse as i64), per_night: (short > 0 && per_night(l) > 0).then(|| per_night(l)), ..Default::default() });
            KitLadder { slot: (*slot).into(), owned: n as u32, steps, next }
        })
        .collect()
}

/// Buy the next step of `slot` by hand (the ledger reads `forge <label>`; Cut 30.5: it counts toward the
/// apprentice).
pub fn buy(game: &mut Game, slot: &str) -> Result<(), String> {
    buy_step(&mut game.lineage, slot)?;
    crate::tree::did(&mut game.lineage, "forge");
    Ok(())
}

/// Buy the next step of `slot` from the purse (the apprentice's, and `buy`'s).
pub fn buy_step(l: &mut LineageState, slot: &str) -> Result<(), String> {
    buy_step_off(l, slot, 0)
}

/// `buy_step` at `off` percent off the price (Cut 30.5: the apprentice's rank).
pub fn buy_step_off(l: &mut LineageState, slot: &str, off: u32) -> Result<(), String> {
    if !KIT_SLOTS.contains(&slot) {
        return Err("unknown slot".into());
    }
    lock_unit(l);
    let n = owned(l, slot) as usize;
    if n >= mults(slot).len() {
        return Err("top of the ladder".into());
    }
    let p = price(l, slot, n) as i32;
    let p = p - p * off.min(100) as i32 / 100;
    if crate::tree::purse(l) < p {
        return Err("not enough gold".into());
    }
    let label = step_label(l, slot, n);
    l.gold_move(-p, &format!("forge {label}"));
    *l.kit.entry(slot.into()).or_insert(0) += 1;
    Ok(())
}

/// Every step of every ladder owned (the `KITTED` bot).
pub fn buy_all(l: &mut LineageState) {
    for slot in KIT_SLOTS {
        l.kit.insert(slot.into(), mults(slot).len() as u32);
    }
}

/// The heir's starting kit: the class arm at the weapon ladder's enchant, the armour step's
/// piece (none before the first step). Items with the kit's ids are never loot.
pub fn equip(l: &LineageState, hero: &mut Hero) {
    let mut w = Item::new(WEAPON_ID, l.class.starting_weapon());
    w.enchant = owned(l, "weapon") as i32;
    hero.auto_equip(w);
    let a = owned(l, "armour") as usize;
    if a > 0 {
        let (k, e) = ARMOUR_STEPS[a - 1];
        let mut it = Item::new(ARMOUR_ID, k);
        it.enchant = e;
        hero.auto_equip(it);
    }
}

/// The ladders with each next step measured (`Engine.kitDeltas`): the camp's panel for the
/// active set with the step owned, paired against the panel without (the same seeds, the
/// forecast's own sims), read at the floor its own reach is nearest half (≤ best + 1): the reach there and the
/// bank and death shares. Memoised like the cage tablet (the options' panels land in the
/// game's panel cache, keyed by the lineage fingerprint, which carries the kit).
pub fn deltas(game: &Game) -> Vec<KitLadder> {
    let rules = game.lineage.rules().clone();
    // Cut 25 §4: the first pass's sims (`forecast::option_sims`).
    let sims = crate::forecast::option_sims(game, &rules);
    let base = crate::forecast::camp_panel(game, &rules, sims);
    // the move is read where the set's own reach is nearest even (best + 1 sits under 1 %, so
    // every step read `≈`): the deepest floor the base reaches on about half its runs
    let reach_at = |d: u32| base.iter().filter(|r| r.max_depth >= d).count() as f64 / base.len().max(1) as f64;
    let depth = (2..=game.lineage.best_depth + 1)
        .min_by(|&x, &y| (reach_at(x) - 0.5).abs().total_cmp(&(reach_at(y) - 0.5).abs()).then(y.cmp(&x)))
        .unwrap_or(game.lineage.best_depth + 1);
    let mut out = ladders(&game.lineage);
    for lad in out.iter_mut() {
        let Some(next) = lad.next.as_mut() else { continue };
        let mut g = game.sim_clone();
        *g.lineage.kit.entry(lad.slot.clone()).or_insert(0) += 1;
        // Cut 25 §4: a pack step holds one more supply — measured holding it (the shelf's most
        // packed kind, one more), not empty (an empty slot is the same panel as the base, a
        // third of the forge's time for an exact 0).
        if lad.slot == "pack" {
            let mut n: BTreeMap<&str, usize> = BTreeMap::new();
            for it in &game.lineage.supplies {
                *n.entry(it.kind.as_str()).or_insert(0) += 1;
            }
            if let Some((kind, _)) = n.into_iter().max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0))) {
                let gold = g.lineage.gold;
                g.lineage.gold = i32::MAX / 2;
                let _ = g.buy_supply(kind);
                g.lineage.gold = gold;
            }
        }
        *g.panel_cache.borrow_mut() = game.panel_cache.borrow().clone();
        let a = crate::forecast::camp_panel(&g, &rules, sims);
        for (k, v) in g.panel_cache.into_inner() {
            crate::forecast::panel_insert(game, k, v);
        }
        let n = a.len().min(base.len());
        let (a, b) = (&a[..n], &base[..n]);
        let ind = |x: bool| if x { 1.0 } else { 0.0 };
        // Cut 25 §6 (AM: the forge `≈` on every item late): the move is read where it is clearest —
        // the floor whose paired move clears its ± by the most — else where the reach is nearest
        // half (the step moves nothing past its ±: `≈ ±N` there, honestly).
        let clear = (2..=game.lineage.best_depth + 1)
            .map(|d| (d, crate::forecast::paired(a, b, |r| ind(r.max_depth >= d))))
            .filter(|(_, m)| m.delta.abs() > m.pm + 1e-9)
            .max_by(|x, y| (x.1.delta.abs() - x.1.pm).total_cmp(&(y.1.delta.abs() - y.1.pm)).then(x.0.cmp(&y.0)));
        let (depth, reach) = clear.unwrap_or_else(|| (depth, crate::forecast::paired(a, b, |r| ind(r.max_depth >= depth))));
        next.depth = Some(depth);
        next.delta = Some(reach.delta);
        next.pm = Some(reach.pm);
        next.bank = Some(crate::forecast::paired(a, b, |r| ind(r.tier == crate::engine::ExitTier::Bank && !r.timed_out)).delta);
        next.death = Some(crate::forecast::paired(a, b, |r| ind(r.tier == crate::engine::ExitTier::Death)).delta);
    }
    out
}

/// Cut 29 §5: a commission's price in forge units, before the climb (`commission_price`).
pub const COMMISSION_UNITS: u32 = 10;

/// The works a lineage commissions, in order (then numbered): no sim reads them.
pub const WORKS: [&str; 8] = ["heir's statue", "camp hall", "chronicle wall", "boss trophies", "the forge's bell", "a banner", "the long table", "a lantern tower"];

/// Cut 29 §5: the next commission's price — 10 forge units × 1.25ⁿ (n the works built), in tens —
/// priced against income: never more than half a day's net (`LineageState::last_day_net`, fixed
/// through a day), nor under `COMMISSION_FLOOR` units. The climb alone outran the purse at a stall
/// (rater AS s1: $5959 held on day 5, a work at ~$5900 against a day's net of ~$2600); at a whole
/// day's net a purse holding tomorrow's oath beside it still sat over 1.5 days' net (AS s2: $3078,
/// a $2020 work, a $930 oath).
pub fn commission_price(l: &LineageState) -> i32 {
    let unit = unit_of(l) as f64;
    let climb = COMMISSION_UNITS as f64 * unit * 1.25f64.powi(l.works.len() as i32);
    let income = (l.last_day_net.max(0) as f64 / 2.0).max(COMMISSION_FLOOR as f64 * unit);
    (climb.min(income) / 10.0).round() as i32 * 10
}

/// The fewest forge units a commission costs (its price against a day's net: `commission_price`).
pub const COMMISSION_FLOOR: u32 = 5;

fn work_label(n: usize) -> String {
    match WORKS.get(n) {
        Some(w) => w.to_string(),
        None => format!("{} {}", WORKS[n % WORKS.len()], n / WORKS.len() + 1),
    }
}

pub fn commission_wire(l: &LineageState) -> crate::wire::Commission {
    let price = commission_price(l);
    crate::wire::Commission { price, label: work_label(l.works.len()), available: crate::tree::purse(l) >= price }
}

/// Cut 29 §5: commission the next work (policy-neutral: the gold's sink, the chronicle's line).
pub fn commission(game: &mut Game) -> Result<String, String> {
    let l = &mut game.lineage;
    lock_unit(l);
    let price = commission_price(l);
    if crate::tree::purse(l) < price {
        return Err("not enough gold".into());
    }
    let label = work_label(l.works.len());
    l.gold_move(-price, &format!("forge commission {label}"));
    l.works.push(label.clone());
    Ok(label)
}
