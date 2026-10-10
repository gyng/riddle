//! Cut 119 (docs/CUT119_COMPANIONS.md): companions with depth.
//!
//! - item 0, pets for everyone: a default `tame` row (`drill:tame`, below the guard rows, above the fallback) tames a
//!   stray or a weakened tameable foe while a leash is held and a party slot is open; announced once (`TAMES STRAYS`),
//!   revocable (`revoke_tame`). The first stray waits on every lineage, its kind drawn across the roles.
//! - item 1, the pet carries the line: a heir's death with a party pet standing brings home a share of the lost
//!   carry (`fetched`, out of the grave's gold: no gold is made; the share stays under the return's 60 % less the
//!   heir purse's 30 %). A pet that falls is lamed for `LAME_RUNS` runs, its level kept; gone only at its
//!   `GONE_FALLS`-th fall (named). The killer, a band boss, is the pet's grudge (+1 a blow on him, cleared when he
//!   falls). A pet that served three heirs is the line's old hound.
//! - item 2, roles: each kind has one (`role_of`): fetcher (picks up gold in reach), guard (draws blows aimed at the
//!   hero), scout (sees the floor ahead), mender (heals the hero between fights). A ranged pet's blow is capped.
//! - item 3, levels: xp from every run (the depth reached; bench pets a quarter), hp and attack per level, the role's
//!   signature at L3 and L5 (announced once), cap L7.
//! - item 4, the kennel keeper's order (`StandingSwitches.kennel`: `breed` for the wall · `best` · `off`): an egg every
//!   `BREED_EVERY` runs while away or not (cap `EGG_CAP`), its generation in the name (`Rook II`), +2 % stats a
//!   generation (cap +10 %), surplus released.
//! - item 5, pet + build synergies (`PET_SYNERGIES`): a party pet's role and a worn package, no level gate.
//!
//! `PetsState.off` pins every Cut 119 system off (the 307dbed hash, probes). Every draw is the run's or the lineage's
//! seeded stream: deterministic.
use crate::engine::{ExitTier, Game, LineageState, Run};
use crate::wire::Companion;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    Fetcher,
    Guard,
    Scout,
    Mender,
}

pub const ROLES: [Role; 4] = [Role::Fetcher, Role::Guard, Role::Scout, Role::Mender];

impl Role {
    pub fn word(self) -> &'static str {
        match self {
            Role::Fetcher => "fetcher",
            Role::Guard => "guard",
            Role::Scout => "scout",
            Role::Mender => "mender",
        }
    }
    pub fn parse(s: &str) -> Option<Role> {
        ROLES.into_iter().find(|r| r.word() == s)
    }
    /// The fixed signatures (L3, L5).
    pub fn signatures(self) -> [&'static str; 2] {
        match self {
            Role::Fetcher => ["carry more", "bring back"],
            Role::Guard => ["taunt", "bulwark"],
            Role::Scout => ["map", "warn"],
            Role::Mender => ["patch", "revive once"],
        }
    }
}

/// A kind's role (its base behaviour; tags still add verbs).
pub fn role_of(kind: &str) -> Role {
    match kind {
        "rat" | "jackal" | "monkey" | "forge_imp" | "spectral_hound" => Role::Fetcher,
        "goblin_archer" | "bell_sentinel" | "lurker" | "wraith" | "siren" | "goblin_conjurer" | "eel" | "deep_eel" => Role::Scout,
        "pink_jelly" | "bloat" | "acolyte" | "mirror_shade" | "echo" | "smith" => Role::Mender,
        _ => Role::Guard,
    }
}

/// A companion's role (its kind's).
pub fn role(c: &Companion) -> Role {
    role_of(&c.kind)
}

