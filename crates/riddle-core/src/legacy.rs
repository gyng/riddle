//! Explicit inherited bloodline choices, independent of class XP and gold.
use crate::{engine::{Game, LineageState}, hero::Hero, wire::{HeroLegacy, LegacyUpgrade, BloodlineLegacy}};

/// Stable cosmetic identity; never consumes the game's random stream.
pub fn hero_name(seed: u64, heir: u32) -> &'static str {
    const NAMES: [&str; 24] = ["Alden", "Bryn", "Corin", "Dara", "Elian", "Fenn", "Galen", "Hale",
        "Iris", "Jora", "Kael", "Lark", "Maren", "Niall", "Orin", "Petra", "Quill", "Rook",
        "Sable", "Toren", "Una", "Vale", "Wren", "Yara"];
    let first = crate::rng::splitmix(seed) % NAMES.len() as u64;
    NAMES[((first + u64::from(heir.saturating_sub(1))) % NAMES.len() as u64) as usize]
}

/// The family belongs to the persistent slot; given names keep their old derivation.
/// Distinct complete names across active slots without depending on other heirs.
pub fn hero_identity(seed: u64, heir: u32, bloodline_id: u32) -> String {
    let family = match bloodline_id {1 => "Ash".into(), 2 => "Thorn".into(), 3 => "Flint".into(),
        id => format!("Wayfarer{id}")};
    format!("{} {family}", hero_name(seed, heir))
}

/// Cut 121 §1 (blind 1a7d834: both raters slew the King in the first session; Legacy ≈ 110 ◆ on day 1, the whole old
/// tree 216): a root takes `CAP` ranks on a rising curve (`ROOT_PRICES`, the rank's price), and the effects cost
/// `EFFECT_PRICES` by their depth — a spending player's power grows over the fortnight, not the first hours.
/// Cut 122 §1 (blind 9621b19, A and B: rank 1 cost 15, rank 2 600 — 388 ◆ sat unusable): a smooth curve, each rank
/// 2–3× the last, more ranks; a rank's effect is `root_effect`'s.
pub const CAP: u32 = 8;
pub const ROOT_PRICES: [u32; CAP as usize] = [15, 40, 110, 300, 800, 2000, 5000, 12000];
/// The effects' prices by their tier: the D8 ones, the D18 ones.
pub const EFFECT_PRICES: [u32; 2] = [700, 1800];

/// A root's price at `rank` (the rank owned; the next one's price), saturating past the cap.
pub fn root_price(rank: u32) -> u32 {
    ROOT_PRICES.get(rank as usize).copied().unwrap_or(u32::MAX)
}
/// Cut 122 §1: what a root's `rank` gives the hero — max hp (`health`), damage (`damage`), armour (`armour`).
pub fn root_effect(id: &str, rank: u32) -> i32 {
    let r = rank.min(CAP) as i32;
    match id {
        "health" => 3 * r,
        _ => r,
    }
}
pub const IDS: [&str; 3] = ["health", "damage", "armour"];

pub const RESTORATION:u16=1;
pub const MENDING:u16=2;
pub const RENEWAL:u16=4;
pub const CONTROL:u16=8;
pub const VENOM:u16=16;
pub const DEBILITATE:u16=32;
pub const CLEAR_LUNGS:u16=64;
pub const FIREWARD:u16=128;
pub const BRACE:u16=256;
struct Node {
    id:&'static str,name:&'static str,branch:&'static str,effect:&'static str,
    parent:Option<&'static str>,depth:u32,other:Option<&'static str>,mask:u16,price:u32,
}
const NODES:[Node;12]=[
    Node{id:"health",name:"Health",branch:"Recovery",effect:"+3 HP",parent:None,depth:0,other:None,mask:0,price:0},
    Node{id:"damage",name:"Damage",branch:"Control",effect:"+1 damage",parent:None,depth:0,other:None,mask:0,price:0},
    Node{id:"armour",name:"Armour",branch:"Warding",effect:"+1 armour",parent:None,depth:0,other:None,mask:0,price:0},
    Node{id:"restoration",name:"Restoration",branch:"Recovery",effect:"+1 HP per safe rest",parent:Some("health"),depth:8,other:None,mask:RESTORATION,price:EFFECT_PRICES[0]},
    Node{id:"control",name:"Control",branch:"Control",effect:"Stun and Slow +5 ticks",parent:Some("damage"),depth:8,other:None,mask:CONTROL,price:EFFECT_PRICES[0]},
    Node{id:"clear_lungs",name:"Clear lungs",branch:"Warding",effect:"Gas and poison damage −2",parent:Some("armour"),depth:8,other:None,mask:CLEAR_LUNGS,price:EFFECT_PRICES[0]},
    Node{id:"mending",name:"Mending",branch:"Recovery",effect:"Heal potions +25% HP",parent:Some("restoration"),depth:18,other:Some("renewal"),mask:MENDING,price:EFFECT_PRICES[1]},
    Node{id:"renewal",name:"Renewal",branch:"Recovery",effect:"Natural enemy kills +1 HP",parent:Some("restoration"),depth:18,other:Some("mending"),mask:RENEWAL,price:EFFECT_PRICES[1]},
    Node{id:"venom",name:"Venom",branch:"Control",effect:"Thrown poison +1 damage per pulse",parent:Some("control"),depth:18,other:Some("debilitate"),mask:VENOM,price:EFFECT_PRICES[1]},
    Node{id:"debilitate",name:"Debilitate",branch:"Control",effect:"Slow +15 further ticks",parent:Some("control"),depth:18,other:Some("venom"),mask:DEBILITATE,price:EFFECT_PRICES[1]},
    Node{id:"fireward",name:"Fireward",branch:"Warding",effect:"Fire damage halved",parent:Some("clear_lungs"),depth:18,other:Some("brace"),mask:FIREWARD,price:EFFECT_PRICES[1]},
    Node{id:"brace",name:"Brace",branch:"Warding",effect:"Below 25% HP: damage −2",parent:Some("clear_lungs"),depth:18,other:Some("fireward"),mask:BRACE,price:EFFECT_PRICES[1]},
];
pub fn has(hero:&Hero,mask:u16)->bool {hero.legacy_effects&mask!=0}
pub fn empty_effects(mask:&u16)->bool {*mask==0}
pub fn deepest(l:&LineageState)->u32 {
    l.hero_legacy.iter().map(|h|h.best_depth).max().unwrap_or(0).max(l.best_depth)
}

