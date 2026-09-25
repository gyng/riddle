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

/// The ladders, in the camp's order.
pub const KIT_SLOTS: [&str; 3] = ["weapon", "armour", "pack"];

/// Weapon steps: the class's starting arm, +1 per step, to +3. (Deviation from the contract's
/// 4–5 steps: a fourth, +4, doubled the fighter's blow at depth and broke the D33 wall without
/// its counter — the kitted FULL−D33 passed on 3–5 of 30 seeds, bar 3.)
pub const WEAPON_MULT: [u32; 3] = [1, 4, 9];
/// Armour steps: (kind, enchant) and their multiples. The top is mail +1 (4 armour): armour
/// subtracts from every blow, and a fifth point (mail +2) made the Deep's lurkers harmless —
/// the kitted FULL−D28 passed the Queen's wall on 11 of 30 seeds without her counter.
pub const ARMOUR_STEPS: [(&str, i32); 4] = [("leather", 0), ("leather", 1), ("mail", 0), ("mail", 1)];
pub const ARMOUR_MULT: [u32; 4] = [1, 4, 8, 14];
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

/// The price of step `i` of a ladder today.
pub fn price(l: &LineageState, slot: &str, i: usize) -> u32 {
    unit(l.best_depth) * mults(slot).get(i).copied().unwrap_or(0)
}

/// Cut 23 §1: a row slot's gold price on the forge's ladder (`None` for any other unlock).
pub fn row_gold(l: &LineageState, id: &str) -> Option<u32> {
    ROW_MULT.iter().find(|(r, _)| *r == id).map(|(_, m)| unit(l.best_depth) * m)
}

/// The night's net `nights` divides by (the last full night's, or tonight's when larger).
pub fn per_night(l: &LineageState) -> i32 {
    l.last_night_net.max(l.night_net)
}

fn s_short(l: &LineageState, price: u32) -> i64 {
    price as i64 - l.gold as i64
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
            let next = steps.get(n).map(|s| KitNext { label: s.label.clone(), price: s.price, affordable: l.gold >= s.price as i32, nights: nights(l, s.price as i64 - l.gold as i64), per_night: (short > 0 && per_night(l) > 0).then(|| per_night(l)), ..Default::default() });
            KitLadder { slot: (*slot).into(), owned: n as u32, steps, next }
        })
        .collect()
}

/// Buy the next step of `slot` (the ledger reads `forge <label>`).
pub fn buy(game: &mut Game, slot: &str) -> Result<(), String> {
    if !KIT_SLOTS.contains(&slot) {
        return Err("unknown slot".into());
    }
    let l = &mut game.lineage;
    let n = owned(l, slot) as usize;
    if n >= mults(slot).len() {
        return Err("top of the ladder".into());
    }
    let p = price(l, slot, n) as i32;
    if l.gold < p {
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
    let sims = crate::forecast::camp_sims(game, &rules);
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
        *g.panel_cache.borrow_mut() = game.panel_cache.borrow().clone();
        let a = crate::forecast::camp_panel(&g, &rules, sims);
        for (k, v) in g.panel_cache.into_inner() {
            crate::forecast::panel_insert(game, k, v);
        }
        let n = a.len().min(base.len());
        let (a, b) = (&a[..n], &base[..n]);
        let ind = |x: bool| if x { 1.0 } else { 0.0 };
        let reach = crate::forecast::paired(a, b, |r| ind(r.max_depth >= depth));
        next.depth = Some(depth);
        next.delta = Some(reach.delta);
        next.pm = Some(reach.pm);
        next.bank = Some(crate::forecast::paired(a, b, |r| ind(r.tier == crate::engine::ExitTier::Bank && !r.timed_out)).delta);
        next.death = Some(crate::forecast::paired(a, b, |r| ind(r.tier == crate::engine::ExitTier::Death)).delta);
    }
    out
}
