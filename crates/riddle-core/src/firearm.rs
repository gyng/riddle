//! Firearm profiles, per-item chambers and scheduled reload commitment.
//! The AI reserves chambers before resolving ordinary ranged hits.
use crate::geom::Pos;
use serde::{Deserialize, Serialize};

pub const LONG_GUN: crate::defs::ItemDef = crate::defs::ItemDef {
    kind: "long_gun", cat: crate::defs::Cat::Weapon, a: (6, 10), speed: 0,
    ranged: true, value: 35, benevolent: true, weight: 0,
};
pub const SHORT_GUN: crate::defs::ItemDef = crate::defs::ItemDef {
    kind: "short_gun", cat: crate::defs::Cat::Weapon, a: (4, 7), speed: 0,
    ranged: true, value: 35, benevolent: true, weight: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub capacity: u8,
    pub range: i32,
    pub damage: (i32, i32),
    pub armour_piercing: i32,
    pub reload_ticks: u32,
}

#[cfg(test)]
#[path = "firearm_combat_tests.rs"]
mod combat_tests;
#[cfg(test)]
#[path="gunner_tests.rs"]
mod class_tests;

impl Profile {
    pub fn of(kind: &str) -> Option<Self> {
        match kind {
            "long_gun" => Some(Self { capacity: 1, range: 8, damage: LONG_GUN.a, armour_piercing: 2, reload_ticks: 20 }),
            "short_gun" => Some(Self { capacity: 2, range: 3, damage: SHORT_GUN.a, armour_piercing: 0, reload_ticks: 15 }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Chambers {
    pub loaded: u8,
    /// Absolute run tick. Save/load and slice boundaries cannot shorten it;
    /// scheduler integration must stop batches at this deadline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reload_at: Option<u32>,
}

/// One reload commitment per hero; the deadline also lives on the actual item.
/// This lets quiet batches stop at completion without scanning inventories.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reload {
    pub item: u32,
    pub at: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub duration: u32,
}
fn is_zero(n: &u32) -> bool { *n == 0 }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Aim { pub item:u32, pub target:u32, pub from:Pos }
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skills {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub aim:Option<Aim>,
    #[serde(default)]
    pub smoke_ready:u32,
    #[serde(default)]
    pub fast_ready:u32,
    #[serde(default)]
    pub finish_ready:u32,
}
pub const SMOKE_COOLDOWN:u32=90;
pub const FAST_COOLDOWN:u32=100;
pub const FINISH_COOLDOWN:u32=60;
/// Earned progression, action art and catch-up acceptance permit the paid class offer.
pub const UI_READY:bool=true;
pub fn starting_kind(l:&crate::engine::LineageState)->&'static str {
    if l.class==crate::hero::Class::Gunner&&l.kit.get("gun_choice")==Some(&1)&&l.kit.get("short_gun")==Some(&1) {"short_gun"}
    else {l.class.starting_weapon()}
}
pub const SIDEARM_SLOT:&str="gun_sidearm";
pub fn sidearm_selected(l:&crate::engine::LineageState)->bool {
    l.kit.get(SIDEARM_SLOT)==Some(&1)&&l.kit.get("gun_sidearm_off")!=Some(&1)
}
pub fn set_sidearm(game:&mut crate::Game,selected:bool)->Result<(),String> {
    if game.lineage.class!=crate::hero::Class::Gunner||!game.lineage.unlocks.contains("gunner") {return Err("choose Gunner".into());}
    if game.run.is_some() {return Err("hero away".into());}
    if !game.lineage.town.home.unwrap_or(true) {return Err("build a house".into());}
    if game.lineage.kit.get(SIDEARM_SLOT)!=Some(&1) {return Err("forge melee backup".into());}
    if selected {game.lineage.kit.remove("gun_sidearm_off");}else{game.lineage.kit.insert("gun_sidearm_off".into(),1);}Ok(())
}
pub fn forge_sidearm(game:&mut crate::Game)->Result<(),String> {
    if game.lineage.class!=crate::hero::Class::Gunner||!game.lineage.unlocks.contains("gunner") {return Err("choose Gunner".into());}
    if game.run.is_some() {return Err("hero away".into());}
    if !game.lineage.town.home.unwrap_or(true) {return Err("build a house".into());}
    if game.lineage.kit.get(SIDEARM_SLOT)==Some(&1) {return Err("already forged".into());}
    let price=i32::try_from(crate::kit::unit_of(&game.lineage)).map_err(|_|"price too high")?;
    if crate::tree::purse(&game.lineage)<price {return Err("not enough gold".into());}
    crate::kit::lock_unit(&mut game.lineage);
    game.lineage.gold_move(-price,"forge melee backup");
    game.lineage.kit.insert(SIDEARM_SLOT.into(),1);
    crate::tree::did(&mut game.lineage,"forge");Ok(())
}
pub fn sidearm_ladder(l:&crate::engine::LineageState)->Option<crate::wire::KitLadder> {
    if l.class!=crate::hero::Class::Gunner||!l.unlocks.contains("gunner") {return None;}
    let owned=u32::from(l.kit.get(SIDEARM_SLOT)==Some(&1));let price=crate::kit::unit_of(l);
    let mut item=crate::item::Item::new(crate::kit::SIDEARM_ID,"sword");item.enchant=crate::kit::owned(l,"weapon") as i32;
    let label=if item.enchant>0 {format!("sword +{}",item.enchant)}else{"sword".into()};
    let next=(owned==0).then(||crate::wire::KitNext{label:label.clone(),price,
        affordable:l.town.home.unwrap_or(true)&&i64::from(crate::tree::purse(l))>=i64::from(price),
        nights:crate::kit::nights(l,i64::from(price)-i64::from(crate::tree::purse(l))),..Default::default()});
    Some(crate::wire::KitLadder{slot:SIDEARM_SLOT.into(),selected:Some(sidearm_selected(l)),owned,steps:vec![crate::wire::KitStep{
        label,price,owned:owned==1,kind:Some("sword".into()),rarity:crate::item::rarity(&item,true),branch:None}],next,branches:Vec::new()})
}
pub fn choose(game:&mut crate::Game,kind:&str)->Result<(),String> {
    if Profile::of(kind).is_none() {return Err("unknown gun".into());}
    if game.lineage.class!=crate::hero::Class::Gunner||!game.lineage.unlocks.contains("gunner") {return Err("choose Gunner".into());}
    if game.run.is_some() {return Err("hero away".into());}
    if !game.lineage.town.home.unwrap_or(true) {return Err("build a house".into());}
    let price=if kind=="short_gun"&&game.lineage.kit.get("short_gun")!=Some(&1) {crate::kit::unit_of(&game.lineage)}else{0};
    let price=i32::try_from(price).map_err(|_|"price too high")?;
    if crate::tree::purse(&game.lineage)<price {return Err("not enough gold".into());}
    if price>0 {
        crate::kit::lock_unit(&mut game.lineage);
        game.lineage.gold_move(-price,"forge scattergun");
        game.lineage.kit.insert("short_gun".into(),1);
        crate::tree::did(&mut game.lineage,"forge");
    }
    if kind=="short_gun" {game.lineage.kit.insert("gun_choice".into(),1);}else {game.lineage.kit.remove("gun_choice");}
    Ok(())
}

pub fn row(l:&crate::engine::LineageState)->Option<crate::rules::Row> {
    (l.class==crate::hero::Class::Gunner).then(||
        crate::rules::Row::new(vec![],crate::rules::Verb::new("gunner_tactic")).from("class:gunner"))
}

pub fn offers(l:&crate::engine::LineageState,away:bool)->Vec<crate::wire::GunOffer> {
    if l.class!=crate::hero::Class::Gunner||!l.unlocks.contains("gunner") {return Vec::new();}
    ["long_gun","short_gun"].into_iter().map(|kind| {
        let p=Profile::of(kind).unwrap();let owned=kind=="long_gun"||l.kit.get("short_gun")==Some(&1);
        let price=if owned {0}else{crate::kit::unit_of(l)};
        let blocked=if away {Some("hero away".into())}else if !l.town.home.unwrap_or(true) {Some("build a house".into())}
            else if i64::from(crate::tree::purse(l))<i64::from(price) {Some("not enough gold".into())}else{None};
        let mut weapon=crate::item::Item::new(crate::kit::WEAPON_ID,kind);weapon.enchant=crate::kit::owned(l,"weapon") as i32;
        let mut hero=crate::hero::Hero::new(crate::hero::Class::Gunner,Pos::new(0,0));
        hero.apply_level(l.class_level());crate::legacy::apply(l,&mut hero);
        hero.weapon=Some(weapon).into();
        crate::wire::GunOffer {kind:kind.into(),selected:starting_kind(l)==kind,owned,price,available:blocked.is_none(),blocked,
            capacity:p.capacity,range:p.range,damage:hero.atk(),armour_piercing:p.armour_piercing,reload_ticks:p.reload_ticks}
    }).collect()
}
pub fn cancel_aim(run:&mut crate::engine::Run,cx:&mut crate::engine::Ctx) {
    if run.gun_skills.as_mut().is_some_and(|s|s.aim.take().is_some()) {
        crate::chronicle::callout(run,cx,"aim lost");
    }
}
/// Validate prepared aim only at an action's existing visibility pass, never
/// by adding enemy scans to the tick scheduler.
pub fn on_view(run:&mut crate::engine::Run,cx:&mut crate::engine::Ctx,v:&crate::turn::View) {
    let Some(aim)=run.gun_skills.as_ref().and_then(|s|s.aim) else {return;};
    if run.hero.pos!=aim.from || run.hero.weapon.as_ref().is_none_or(|w|w.id!=aim.item) ||
        !v.foes.iter().any(|&i|run.monsters[i].id==aim.target&&run.floor.map.los(run.hero.pos,run.monsters[i].pos)&&run.hero.pos.cheb(run.monsters[i].pos)<=8) {
        cancel_aim(run,cx);
    }
}

pub fn reload_fast(run:&mut crate::engine::Run,cx:&mut crate::engine::Ctx)->bool {
    if !crate::hero::class_has_verb(run.hero.class,run.hero.level,"fast_reload") ||
        run.gun_skills.as_ref().is_some_and(|s|s.fast_ready>run.turn) {return false;}
    let Some(ready)=run.turn.checked_add(FAST_COOLDOWN) else {return false;};
    if !reload_at_speed(run,cx,true) {return false;}
    run.gun_skills.get_or_insert_with(Default::default).fast_ready=ready;
    true
}

pub fn reload(run: &mut crate::engine::Run, cx: &mut crate::engine::Ctx) -> bool {
    reload_at_speed(run,cx,false)
}
fn reload_at_speed(run:&mut crate::engine::Run,cx:&mut crate::engine::Ctx,fast:bool)->bool {
    if run.gun_reload.is_some() { return false; }
    let Some(weapon) = run.hero.weapon.as_ref() else { return false; };
    let Some(mut profile) = Profile::of(&weapon.kind) else { return false; };
    if fast {profile.reload_ticks=profile.reload_ticks.div_ceil(2);}
    let Some(mut chambers) = weapon.firearm else { return false; };
    let Ok(at) = chambers.reload(profile, run.turn) else { return false; };
    let item = weapon.id;
    run.hero.weapon.as_mut().unwrap().firearm = Some(chambers);
    run.gun_reload = Some(Reload { item, at, duration: profile.reload_ticks });
    crate::chronicle::callout(run, cx, "reloading");
    true
}

/// At the actual tick boundary, before hero actions. Stowing the gun preserves
/// its commitment. Completion emits once; ordinary items incur no item scan.
pub fn tick(run: &mut crate::engine::Run, cx: &mut crate::engine::Ctx) {
    let Some(reload) = run.gun_reload.filter(|r| run.turn >= r.at) else { return; };
    run.gun_reload = None;
    let completed = if run.hero.weapon.as_ref().is_some_and(|w| w.id == reload.item) {
        complete_item(run.hero.weapon.as_mut().unwrap(), run.turn)
    } else if let Some(index) = run.hero.inv.iter().position(|w| w.id == reload.item) {
        complete_item(&mut run.hero.inv[index], run.turn)
    } else if let Some(w) = run.bow_swap.as_mut().filter(|w| w.id == reload.item) {
        complete_item(w, run.turn)
    } else if let Some(w) = run.items.iter_mut().find(|w| w.item.id == reload.item) {
        complete_item(&mut w.item, run.turn)
    } else {
        // A thief may take a reloading gun; its physical chambers keep time.
        run.monsters.iter_mut().filter_map(|m| m.stolen.as_mut())
            .find(|w| w.id == reload.item).is_some_and(|w| complete_item(w, run.turn))
    };
    if completed { crate::chronicle::callout(run, cx, "loaded"); }
}

fn complete_item(item: &mut crate::item::Item, now: u32) -> bool {
    let Some(profile) = Profile::of(&item.kind) else { return false; };
    item.firearm.as_mut().is_some_and(|c| c.complete(profile, now))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal { Reloading, Empty, Full, InvalidTarget, Range, Sight, Burst, Clock }

/// Already observed target facts. No remembered-target mutation occurs until
/// this validation and chamber commitment both succeed in the action caller.
#[derive(Clone, Copy, Debug)]
pub struct ShotTarget {
    pub from: Pos,
    pub to: Pos,
    pub hostile_alive: bool,
    pub visible: bool,
    pub los: bool,
}

impl Chambers {
    pub fn loaded(profile: Profile) -> Self { Self { loaded: profile.capacity, reload_at: None } }

    pub fn reload(&mut self, profile: Profile, now: u32) -> Result<u32, Refusal> {
        if self.reload_at.is_some() { return Err(Refusal::Reloading); }
        if self.loaded >= profile.capacity { return Err(Refusal::Full); }
        let at = now.checked_add(profile.reload_ticks).ok_or(Refusal::Clock)?;
        self.reload_at = Some(at);
        Ok(at)
    }

    /// True exactly once, at/after the deadline, for the completion event.
    pub fn complete(&mut self, profile: Profile, now: u32) -> bool {
        if self.reload_at.is_some_and(|at| now >= at) {
            self.loaded = profile.capacity;
            self.reload_at = None;
            return true;
        }
        false
    }

    /// Reserve chambers before any hit/miss/reflection RNG. An invalid request
    /// leaves all state exact; a valid miss still spends its reserved chamber.
    pub fn fire(&mut self, profile: Profile, target: ShotTarget, burst: bool) -> Result<(), Refusal> {
        if !target.hostile_alive || target.from == target.to { return Err(Refusal::InvalidTarget); }
        if target.from.cheb(target.to) > profile.range { return Err(Refusal::Range); }
        if !target.visible || !target.los { return Err(Refusal::Sight); }
        if self.reload_at.is_some() { return Err(Refusal::Reloading); }
        if burst && profile.capacity != 2 { return Err(Refusal::Burst); }
        let cost = if burst { 2 } else { 1 };
        if self.loaded < cost { return Err(Refusal::Empty); }
        self.loaded -= cost;
        Ok(())
    }
}

/// Cheap geometric filter for short-gun secondary targets. The caller supplies
/// only live, hostile, visible, LOS candidates, and caps the total at three.
/// A 90-degree forward cone centred on the actual primary shot, within range,
/// with secondary targets adjacent to the primary (not the shooter).
pub fn in_spread(from: Pos, primary: Pos, candidate: Pos) -> bool {
    if from == primary || from == candidate || candidate == primary ||
        from.cheb(candidate) > 3 || !primary.adjacent(candidate) { return false; }
    let ax = i64::from(primary.x) - i64::from(from.x);
    let ay = i64::from(primary.y) - i64::from(from.y);
    let bx = i64::from(candidate.x) - i64::from(from.x);
    let by = i64::from(candidate.y) - i64::from(from.y);
    let dot = ax * bx + ay * by;
    let cross = ax * by - ay * bx;
    dot > 0 && cross.abs() <= dot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;
    fn target(distance: i32) -> ShotTarget {
        ShotTarget { from: Pos::new(0, 0), to: Pos::new(distance, 0), hostile_alive: true, visible: true, los: true }
    }
    #[test]
    fn exact_ranges_and_failed_shots_preserve_chambers() {
        for (kind, range) in [("long_gun", 8), ("short_gun", 3)] {
            let p = Profile::of(kind).unwrap();
            let mut c = Chambers::loaded(p);
            let original = c;
            assert_eq!(c.fire(p, target(range + 1), false), Err(Refusal::Range));
            for invalid in [ShotTarget { los: false, ..target(range) }, ShotTarget { visible: false, ..target(range) }, ShotTarget { hostile_alive: false, ..target(range) }, target(0)] {
                assert!(c.fire(p, invalid, false).is_err());
                assert_eq!(c, original);
            }
            assert_eq!(c.fire(p, target(range), false), Ok(()));
            assert_eq!(c.loaded, p.capacity - 1);
        }
    }
    #[test]
    fn exact_reload_boundaries_refuse_fire_and_repeated_reload() {
        for kind in ["long_gun", "short_gun"] {
            let p = Profile::of(kind).unwrap();
            let mut c = Chambers::loaded(p);
            assert_eq!(c.reload(p, 40), Err(Refusal::Full));
            c.fire(p, target(1), false).unwrap();
            let deadline = c.reload(p, 40).unwrap();
            let pending = c;
            assert_eq!(deadline, 40 + p.reload_ticks);
            assert_eq!(c.reload(p, 41), Err(Refusal::Reloading));
            assert_eq!(c.fire(p, target(1), false), Err(Refusal::Reloading));
            assert!(!c.complete(p, deadline - 1));
            assert_eq!(c, pending);
            assert!(c.complete(p, deadline));
            assert_eq!(c.loaded, p.capacity);
            assert!(!c.complete(p, deadline + 1));
        }
    }
    #[test]
    fn weapon_instances_keep_independent_chambers_across_swap_and_save() {
        let mut long = Item::new(101, "long_gun");
        let mut short = Item::new(102, "short_gun");
        let lp = Profile::of(&long.kind).unwrap();
        let sp = Profile::of(&short.kind).unwrap();
        long.firearm.as_mut().unwrap().fire(lp, target(1), false).unwrap();
        short.firearm.as_mut().unwrap().fire(sp, target(1), false).unwrap();
        long.firearm.as_mut().unwrap().reload(lp, 100).unwrap();
        let mut worn = Some(long);
        let stowed = worn.replace(short).unwrap();
        let text = serde_json::to_string(&(worn, stowed)).unwrap();
        let (mut worn, stowed): (Option<Item>, Item) = serde_json::from_str(&text).unwrap();
        let short = worn.replace(stowed).unwrap();
        assert_eq!(short.firearm.unwrap().loaded, 1);
        let c = worn.as_mut().unwrap().firearm.as_mut().unwrap();
        assert_eq!(c.loaded, 0);
        assert_eq!(c.reload_at, Some(120));
        assert!(!c.complete(lp, 119));
        assert!(c.complete(lp, 120));
    }
    #[test]
    fn slice_and_serialization_boundaries_do_not_shorten_reload() {
        let p = Profile::of("short_gun").unwrap();
        let mut whole = Chambers::loaded(p);
        whole.fire(p, target(1), true).unwrap();
        whole.reload(p, 80).unwrap();
        let mut sliced = whole;
        for now in [81, 84, 90, 94] {
            assert!(!sliced.complete(p, now));
            sliced = serde_json::from_str(&serde_json::to_string(&sliced).unwrap()).unwrap();
        }
        assert!(whole.complete(p, 95));
        assert!(sliced.complete(p, 95));
        assert_eq!(sliced, whole);
    }
    #[test]
    fn burst_requires_both_short_chambers_and_clock_overflow_is_atomic() {
        let lp = Profile::of("long_gun").unwrap();
        let sp = Profile::of("short_gun").unwrap();
        let mut long = Chambers::loaded(lp);
        assert_eq!(long.fire(lp, target(1), true), Err(Refusal::Burst));
        assert_eq!(long.loaded, 1);
        let mut short = Chambers::loaded(sp);
        short.fire(sp, target(1), false).unwrap();
        let original = short;
        assert_eq!(short.fire(sp, target(1), true), Err(Refusal::Empty));
        assert_eq!(short.reload(sp, u32::MAX), Err(Refusal::Clock));
        assert_eq!(short, original);
    }
    #[test]
    fn spread_is_forward_local_and_bounded_by_short_range() {
        let from = Pos::new(0, 0);
        let primary = Pos::new(2, 0);
        for pos in [Pos::new(2, 1), Pos::new(3, -1), Pos::new(1, 1)] { assert!(in_spread(from, primary, pos)); }
        for pos in [from, primary, Pos::new(-1, 0), Pos::new(4, 0), Pos::new(1, 2)] { assert!(!in_spread(from, primary, pos)); }
        assert!(!in_spread(from, from, Pos::new(1, 0)));
    }
    #[test]
    fn ordinary_item_roundtrip_omits_firearm_and_bows_are_not_guns() {
        assert!(Profile::of("bow").is_none());
        let item = Item::new(9, "bow");
        let text = serde_json::to_string(&item).unwrap();
        assert!(!text.contains("firearm"));
        let old: Item = serde_json::from_str(&text).unwrap();
        assert_eq!(old, item);
        assert_eq!(serde_json::to_string(&old).unwrap(), text);
        for kind in ["long_gun", "short_gun"] {
            let gun = Item::new(10, kind);
            let profile = Profile::of(kind).unwrap();
            assert_eq!(gun.def().kind, kind);
            assert_eq!(gun.cat(), crate::defs::Cat::Weapon);
            assert_eq!(gun.atk(), profile.damage);
            assert_eq!(gun.def().weight, 0);
            assert!(!crate::defs::ITEMS.iter().any(|d| d.kind == kind));
        }
    }
}

/// Allocation-free comparison at real tick boundaries. Countdown is derived
/// from the absolute deadline, so quiet ticks emit no duplicate state events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ObservedGun {
    item:u32, short:bool, loaded:u8, damage:(i32,i32),
    reload_until:Option<u32>, reload_ticks:u32, aiming:bool,
}
impl ObservedGun {
    pub(crate) fn of(run:&crate::engine::Run)->Option<Self> {
        let w=run.hero.weapon.as_ref()?;
        let chambers=w.firearm?;
        let p=Profile::of(&w.kind)?;
        let reload=run.gun_reload.filter(|r|r.item==w.id);
        Some(Self {item:w.id,short:w.kind=="short_gun",loaded:chambers.loaded,
            damage:run.hero.atk(),reload_until:reload.map(|r|r.at),
            reload_ticks:reload.filter(|r|r.duration>0).map_or(p.reload_ticks,|r|r.duration),
            aiming:run.gun_skills.as_ref().and_then(|s|s.aim).is_some_and(|a|a.item==w.id),
        })
    }
    pub(crate) fn snapshot(self,turn:u32)->crate::wire::GunSnap {
        let kind=if self.short {"short_gun"}else{"long_gun"};
        let p=Profile::of(kind).unwrap();
        crate::wire::GunSnap {item:self.item,kind:kind.into(),loaded:self.loaded,
            capacity:p.capacity,range:p.range,damage:self.damage,
            armour_piercing:p.armour_piercing,reload_ticks:self.reload_ticks,
            reload_left:self.reload_until.map_or(0,|at|at.saturating_sub(turn)),
            reload_until:self.reload_until,aiming:self.aiming,
        }
    }
}