pub fn current(l: &LineageState) -> Option<&BloodlineLegacy> { l.bloodline.as_ref() }
pub fn ensure(l: &mut LineageState) {
    if l.bloodline.is_none() {
        let mut b = BloodlineLegacy::default();
        for h in l.hero_legacy.iter() {
            b.points += h.points; b.spent += h.spent;
            for (id, rank) in &h.upgrades { let r = b.upgrades.entry(id.clone()).or_default(); *r = (*r).max(*rank).min(CAP); }
        }
        l.bloodline = Some(b);
    }
    if l.hero_legacy.last().is_none_or(|h| h.heir != l.heir) {
        l.hero_legacy.push(HeroLegacy { heir: l.heir, best_depth: l.heir_best, class: l.class.name().into(), ..Default::default() });
    }
}
pub fn offers(l: &LineageState, away: bool) -> Vec<LegacyUpgrade> {
    let h = current(l);
    let rank_of=|id:&str|h.and_then(|h|h.upgrades.get(id)).copied().unwrap_or(0);
    let depth=deepest(l);
    NODES.iter().map(|node| {
        let rank=rank_of(node.id);let cap=if node.mask==0 {CAP}else{1};
        let price=if node.mask==0 {root_price(rank.min(CAP-1))}else{node.price};
        let blocked=if rank>=cap {Some("Complete".into())}
            else if !l.town.home.unwrap_or(true) {Some("Build a house".into())}
            else if depth<node.depth {Some(format!("Reach D{}",node.depth))}
            else if let Some(parent)=node.parent.filter(|id|rank_of(id)==0) {
                Some(format!("Buy {}",NODES.iter().find(|n|n.id==parent).expect("parent").name))
            } else if let Some(other)=node.other.filter(|id|rank_of(id)>0) {
                Some(format!("Chosen {}",NODES.iter().find(|n|n.id==other).expect("fork").name))
            } else if h.is_none_or(|h|h.points<price) {Some("More Legacy needed".into())}
            else {None};
        // blind b58b431: away, an upgrade the points buy is for the next run (`buy_next`)
        let next_run=away&&blocked.is_none();
        let blocked=if next_run {Some("Hero away".into())} else {blocked};
        LegacyUpgrade {id:node.id.into(),rank,cap,price,effect:node.effect.into(),affordable:blocked.is_none(),next_run,
            name:Some(node.name.into()),branch:Some(node.branch.into()),parent:node.parent.map(str::to_owned),
            min_depth:(node.depth>0).then_some(node.depth),blocked,
            owned_effect:(node.mask==0&&rank>0).then(||match node.id {
                "health"=>format!("+{} HP",root_effect("health",rank)),
                "damage"=>format!("+{} damage",root_effect("damage",rank)),
                _=>format!("+{} armour",root_effect("armour",rank)),
            })}
    }).collect()
}
/// The hero is away: a run under way (or ended, its exit pending). Blind ad71e72 (B: Legacy piled
/// to 80, "could not buy"): the camp rest's clock readies the next run at its first tick
/// (`Game::step` → `ensure_run`) while the hero still rests at home — that run is no absence
/// until the rest is out or a send skips it.
pub fn away(g: &Game) -> bool {
    g.run.as_ref().is_some_and(|r| r.turn > 0 || r.over.is_some() || g.lineage.rest_left == 0)
}
/// A run readied by the rest clock but not begun carries the bloodline as it stands: its hero and
/// the camp state it left from (the verdict's replays) take a purchase or a respec made since.
fn refit(g: &mut Game, before: &BloodlineLegacy) {
    if away(g) { return; }
    let Some(run) = g.run.as_mut() else { return };
    let after = g.lineage.bloodline.clone().unwrap_or_default();
    let rank = |b: &BloodlineLegacy, id: &str| root_effect(id, b.upgrades.get(id).copied().unwrap_or(0));
    let hero = &mut run.hero;
    let hp = rank(&after, "health") - rank(before, "health");
    hero.max_hp += hp; hero.max_hp_base += hp; hero.hp = (hero.hp + hp).clamp(1, hero.max_hp.max(1));
    hero.str_bonus += rank(&after, "damage") - rank(before, "damage");
    hero.legacy_armour = rank(&after, "armour");
    hero.legacy_effects = NODES.iter().filter(|n| after.upgrades.get(n.id).is_some_and(|r| *r > 0)).fold(0, |mask, n| mask | n.mask);
    if let Some(sent) = g.sent_state.as_mut() { sent.lineage.bloodline = Some(after); }
}
pub fn buy(g: &mut Game, id: &str) -> Result<(), String> {
    if away(g) || !g.lineage.town.home.unwrap_or(true) { return Err("hero away".into()); }
    purchase(g, id)
}
/// Blind b58b431 (A: Legacy 146 unspendable — with the scout sending him, the hero is nearly always
/// away, and a purchase waited for a rest the player never saw): while he is away a purchase is
/// for the next run — the bloodline takes it now, the run under way keeps the hero it sent
/// (`refit` leaves an away run alone; `apply` gives the next heir it at his send).
pub fn buy_next(g: &mut Game, id: &str) -> Result<(), String> {
    if !away(g) { return buy(g, id); }
    if !g.lineage.town.home.unwrap_or(true) { return Err("build a house".into()); }
    purchase(g, id)
}
fn purchase(g: &mut Game, id: &str) -> Result<(), String> {
    let before = g.lineage.bloodline.clone().unwrap_or_default();
    let offer = offers(&g.lineage, false).into_iter().find(|u| u.id == id).ok_or("unknown upgrade")?;
    if offer.rank >= offer.cap { return Err("upgrade complete".into()); }
    if !offer.affordable { return Err(offer.blocked.unwrap_or_else(||"more Legacy needed".into())); }
    let spent=current(&g.lineage).expect("affordable bloodline").spent.checked_add(offer.price).ok_or("Legacy total overflow")?;
    ensure(&mut g.lineage);
    let h = g.lineage.bloodline.as_mut().expect("bloodline");
    h.points -= offer.price;
    h.spent = spent;
    h.upgrades.insert(id.into(), offer.rank + 1);
    refit(g, &before);
    Ok(())
}
fn respec_points(l:&LineageState,away:bool)->Result<u32,String> {
    if away {return Err("hero away".into());}
    if !l.town.home.unwrap_or(true) {return Err("build a house".into());}
    let h=current(l).ok_or("no upgrades")?;
    if h.upgrades.is_empty()&&h.spent==0 {return Err("no upgrades".into());}
    h.points.checked_add(h.spent).ok_or_else(||"Legacy total overflow".into())
}
pub fn respec_offer(l:&LineageState,away:bool)->crate::wire::LegacyRespec {
    let result=respec_points(l,away);
    crate::wire::LegacyRespec{refund:current(l).map_or(0,|h|h.spent),available:result.is_ok(),
        points_after:result.as_ref().ok().copied(),blocked:result.err()}
}
pub fn respec(g:&mut Game)->Result<(),String> {
    let points=respec_points(&g.lineage,away(g))?;
    let before=g.lineage.bloodline.clone().unwrap_or_default();
    let h=g.lineage.bloodline.as_mut().expect("bloodline");
    h.points=points;h.spent=0;h.upgrades.clear();
    refit(g,&before);Ok(())
}
pub fn apply(l: &LineageState, hero: &mut Hero) {
    if let Some(h) = current(l) {
        let rank = |id: &str| root_effect(id, h.upgrades.get(id).copied().unwrap_or(0));
        let hp = rank("health");
        hero.max_hp += hp; hero.max_hp_base += hp; hero.hp += hp;
        hero.str_bonus += rank("damage");
        hero.legacy_armour = rank("armour");
        hero.legacy_effects=NODES.iter().filter(|n|h.upgrades.get(n.id).is_some_and(|rank|*rank>0)).fold(0,|mask,n|mask|n.mask);
    }
}

#[cfg(test)]
#[path="legacy_tests.rs"]
mod tests;