/// A pet's life past its kind (serde-default: an older save's pets start at zero).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PetLife {
    /// `fetcher · guard · scout · mender` (set while Cut 119 is on; the wire's word).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub xp: u32,
    /// Runs it sits out, lamed (`lame > 0`: not fielded).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub lame: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub falls: u32,
    /// The heirs it went down with (numbers), first first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub heirs: Vec<u32>,
    /// The band boss that killed a heir beside it (+1 a blow on him until he falls).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grudge: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub runs: u32,
    /// Bred by the kennel keeper (not tamed wild).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bred: bool,
    /// The signature levels announced (3, 5).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub announced: Vec<u32>,
    /// Gold it brought home from fallen heirs, lifetime.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub fetched: i32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}
fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

impl PetLife {
    pub fn is_default(&self) -> bool {
        *self == PetLife::default()
    }
}

/// The lineage's companion state (`LineageState.pets`).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PetsState {
    /// Every Cut 119 system pinned off (probes, the 307dbed hash).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub off: bool,
    /// The default tame row revoked by the player (one tap; `revoke_tame`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tame_revoked: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tame_announced: bool,
    /// Runs since the keeper's last egg.
    #[serde(default)]
    pub breed_runs: u32,
    /// Synergies named (once each).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub named: BTreeSet<String>,
    /// Gold pets brought home from fallen heirs, lifetime.
    #[serde(default)]
    pub fetched: i64,
    /// The old hounds' chronicle lines (`Rook · served Ada, Bram, Cole`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub old_hounds: Vec<String>,
    #[serde(default)]
    pub bred: u32,
    #[serde(default)]
    pub released: u32,
    /// The death this run fetched from (run id, gold): the grave holds the rest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetched_run: Option<(u32, i32)>,
}

/// The run's companion switches (`Run.pets`), read in the sim.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunPets {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub on: bool,
    /// The pet synergies formed at the send (a bit each, `PET_SYNERGIES`).
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub syn: u8,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub revived: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub mend_t: u32,
    /// The floor whose first blow a scout's `warn` softened.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub warned: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub slots: u32,
    /// The first stray's kind (empty: the jackal).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub first_kind: String,
}

fn is_zero_u8(n: &u8) -> bool {
    *n == 0
}

impl PetsState {
    pub fn is_default(&self) -> bool {
        *self == PetsState::default()
    }
}

impl RunPets {
    pub fn is_default(&self) -> bool {
        *self == RunPets::default()
    }
}

/// Pin every Cut 119 system off (the set recompiled: the tame row out).
pub fn pin_off(l: &mut LineageState) {
    l.pets.off = true;
    crate::packages::recompile(l);
}

pub fn on(l: &LineageState) -> bool {
    !l.pets.off
}

// ---------------------------------------------------------------- levels

pub const CAP: u32 = 7;
/// The xp to reach each level (L1 … L7): a run pays its depth reached.
pub const LEVEL_XP: [u32; 7] = [0, 500, 1200, 2400, 4000, 6000, 9000];
/// Bench pets (the kennel, a lamed party pet) earn this share of a run's xp.
pub const BENCH_PCT: u32 = 25;
/// hp a level past the first (was 2: Cut 20).
pub const LEVEL_HP: i32 = 4;

pub fn level_for(xp: u32) -> u32 {
    LEVEL_XP.iter().rposition(|t| xp >= *t).map_or(1, |i| i as u32 + 1).min(CAP)
}

/// The pedigree's stat bonus, percent (+2 a generation, cap +10).
pub fn pedigree_pct(gen: u32) -> i32 {
    (2 * gen as i32).min(10)
}

/// The signature a level opens (L3, L5).
pub fn signature(r: Role, level: u32) -> Option<&'static str> {
    match level {
        3 => Some(r.signatures()[0]),
        5 => Some(r.signatures()[1]),
        _ => None,
    }
}

/// The pet's attack bonus at its level (+1 to the top of the blow every two levels, +1 to the floor at L4 and L7).
pub fn atk_bonus(level: u32) -> (i32, i32) {
    let l = level.max(1) as i32 - 1;
    (l / 3, l / 2)
}

