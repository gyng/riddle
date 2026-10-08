//! Cut 113 §3 (blind cohorts: "the 20m return had little", "4h and 8h returns read the same", "the 8h
//! return felt like losses I didn't choose"): each return carries a decision — one pick of three
//! (a package drill, Legacy, a forge step at cost), sized by the absence. Nothing punishes absence:
//! an untaken pick waits at camp, and the next return adds its minutes to it (the pick grows, never
//! lapses). Deterministic: the offers are drawn from the lineage seed and the absence that opened
//! the pick, never from the game's dice, so a lineage that never picks plays exactly as before.
use crate::engine::{Game, LineageState};
use crate::wire::{ReturnOffer, ReturnPick};
use serde::{Deserialize, Serialize};

/// The pick waiting at camp: the absence minutes behind it and the absence that opened it (its draw).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Pending {
    pub minutes: u64,
    pub absence: u32,
}

/// An absence shorter than this offers no pick (a send's one-second settle is no return).
pub const MIN_S: u64 = 10 * 60;
/// The size steps, in absence minutes: 20 m → 1, 1 h → 2, 4 h → 3, 8 h → 4, 16 h → 5.
pub const SIZE_MINUTES: [u64; 4] = [60, 240, 480, 960];
/// Per size: a drill's runs on a worn package, Legacy points, the forge step's discount (percent).
pub const DRILL_RUNS: [u32; 5] = [3, 6, 10, 16, 24];
pub const LEGACY_POINTS: [u32; 5] = [2, 4, 6, 9, 12];
pub const FORGE_OFF: [u32; 5] = [10, 20, 30, 40, 50];

/// The pick's size (1–5) for `minutes` away.
pub fn size(minutes: u64) -> usize {
    1 + SIZE_MINUTES.iter().filter(|m| minutes >= **m).count()
}

/// A return of `elapsed_s` seconds: open a pick, or grow the one waiting.
pub fn on_return(l: &mut LineageState, elapsed_s: u64) {
    if elapsed_s < MIN_S || l.ended {
        return;
    }
    let absence = l.absences;
    let p = l.return_pick.get_or_insert(Pending { minutes: 0, absence });
    p.minutes = p.minutes.saturating_add(elapsed_s / 60);
}

fn mix(seed: u64, absence: u32, salt: u64) -> u64 {
    let mut x = seed ^ (absence as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    x
}

/// The worn package the drill offers (seeded among those below the top level).
fn drill_target(l: &LineageState, p: &Pending) -> Option<String> {
    let worn: Vec<String> = l.pkg.equipped().into_iter().filter(|id| crate::packages::def(id).is_some() && l.pkg.level(id) < crate::packages::MAX_LEVEL).collect();
    (!worn.is_empty()).then(|| worn[(mix(l.seed, p.absence, 1) % worn.len() as u64) as usize].clone())
}

/// The forge ladder the pick discounts (seeded among those with a next step; none before the blacksmith).
fn forge_target(l: &LineageState, p: &Pending) -> Option<String> {
    if !crate::town::built(l, "blacksmith") {
        return None;
    }
    let open: Vec<&str> = crate::kit::KIT_SLOTS.iter().copied().filter(|s| (crate::kit::owned(l, s) as usize) < crate::kit::mults(s).len()).collect();
    (!open.is_empty()).then(|| open[(mix(l.seed, p.absence, 2) % open.len() as u64) as usize].to_string())
}

/// The three offers of the waiting pick (a fourth kind, marks, stands in when the drill or the forge has
/// nothing left to give).
pub fn offers(l: &LineageState, p: &Pending) -> Vec<ReturnOffer> {
    let s = size(p.minutes) - 1;
    let mut out = Vec::new();
    match drill_target(l, p) {
        Some(id) => out.push(ReturnOffer { id: "drill".into(), title: crate::packages::name(&id).into(), line: format!("+{} runs", DRILL_RUNS[s]), price: 0, available: true, target: Some(id) }),
        None => out.push(ReturnOffer { id: "marks".into(), title: "Marks".into(), line: format!("+◆{}", 1 + s as u32 / 2), price: 0, available: true, target: None }),
    }
    out.push(ReturnOffer { id: "legacy".into(), title: "Legacy".into(), line: format!("+{}", LEGACY_POINTS[s]), price: 0, available: true, target: None });
    match forge_target(l, p) {
        Some(slot) => {
            let n = crate::kit::owned(l, &slot) as usize;
            let full = crate::kit::price(l, &slot, n) as i32;
            let price = full - full * FORGE_OFF[s] as i32 / 100;
            let lean = l.kit_lean.get(&slot).map(String::as_str).or_else(|| crate::kit::default_branch(&slot, n));
            let label = lean.map_or_else(|| crate::kit::step_label(l, &slot, n), |b| crate::kit::branch_label(l, &slot, b));
            out.push(ReturnOffer { id: "forge".into(), title: label, line: format!("−{}%", FORGE_OFF[s]), price, available: crate::tree::purse(l) >= price, target: Some(slot) });
        }
        None if out[0].id != "marks" => out.push(ReturnOffer { id: "marks".into(), title: "Marks".into(), line: format!("+◆{}", 1 + s as u32 / 2), price: 0, available: true, target: None }),
        None => out.push(ReturnOffer { id: "legacy2".into(), title: "Legacy".into(), line: format!("+{}", LEGACY_POINTS[s] / 2 + 1), price: 0, available: true, target: None }),
    }
    out
}

/// The waiting pick as the camp and the report show it.
pub fn wire(l: &LineageState) -> Option<ReturnPick> {
    let p = l.return_pick.as_ref()?;
    Some(ReturnPick { minutes: p.minutes, size: size(p.minutes) as u32, offers: offers(l, p) })
}

/// Take offer `id` of the waiting pick; the pick closes. Returns the taken offer's tile line.
pub fn take(game: &mut Game, id: &str) -> Result<String, String> {
    let p = game.lineage.return_pick.clone().ok_or("no pick")?;
    let offer = offers(&game.lineage, &p).into_iter().find(|o| o.id == id).ok_or("not on offer")?;
    if !offer.available {
        return Err("not enough gold".into());
    }
    let s = size(p.minutes) - 1;
    let l = &mut game.lineage;
    match id {
        "drill" => {
            let pkg = offer.target.clone().ok_or("no package")?;
            let before = l.pkg.level(&pkg);
            *l.pkg.runs.entry(pkg.clone()).or_insert(0) += DRILL_RUNS[s];
            let after = l.pkg.level(&pkg);
            if after > before {
                l.pkg.news.push(format!("{} L{after}", crate::packages::name(&pkg)));
                crate::packages::recompile(l);
            }
        }
        "legacy" | "legacy2" => {
            crate::legacy::ensure(l);
            let n = if id == "legacy" { LEGACY_POINTS[s] } else { LEGACY_POINTS[s] / 2 + 1 };
            l.bloodline.as_mut().expect("bloodline").points += n;
        }
        "marks" => l.marks += 1 + s as u32 / 2,
        "forge" => {
            let slot = offer.target.clone().ok_or("no ladder")?;
            let lean = l.kit_lean.get(&slot).cloned();
            crate::kit::buy_branch_off(l, &slot, lean.as_deref(), FORGE_OFF[s])?;
            crate::tree::did(l, "forge");
        }
        _ => return Err("not on offer".into()),
    }
    game.lineage.return_pick = None;
    Ok(format!("{} {}", offer.title, offer.line))
}