/// A Cut 119 pet's monster stats (`engine::companion_monster`): level, pedigree.
pub fn shape_monster(m: &mut crate::monster::Monster, c: &Companion) {
    if c.life.role.is_empty() {
        return;
    }
    let (a, b) = atk_bonus(c.level);
    let ped = pedigree_pct(c.gen);
    m.atk = ((m.atk.0 + a) * (100 + ped) / 100, (m.atk.1 + b) * (100 + ped) / 100);
}

/// A Cut 119 pet's max hp on top of its kind's (`engine::pet_max_hp`).
pub fn max_hp(c: &Companion, base: i32) -> i32 {
    (base + LEVEL_HP * (c.level.max(1) as i32 - 1)) * (100 + pedigree_pct(c.gen)) / 100
}

// ---------------------------------------------------------------- roles in the sim

/// The fetcher's reach from the hero (L3 `carry more`, the Quartermaster add).
pub fn fetch_reach(level: u32, syn: u8) -> i32 {
    let _ = syn;
    14 + if level >= 3 { 4 } else { 0 }
}

/// The guard's chance to take a blow aimed at the hero (percent).
pub fn guard_pct(level: u32, syn: u8) -> u32 {
    (65 + if level >= 3 { 15 } else { 0 } + if syn & (syn_bit("packmaster") | syn_bit("shieldmate")) != 0 { 10 } else { 0 }).min(90)
}

/// Ticks between a mender's heals.
pub fn mend_every(syn: u8) -> u32 {
    let _ = syn;
    90
}

/// The mender heals the hero only under this share of his hp (between fights).
pub const MEND_UNDER_PCT: i32 = 60;

/// hp a mender's heal gives (L3 `patch`: 2).
pub fn mend_hp(level: u32) -> i32 {
    if level >= 3 {
        2
    } else {
        1
    }
}

/// The scout's reveal radius round the stairs down (L3 `map`, the Falconer add).
pub fn scout_radius(level: u32, syn: u8) -> i32 {
    4 + if level >= 3 { 4 } else { 0 } + if syn & syn_bit("falconer") != 0 { 3 } else { 0 }
}

/// A ranged pet's blow is capped (the summons' cap): it never carries the fight.
pub fn shot_cap(level: u32) -> i32 {
    1 + level as i32 / 3
}

/// The pet's blow: the ranged cap, the grudge.
pub fn pet_blow(run: &Run, mi: usize, ti: usize, verb: &str, dmg: i32) -> i32 {
    if !run.pets.on || dmg <= 0 {
        return dmg;
    }
    let Some(c) = run.monsters[mi].cid.and_then(|cid| run.companion(cid)) else { return dmg };
    let mut d = dmg;
    if matches!(verb, "shoot") {
        d = d.min(shot_cap(c.level));
    }
    if c.life.grudge.as_deref().is_some_and(|g| g == run.monsters[ti].kind) {
        d += 1;
    }
    d
}

/// The scout's look ahead at a floor's arrival: the stairs down and the tiles round them, seen.
pub fn on_floor(run: &mut Run) {
    if !run.pets.on {
        return;
    }
    let best = run
        .monsters
        .iter()
        .filter(|m| m.hp > 0 && m.is_companion())
        .filter_map(|m| m.cid.and_then(|cid| run.companion(cid)))
        .filter(|c| role(c) == Role::Scout)
        .map(|c| c.level)
        .max();
    let Some(level) = best else { return };
    let r = scout_radius(level, run.pets.syn);
    let s = run.floor.stairs_down;
    let map = &mut run.floor.map;
    for dy in -r..=r {
        for dx in -r..=r {
            let p = crate::geom::Pos::new(s.x + dx, s.y + dy);
            if map.in_bounds(p) {
                let i = map.idx(p);
                map.seen[i] = true;
            }
        }
    }
    crate::petstats::bump(crate::petstats::Stat::ScoutReveals, 1);
}

/// A living party pet of `r` at or above `level`: its monster index.
pub fn living(run: &Run, r: Role, level: u32) -> Option<usize> {
    run.monsters.iter().position(|m| m.hp > 0 && m.is_companion() && m.cid.and_then(|cid| run.companion(cid)).is_some_and(|c| role(c) == r && c.level >= level))
}

// ---------------------------------------------------------------- the tame row

pub const TAME_ORIGIN: &str = "drill:tame";
/// The tame row's words (its drill line).
pub const TAME_TEXT: &str = "tames strays";

/// The default tame row: a stray or a weakened tameable foe, a leash held, a party slot open (`ai::verb_tame`'s `open`).
pub fn tame_row() -> crate::rules::Row {
    crate::rules::Row::new(vec![crate::rules::Cond::n("foes>=", 1)], crate::rules::Verb::arg("tame", "open")).from(TAME_ORIGIN)
}

/// The row compiles (pets on, not revoked, `tame` owned).
pub fn tame_on(l: &LineageState) -> bool {
    // (a custom stance plays as written: the pen's own set)
    on(l) && !l.pets.tame_revoked && !l.pkg.literal && l.pkg.stance != crate::packages::CUSTOM && l.unlocks.contains("tame")
}

/// Revoke (or restore) the tame row — one tap.
pub fn revoke_tame(l: &mut LineageState, revoked: bool) {
    l.pets.tame_revoked = revoked;
    crate::packages::recompile(l);
}

/// `open`: a weakened foe is worth a leash only when its kind is known well enough (≥ this chance).
pub const OPEN_TAME_MIN: u32 = 40;

// ---------------------------------------------------------------- synergies

pub struct PetSynergy {
    pub id: &'static str,
    pub name: &'static str,
    pub role: Role,
    /// The package (a stance, a tactic, a temperament) worn.
    pub pick: &'static str,
    pub effect: &'static str,
}

/// (The Quartermaster — light hands + a fetcher — formed on 0/16 PICKED seeds in the first probe: dropped, research §5.)
/// (The Field medic — guarded + a mender — formed on 1/8 PICKED seeds in the second probe: dropped, research §5.)
pub const PET_SYNERGIES: [PetSynergy; 3] = [
    PetSynergy { id: "packmaster", name: "Packmaster", role: Role::Guard, pick: "pack_break", effect: "guard draws more blows" },
    PetSynergy { id: "shieldmate", name: "Shieldmate", role: Role::Guard, pick: "corridor_fighting", effect: "guard draws more blows" },
    PetSynergy { id: "falconer", name: "Falconer", role: Role::Scout, pick: "kite_archers", effect: "scout sees further" },
];

pub fn syn_bit(id: &str) -> u8 {
    PET_SYNERGIES.iter().position(|s| s.id == id).map_or(0, |i| 1 << i)
}

/// The fielded roles (party pets not lamed).
fn fielded_roles(l: &LineageState) -> Vec<Role> {
    l.party.iter().filter(|c| c.life.lame == 0).map(role).collect()
}

/// The pet synergies the lineage forms now (role + build alone; no level gate).
pub fn synergies(l: &LineageState) -> Vec<&'static PetSynergy> {
    if !on(l) || l.pkg.literal {
        return Vec::new();
    }
    let roles = fielded_roles(l);
    let worn = l.pkg.equipped();
    PET_SYNERGIES.iter().filter(|s| roles.contains(&s.role) && worn.iter().any(|w| w == s.pick)).collect()
}

pub fn syn_mask(l: &LineageState) -> u8 {
    synergies(l).iter().fold(0, |m, s| m | syn_bit(s.id))
}

// ---------------------------------------------------------------- the send

/// The first stray's kind: the roles in turn across lineages (Cut 119 §2: strays arrive with a balanced mix).
pub fn first_kind(seed: u64) -> &'static str {
    ["jackal", "skeleton", "goblin_archer", "pink_jelly"][(crate::rng::Rng::derive(seed, crate::rng::hash_str("first_stray_kind")).below(4)) as usize]
}

/// At the send: the run's switches.
pub fn on_send(run: &mut Run, l: &LineageState) {
    if !on(l) {
        run.pets = RunPets::default();
        return;
    }
    run.pets = RunPets { on: true, syn: syn_mask(l), slots: l.party_slots(), first_kind: first_kind(l.seed).to_string(), ..RunPets::default() };
}

/// The kennel's party fielded this send (lamed pets sit out).
pub fn fielded(l: &LineageState) -> Vec<Companion> {
    l.party.iter().filter(|c| !on(l) || c.life.lame == 0).cloned().collect()
}

// ---------------------------------------------------------------- the run's end

/// The killer of a death, a band boss: the grudge.
fn killer_boss(run: &Run) -> Option<String> {
    run.death_cause.clone().filter(|k| crate::defs::MONSTERS.iter().any(|m| m.kind == *k && m.boss))
}

/// A companion's growth after a run it went on (alive or fallen): xp, levels, signatures (announced once), the heir
/// served, the grudge. Returns the report's lines (`Rook L3 · taunt`).
pub fn grow(c: &mut Companion, run: &Run, tier: ExitTier) -> Vec<String> {
    let mut out = Vec::new();
    let r = role(c);
    c.life.role = r.word().to_string();
    c.life.runs += 1;
    c.life.xp += run.max_depth.max(1);
    let lv = level_for(c.life.xp).max(c.level).min(CAP);
    if lv > c.level {
        c.level = lv;
        c.max_rows = (1 + lv as usize).min(4);
        for s in [3, 5] {
            if lv >= s && !c.life.announced.contains(&s) {
                c.life.announced.push(s);
                if let Some(sig) = signature(r, s) {
                    out.push(format!("{} L{s} · {sig}", c.name));
                }
            }
        }
        if out.is_empty() {
            out.push(format!("{} L{lv}", c.name));
        }
    }
    if !c.life.heirs.contains(&run.heir) {
        c.life.heirs.push(run.heir);
    }
    if tier == ExitTier::Death {
        if let Some(b) = killer_boss(run) {
            c.life.grudge = Some(b);
        }
    }
    if c.life.grudge.as_ref().is_some_and(|g| run.boss_kills.iter().any(|(_, k)| k == g)) {
        c.life.grudge = None;
    }
    out
}

/// A pet falls: lamed (level kept) — gone only at its `GONE_FALLS`-th fall. True: lamed (stays).
pub const LAME_RUNS: u32 = 3;
pub const GONE_FALLS: u32 = 6;
pub fn lame(c: &mut Companion) -> bool {
    c.life.falls += 1;
    if c.life.falls >= GONE_FALLS {
        return false;
    }
    c.life.lame = LAME_RUNS;
    true
}

/// The pets that did not go on this run: a quarter of its xp, a lamed one a run nearer the field.
pub fn bench(l: &mut LineageState, run: &Run) -> u32 {
    let mut healed = 0;
    let went: Vec<u32> = run.companions.iter().map(|c| c.id).collect();
    let gain = run.max_depth.max(1) * BENCH_PCT / 100;
    for c in l.party.iter_mut().chain(l.kennel.iter_mut()).filter(|c| !went.contains(&c.id)) {
        c.life.role = role_of(&c.kind).word().to_string();
        c.life.xp += gain;
        let lv = level_for(c.life.xp).max(c.level).min(CAP);
        if lv > c.level {
            c.level = lv;
            c.max_rows = (1 + lv as usize).min(4);
        }
        if c.life.lame == 1 {
            healed += 1;
        }
        c.life.lame = c.life.lame.saturating_sub(1);
    }
    healed
}

/// The share of a fallen heir's lost carry a standing party pet brings home (percent): 20, a fetcher 24, its
/// `bring back` 28. The invariant (research §1): this plus the heir purse floor (30) stays under a return's 60.
pub fn fetch_pct(c: &Companion) -> i32 {
    match (role(c), c.level >= 5) {
        (Role::Fetcher, true) => 28,
        (Role::Fetcher, false) => 24,
        _ => 20,
    }
}
pub const FETCH_MAX_PCT: i32 = 28;

/// Cut 119 at a live run's end (after the exit line, before Cut 118's graves): the pack fetched home on a death, the
/// keeper's eggs, the old hounds, the synergies named, the tame row announced. Report lines into the batch.
pub fn on_run_end(g: &mut Game, run: &Run, tier: ExitTier) {
    if g.sim || !run.pets.on || g.lineage.pets.off {
        return;
    }
    let day = crate::feats::day_of(g.lineage.clock_s);
    g.lineage.pets.fetched_run = None;
    // the pack fetched home: the best fetcher among the pets standing at the death
    if tier == ExitTier::Death {
        let lost = g.last_exit.as_ref().filter(|x| x.run_id == run.id).map_or(0, |x| (x.carried - x.kept).max(0));
        let standing: Vec<u32> = run.party_alive().filter_map(|m| m.cid).collect();
        let best = run.companions.iter().filter(|c| standing.contains(&c.id)).max_by_key(|c| (fetch_pct(c), c.level, std::cmp::Reverse(c.id))).cloned();
        if let (Some(c), true) = (best, lost > 0) {
            let gold = lost * fetch_pct(&c) / 100;
            if gold > 0 {
                let heir = crate::legacy::hero_name(g.lineage.seed, run.heir);
                g.lineage.gold_move(gold, &format!("fetched by {}", c.name));
                g.lineage.pets.fetched += i64::from(gold);
                g.lineage.pets.fetched_run = Some((run.id, gold));
                if let Some(p) = g.lineage.party.iter_mut().chain(g.lineage.kennel.iter_mut()).find(|p| p.id == c.id) {
                    p.life.fetched += gold;
                }
                let text = format!("{} brought {heir}'s pack", c.name);
                g.events.push(crate::wire::Ev::Note { t: run.turn, text: format!("{text}.") });
                crate::feats::news(&mut g.lineage, "fetched", format!("{text} · ${gold}"), day);
                crate::petstats::bump(crate::petstats::Stat::FetchedGold, gold as u64);
                crate::petstats::bump(crate::petstats::Stat::FetchDeaths, 1);
            }
        }
        if lost > 0 {
            crate::petstats::bump(crate::petstats::Stat::LostCarry, lost as u64);
            if !standing.is_empty() {
                crate::petstats::bump(crate::petstats::Stat::LostCarryWithPet, lost as u64);
            }
        }
    }
    // the old hounds: a pet that served three heirs
    let names: Vec<(String, Vec<u32>)> = g.lineage.party.iter().chain(g.lineage.kennel.iter()).filter(|c| c.life.heirs.len() == 3).map(|c| (c.name.clone(), c.life.heirs.clone())).collect();
    for (name, heirs) in names {
        if g.lineage.pets.old_hounds.iter().any(|h| h.starts_with(&format!("{name} ·"))) {
            continue;
        }
        let served: Vec<&str> = heirs.iter().map(|h| crate::legacy::hero_name(g.lineage.seed, *h)).collect();
        let line = format!("{name} · old hound · served {}", served.join(", "));
        g.lineage.pets.old_hounds.push(line.clone());
        g.lineage.heir_deed(format!("{name} became the old hound"));
        crate::feats::news(&mut g.lineage, "old_hound", line, day);
    }
    // the synergies, named once
    for s in synergies(&g.lineage) {
        if g.lineage.pets.named.insert(s.id.to_string()) {
            g.batch.pkg_lines.push(s.name.to_uppercase());
            crate::feats::news(&mut g.lineage, "pet_synergy", format!("{} · {}", s.name, s.effect), day);
        }
    }
    // the tame row, announced once (as a drill)
    if tame_on(&g.lineage) && !g.lineage.pets.tame_announced {
        g.lineage.pets.tame_announced = true;
        g.batch.pkg_lines.push(TAME_TEXT.to_uppercase());
    }
    keeper(g, day);
}

// ---------------------------------------------------------------- the kennel keeper

/// The keeper's orders (`StandingSwitches.kennel`), the default first.
pub const KENNEL_ORDERS: [&str; 3] = ["breed", "best", "off"];
/// Runs between the keeper's eggs.
pub const BREED_EVERY: u32 = 20;
/// Bred eggs waiting at once (eggs accumulate while away, up to this).
pub const EGG_CAP: usize = 2;
/// Pets the kennel keeps past the party's slots (the rest released, the weakest first).
pub const KENNEL_SPARE: usize = 3;

/// The role a wall wants (the keeper's `breed for the wall`): guards at the boss walls, menders in the long bands,
/// scouts in the Deep, fetchers early.
pub fn wall_role(best: u32) -> Role {
    match best {
        0..=7 => Role::Fetcher,
        8..=17 => Role::Guard,
        18..=27 => Role::Mender,
        _ => Role::Scout,
    }
}

/// The keeper's score of a pet (`keep the best`): its level, pedigree and fit to the wall.
pub fn score(c: &Companion, best: u32) -> i32 {
    c.level as i32 * 10 + pedigree_pct(c.gen) + if role(c) == wall_role(best) { 15 } else { 0 } - c.life.lame as i32
}

/// The keeper's release score: `score`, less for a kind the kennel holds twice (the line's pets stay a mix).
fn keep_score(l: &LineageState, c: &Companion, best: u32) -> i32 {
    let same = l.all_companions().filter(|o| o.kind == c.kind).count() as i32;
    score(c, best) - 12 * (same - 1).max(0)
}

fn keeper(g: &mut Game, day: u32) {
    let order = g.lineage.orders.kennel.clone();
    let l = &mut g.lineage;
    if order == "off" {
        return;
    }
    l.pets.breed_runs += 1;
    let bred_eggs = l.eggs.iter().filter(|e| !e.from_loss).count();
    if l.pets.breed_runs >= BREED_EVERY && bred_eggs < EGG_CAP {
        let best = l.best_depth;
        let mut pets: Vec<Companion> = l.all_companions().filter(|c| c.level >= 2).cloned().collect();
        pets.sort_by_key(|c| (std::cmp::Reverse(score(c, best)), c.id));
        if let Some(a) = pets.first().cloned() {
            l.pets.breed_runs = 0;
            // the egg's kind from either parent: the wall's role when one has it (`breed`), else the better
            // (the second parent of another kind when the kennel has one)
            let b = pets.iter().skip(1).find(|c| c.kind != a.kind).or(pets.get(1)).cloned();
            // (`breed`: the parent whose kind the party holds least, then the wall's role — the line's pets stay a mix)
            let held = |k: &str| l.party.iter().filter(|c| c.kind == k).count();
            let sire = match (&b, order.as_str()) {
                (Some(b), "breed") if (held(&b.kind), role(b) != wall_role(best)) < (held(&a.kind), role(&a) != wall_role(best)) => b.clone(),
                _ => a.clone(),
            };
            let gen = a.gen.max(b.as_ref().map_or(0, |b| b.gen)) + 1;
            let eid = l.new_comp_id();
            let hatch_in = l.egg_rests();
            l.eggs.push(crate::wire::Egg { id: eid, kind: sire.kind.clone(), tags: sire.tags.clone(), gen, hatch_in, from_loss: false, sire: base_name(&sire.name).to_string() });
            l.eggs_laid += 1;
            l.pets.bred += 1;
            l.bred.insert(sire.kind.clone());
            crate::feats::news(l, "bred", format!("{} egg · {}", base_name(&sire.name), roman(gen)), day);
            crate::petstats::bump(crate::petstats::Stat::Bred, 1);
        }
    }
    release(g, day);
}

/// Surplus released: past the party's slots and `KENNEL_SPARE`, the kennel's weakest go free (never a party pet).
pub fn release(g: &mut Game, day: u32) {
    let l = &mut g.lineage;
    let cap = l.party_slots() as usize + KENNEL_SPARE;
    let best = l.best_depth;
    // (never a lamed pet: it is the line's, mending)
    while l.party.len() + l.kennel.len() > cap && l.kennel.iter().any(|c| c.life.lame == 0) {
        let Some(i) = (0..l.kennel.len()).filter(|&i| l.kennel[i].life.lame == 0).min_by_key(|&i| (keep_score(l, &l.kennel[i], best), std::cmp::Reverse(l.kennel[i].id))) else { break };
        let c = l.kennel.remove(i);
        l.pets.released += 1;
        crate::feats::news(l, "released", format!("{} released", c.name), day);
    }
    // the keeper fields the best: a lamed party pet gives way to a fit one in the kennel
    if g.lineage.orders.kennel != "off" {
        let l = &mut g.lineage;
        for i in 0..l.party.len() {
            if l.party[i].life.lame == 0 {
                continue;
            }
            if let Some(k) = (0..l.kennel.len()).filter(|&k| l.kennel[k].life.lame == 0).max_by_key(|&k| (score(&l.kennel[k], best), std::cmp::Reverse(l.kennel[k].id))) {
                let fit = l.kennel.remove(k);
                let out = std::mem::replace(&mut l.party[i], fit);
                l.kennel.push(out);
            }
        }
    }
}

/// A name without its generation (`Rook II` → `Rook`).
pub fn base_name(name: &str) -> &str {
    match name.rsplit_once(' ') {
        Some((b, g)) if !g.is_empty() && g.chars().all(|c| matches!(c, 'I' | 'V' | 'X')) => b,
        _ => name,
    }
}

pub fn roman(n: u32) -> String {
    const T: [(u32, &str); 4] = [(10, "X"), (9, "IX"), (5, "V"), (4, "IV")];
    let mut n = n.max(1);
    let mut s = String::new();
    for (v, r) in T {
        while n >= v {
            s.push_str(r);
            n -= v;
        }
    }
    s.push_str(&"I".repeat(n as usize));
    s
}

/// A bred egg's name: the sire's, its generation in Roman (`Rook III`).
pub fn bred_name(sire: &str, gen: u32) -> String {
    format!("{sire} {}", roman(gen + 1))
}

// ---------------------------------------------------------------- the wire

/// Cut 119's camp read (`Lineage.pets`).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PetsWire {
    /// The default tame row: its words, revoked or not (`revokeTame`).
    pub tame: String,
    pub tame_revoked: bool,
    /// The keeper's order (`breed · best · off`).
    pub kennel: String,
    /// Pet synergies formed now: `Falconer · scout sees further`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synergies: Vec<String>,
    #[serde(default)]
    pub fetched: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub old_hounds: Vec<String>,
    pub bred: u32,
    pub released: u32,
}

pub fn wire(l: &LineageState) -> Option<PetsWire> {
    if !on(l) {
        return None;
    }
    Some(PetsWire {
        tame: TAME_TEXT.into(),
        tame_revoked: l.pets.tame_revoked,
        kennel: l.orders.kennel.clone(),
        synergies: synergies(l).iter().map(|s| format!("{} · {}", s.name, s.effect)).collect(),
        fetched: l.pets.fetched,
        old_hounds: l.pets.old_hounds.clone(),
        bred: l.pets.bred,
        released: l.pets.released,
    })
}

/// The report's pet line (one a return): the lead party pet (`Rook · guard L4 · 3 heirs`).
pub fn report_line(l: &LineageState) -> Option<String> {
    if !on(l) {
        return None;
    }
    let c = l.party.iter().max_by_key(|c| (c.life.lame == 0, c.level, std::cmp::Reverse(c.id)))?;
    let lame = if c.life.lame > 0 { " · lamed" } else { "" };
    Some(format!("{} · {} L{}{lame}", c.name, role(c).word(), c.level))
}

/// The return's pet line (`ReturnReport.feats`, `k: pet`): one a return while a pet is in the party.
pub fn on_report(l: &LineageState, r: &mut crate::wire::ReturnReport) {
    if let Some(text) = report_line(l) {
        r.feats.push(crate::feats::FeatNews { k: "pet".into(), text, day: crate::feats::day_of(l.clock_s) });
    }
}
