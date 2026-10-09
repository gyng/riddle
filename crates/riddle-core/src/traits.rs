//! Cut 30 (docs/TRAITS.md, docs/CUT30.md): heir traits — a condition on the hero, never an action.
//!
//! A trait is **when × gift · cost** drawn from parts: the gift is live while its `when` holds (read at
//! the hero's action), the cost is always on. A trait never picks a verb: it changes what a verb does
//! (`Hero::atk`, `Hero::blunt`, `Hero::speed` read `Hero.gift`; a rest heals more, a retreat step costs
//! less, a heal potion heals less) or what a condition sees (`hp` moves under `mend`; `if: trait <head>`
//! and `if: gift live` read it). Only the rows and the chores choose.
//!
//! The runtime draws from the measured table (`traits.json`, written by `examples/traits.rs`: a shape
//! ships only when its strict build test, its size bound and the lever pass on the cohort sets), never
//! from raw parts. An heir carries two slots — *blood* (inherited; opens at heir 5) and *born* (the wake's
//! pick of three cards) — and the neutral heir (`HeirTraits.neutral`, every bot) carries none.
use crate::engine::{Ctx, LineageState, Run};
use crate::rng::{hash_str, Rng};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// Parts.

/// The context a gift is live in.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum When {
    /// hp < 50 %
    Hurt,
    /// ≥ 2 foes at the hero's elbow
    Crowded,
    /// a boss in view
    Boss,
    /// no foe in view
    Quiet,
    /// D9 and below (`DEEP_FROM`)
    Deep,
    /// the first `FIRST_TURNS` turns of a floor
    First,
    /// always (the verb twists: `iron gut`, `light hands`)
    Always,
    /// the kind that killed the last heir in view (the `grudge` twist)
    Kin,
}

/// What the trait gives while live.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Gift {
    /// + damage on every blow
    Fury,
    /// − damage on every blow taken
    Guard,
    /// + speed
    Quick,
    /// + hp every live action (the hp an hp-row reads moves)
    Mend,
    /// a rest heals more
    Rested,
    /// a retreat / corridor / walk-home step costs less
    Sure,
    /// twist: a malevolent drink harms half
    IronGut,
    /// twist: a pick up that takes something takes no turn
    LightHands,
}

/// What the trait costs, always on (visible at the wake).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum Cost {
    #[default]
    None,
    /// − `FRAIL_PCT` % max hp
    Frail,
    /// − 1 speed
    Slow,
    /// a heal potion heals ⅔
    Thin,
    /// − 2 sight in the Deep
    Dim,
}

/// Rarity (the chip's rim; no word). A marked trait is born of a lineage event, never drawn.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    #[default]
    Common,
    Uncommon,
    Rare,
    Marked,
}

pub const WHENS: [When; 6] = [When::Hurt, When::Crowded, When::Boss, When::Quiet, When::Deep, When::First];
pub const GIFTS: [Gift; 6] = [Gift::Fury, Gift::Guard, Gift::Quick, Gift::Mend, Gift::Rested, Gift::Sure];
pub const COSTS: [Cost; 4] = [Cost::Frail, Cost::Slow, Cost::Thin, Cost::Dim];

/// `deep`: this depth and below.
pub const DEEP_FROM: u32 = 9;
/// `first`: the first turns of a floor.
pub const FIRST_TURNS: u32 = 20;
/// `hurt`: under this hp %.
pub const HURT_PCT: i32 = 50;
/// `frail`: the share of max hp the cost takes.
pub const FRAIL_PCT: i32 = 10;
/// `thin`: the share of a heal potion's heal that lands.
pub const THIN_PCT: i32 = 67;
/// `dim`: sight lost in the Deep.
pub const DIM_SIGHT: i32 = 2;

impl When {
    pub fn word(self) -> &'static str {
        match self {
            When::Hurt => "hurt",
            When::Crowded => "crowded",
            When::Boss => "boss",
            When::Quiet => "quiet",
            When::Deep => "deep",
            When::First => "first",
            When::Always => "always",
            When::Kin => "kin",
        }
    }
    pub fn parse(s: &str) -> Option<When> {
        [When::Hurt, When::Crowded, When::Boss, When::Quiet, When::Deep, When::First, When::Always, When::Kin].into_iter().find(|w| w.word() == s)
    }
}

impl Gift {
    pub fn word(self) -> &'static str {
        match self {
            Gift::Fury => "fury",
            Gift::Guard => "guard",
            Gift::Quick => "quick",
            Gift::Mend => "mend",
            Gift::Rested => "rested",
            Gift::Sure => "sure",
            Gift::IronGut => "iron gut",
            Gift::LightHands => "light hands",
        }
    }
    /// The watch's stamp on the hero's plate, the first live turn of a floor (1 word).
    pub fn stamp(self) -> &'static str {
        match self {
            Gift::Fury => "WRATH",
            Gift::Guard => "GUARD",
            Gift::Quick => "QUICK",
            Gift::Mend => "MEND",
            Gift::Rested => "RESTED",
            Gift::Sure => "SURE",
            Gift::IronGut => "IRON",
            Gift::LightHands => "DEFT",
        }
    }
    pub fn is_twist(self) -> bool {
        matches!(self, Gift::IronGut | Gift::LightHands)
    }
}

impl Cost {
    pub fn word(self) -> &'static str {
        match self {
            Cost::None => "",
            Cost::Frail => "frail",
            Cost::Slow => "slow",
            Cost::Thin => "thin",
            Cost::Dim => "dim",
        }
    }
}

/// One trait: when × gift · cost, at a tier (the gift's size: common 1 step, uncommon 2).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Shape {
    pub when: When,
    pub gift: Gift,
    #[serde(default)]
    pub cost: Cost,
    #[serde(default)]
    pub tier: Tier,
}

impl Shape {
    pub const fn new(when: When, gift: Gift, cost: Cost) -> Shape {
        Shape { when, gift, cost, tier: Tier::Common }
    }
    pub fn at(mut self, tier: Tier) -> Shape {
        self.tier = tier;
        self
    }
    /// The gift's step: common (and marked) 1, uncommon 2; a rare twist 1.
    pub fn step(self) -> i32 {
        if self.tier == Tier::Uncommon {
            2
        } else {
            1
        }
    }
    /// The lexicon head (docs/TRAITS.md §2): one word per (when, gift) pair; `None` for a pair the
    /// generator rejects statically (a gift its context can never use).
    pub fn head_of(when: When, gift: Gift) -> Option<&'static str> {
        use Gift::*;
        use When::*;
        Some(match (when, gift) {
            (Hurt, Fury) => "wrathful",
            (Hurt, Guard) => "stubborn",
            (Hurt, Quick) => "skittish",
            (Hurt, Mend) => "tough-blooded",
            (Hurt, Rested) => "slow-healer",
            (Hurt, Sure) => "sure-footed",
            (Crowded, Fury) => "brawler",
            (Crowded, Guard) => "back-to-wall",
            (Crowded, Quick) => "slippery",
            (Crowded, Mend) => "thick-skinned",
            (Crowded, Sure) => "light-footed",
            (Boss, Fury) => "grudge-keeper",
            (Boss, Guard) => "unbowed",
            (Boss, Quick) => "duellist",
            (Boss, Mend) => "defiant",
            (Boss, Sure) => "wary",
            (Quiet, Quick) => "restless",
            (Quiet, Mend) => "light sleeper",
            (Quiet, Rested) => "sound sleeper",
            (Quiet, Sure) => "wanderer",
            (Deep, Fury) => "fen-born",
            (Deep, Guard) => "mud-hide",
            (Deep, Quick) => "night-eyed",
            (Deep, Mend) => "marsh-blood",
            (Deep, Rested) => "deep sleeper",
            (Deep, Sure) => "cave-wise",
            (First, Fury) => "eager",
            (First, Guard) => "steady",
            (First, Quick) => "keen",
            (First, Mend) => "fresh",
            (First, Rested) => "early riser",
            (First, Sure) => "scout",
            (Always, IronGut) => "iron gut",
            (Always, LightHands) => "light hands",
            (Kin, Fury) => "grudge",
            // rejected: no foe to hit or be hit by when quiet; no rest with foes about
            _ => return None,
        })
    }
    pub fn head(self) -> &'static str {
        Shape::head_of(self.when, self.gift).unwrap_or("?")
    }
    /// The chip: `<head> · <cost>` (≤ 3 words).
    pub fn chip(self) -> String {
        match self.cost {
            Cost::None => self.head().to_string(),
            c => format!("{} · {}", self.head(), c.word()),
        }
    }
    /// The gift's size, as the card's formula writes it (`fury +1`, `mend +1`, `rested +4`, `sure`).
    pub fn gift_text(self) -> String {
        let s = self.step();
        match self.gift {
            Gift::Fury | Gift::Guard => format!("{} +{s}", self.gift.word()),
            Gift::Quick => format!("quick +{}", quick_speed(s)),
            Gift::Mend => format!("mend +{}", mend_hp(s)),
            Gift::Rested => format!("rested +{}", rest_extra(s)),
            Gift::Sure => format!("sure {}%", sure_refund(s)),
            Gift::IronGut | Gift::LightHands => self.gift.word().to_string(),
        }
    }
    /// The card's formula (`[hurt] → fury +1 · frail`), `?` for the size of an unlearned gift.
    pub fn formula(self, learned: bool) -> String {
        let g = if learned { self.gift_text() } else { format!("{} ?", self.gift.word()) };
        let w = self.when.word();
        match self.cost {
            Cost::None => format!("[{w}] → {g}"),
            c => format!("[{w}] → {g} · {}", c.word()),
        }
    }
    /// The trace's mark while the gift acts (`fury +1`).
    pub fn mark(self) -> String {
        self.gift_text()
    }
    /// The fact learned the first time the gift goes live (`trait:wrathful`).
    pub fn fact(self) -> String {
        format!("trait:{}", self.head())
    }
    pub fn is_twist(self) -> bool {
        self.gift.is_twist() || self.when == When::Kin
    }
    /// The code the measurement and the table use (`fury@hurt·frail`).
    pub fn code(self) -> String {
        let c = if self.cost == Cost::None { String::new() } else { format!("·{}", self.cost.word()) };
        format!("{}@{}{c}", self.gift.word().replace(' ', "_"), self.when.word())
    }
    /// The same (when, gift, cost) — tier aside.
    pub fn same_parts(self, o: Shape) -> bool {
        self.when == o.when && self.gift == o.gift && self.cost == o.cost
    }
}

/// `quick`: speed per step.
pub fn quick_speed(step: i32) -> i32 {
    if step >= 2 {
        5
    } else {
        3
    }
}
/// `mend`: hp per live action per step.
pub fn mend_hp(step: i32) -> i32 {
    step
}
/// `rested`: the extra hp a rest heals per step (a rest heals 4).
pub fn rest_extra(step: i32) -> i32 {
    4 * step
}
/// `sure`: the share of an action's energy a sure step gives back, per step.
pub fn sure_refund(step: i32) -> i32 {
    if step >= 2 {
        75
    } else {
        50
    }
}

/// The generator's static filter: every (when, gift, cost) the lexicon names, less a cost that
/// cancels its own gift in its own context (`deep × quick · slow`), the twists apart.
pub fn generated() -> Vec<Shape> {
    let mut v = Vec::new();
    for when in WHENS {
        for gift in GIFTS {
            if Shape::head_of(when, gift).is_none() {
                continue;
            }
            for cost in COSTS {
                if when == When::Deep && gift == Gift::Quick && cost == Cost::Slow {
                    continue;
                }
                if gift == Gift::Rested && cost == Cost::Thin {
                    continue; // (both read a heal; the card would net to nothing it could show)
                }
                v.push(Shape::new(when, gift, cost));
            }
        }
    }
    v
}

/// The rare twists (no cost): `iron gut`, `light hands`, `grudge` (fury +1 vs the last heir's killer).
pub fn twists() -> Vec<Shape> {
    vec![Shape::new(When::Always, Gift::IronGut, Cost::None).at(Tier::Rare), Shape::new(When::Always, Gift::LightHands, Cost::None).at(Tier::Rare), Shape::new(When::Kin, Gift::Fury, Cost::None).at(Tier::Rare)]
}

// ---------------------------------------------------------------------------------------------
// The measured table.

/// A shipped shape and what `examples/traits.rs` measured (the gate rows read these).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Entry {
    pub code: String,
    pub when: String,
    pub gift: String,
    #[serde(default)]
    pub cost: String,
    /// "common" (and "uncommon" when the larger gift also held the bound and the lever), or "rare".
    pub tiers: Vec<String>,
    /// The cohort set the build was found on, the rows its best set differs by, the gain with the
    /// trait and without (bank share), its ± and the edit.
    #[serde(default)]
    pub set: String,
    #[serde(default)]
    pub diff: u32,
    #[serde(default)]
    pub gain_t: f64,
    #[serde(default)]
    pub gain_n: f64,
    #[serde(default)]
    pub pm: f64,
    #[serde(default)]
    pub edit: String,
    /// The largest |Δbank| and |Δdeath| on B0 over the sets (the bound) and the worst ratio of
    /// |Δbank| to the set's lever (< 1 on every set).
    #[serde(default)]
    pub d_bank: f64,
    #[serde(default)]
    pub d_death: f64,
    #[serde(default)]
    pub lever_ratio: f64,
}

impl Entry {
    pub fn shape(&self) -> Option<Shape> {
        let when = When::parse(&self.when)?;
        let gift = GIFTS.into_iter().chain([Gift::IronGut, Gift::LightHands]).find(|g| g.word().replace(' ', "_") == self.gift || g.word() == self.gift)?;
        let cost = COSTS.into_iter().find(|c| c.word() == self.cost).unwrap_or(Cost::None);
        let tier = if self.tiers.iter().any(|t| t == "rare") { Tier::Rare } else { Tier::Common };
        Some(Shape { when, gift, cost, tier })
    }
    pub fn uncommon(&self) -> bool {
        self.tiers.iter().any(|t| t == "uncommon")
    }
}

/// The table the build measured (`crates/riddle-core/src/traits.json`), checked in.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Table {
    /// The commit the measurement ran on, the seed, the cohort sets and the sends per arm.
    #[serde(default)]
    pub build: String,
    #[serde(default)]
    pub sets: Vec<String>,
    #[serde(default)]
    pub measured: u32,
    pub shipped: Vec<Entry>,
}

static TABLE_JSON: &str = include_str!("traits.json");

pub fn table() -> &'static Table {
    static T: std::sync::OnceLock<Table> = std::sync::OnceLock::new();
    T.get_or_init(|| serde_json::from_str(TABLE_JSON).unwrap_or_default())
}

/// The shipped shapes (commons and rares) with their uncommon flag.
pub fn shipped() -> Vec<(Shape, bool)> {
    table().shipped.iter().filter_map(|e| e.shape().map(|s| (s, e.uncommon()))).collect()
}

// ---------------------------------------------------------------------------------------------
// The lineage's heirs.

/// A card at the wake: a shape and where it came from (`fresh` · `twist` · `marked`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card {
    pub shape: Shape,
    pub source: String,
}

/// The lineage's traits: the heir's two slots, the wake's cards, the family's lean.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HeirTraits {
    /// 1 once this lineage is on Cut 30's traits (an older save maps its temperament: `upgrade`).
    #[serde(default)]
    pub v: u32,
    /// The neutral heir: no trait, no offer (every bot; `metrics::setup`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub neutral: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blood: Option<Shape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub born: Option<Shape>,
    /// The wake's three cards (empty once sent); `born` is the first until `pick`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub offer: Vec<Card>,
    /// The blood / born gift went live in a banked run of this heir (a blood trait that never did
    /// fades a tier at the next wake).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub blood_live: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub born_live: bool,
    /// This heir's blood was cut (the slot is empty; the next heir's born pick may pass into it).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cut: bool,
    /// The last wake's fade (`wrathful · frail → gone`), for the camp's line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faded: Option<String>,
    /// The family's lean: half of every fresh draw takes this `when`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bloodline: Option<When>,
    /// The kind a `grudge` trait is against (the last heir's killer at the wake that drew it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kin: Option<String>,
}

impl HeirTraits {
    pub fn worn(&self) -> impl Iterator<Item = Shape> + '_ {
        self.blood.iter().chain(self.born.iter()).copied()
    }
}

/// Traits arrive (the `traits` system): heir 3, or the first heir who dies past D5 — never day 0.
pub fn arrived(l: &LineageState) -> bool {
    !l.heirs.neutral && (l.heir >= 3 || l.ascension > 0 || l.graveyard.iter().any(|g| g.depth >= 5))
}

/// The blood slot opens at heir 5 (the `heirs` step).
pub fn blood_open(l: &LineageState) -> bool {
    !l.heirs.neutral && (l.heir >= 5 || l.ascension > 0)
}

/// The bloodline may be chosen at heir 12 or after an ascension.
pub fn bloodline_open(l: &LineageState) -> bool {
    !l.heirs.neutral && (l.heir >= 12 || l.ascension > 0)
}

/// A new lineage: on Cut 30's traits, the first heir neutral.
pub fn fresh() -> HeirTraits {
    HeirTraits { v: 1, ..HeirTraits::default() }
}

/// Cut 30 §1: an older save's temperament maps onto a shape (`cowardly` → `skittish`, `brave` →
/// `unbowed`, `greedy` → `light hands`, `curious` → `iron gut`); its overrides are gone.
pub fn legacy(t: crate::hero::Trait) -> Shape {
    use crate::hero::Trait as T;
    match t {
        T::Cowardly => Shape::new(When::Hurt, Gift::Quick, Cost::Slow),
        T::Brave => Shape::new(When::Boss, Gift::Guard, Cost::Frail),
        T::Greedy => Shape::new(When::Always, Gift::LightHands, Cost::None).at(Tier::Rare),
        T::Curious => Shape::new(When::Always, Gift::IronGut, Cost::None).at(Tier::Rare),
    }
}

/// An older save (no `heirs.v`): the heir keeps its temperament's name as a shape.
pub fn upgrade(l: &mut LineageState) {
    if l.heirs.v >= 1 {
        return;
    }
    l.heirs.v = 1;
    if !l.heirs.neutral && l.heirs.born.is_none() {
        l.heirs.born = Some(legacy(l.trait_));
    }
}

/// The neutral heir for good (bots, harnesses): no trait now or at any wake.
pub fn neutral(l: &mut LineageState) {
    l.heirs = HeirTraits { v: 1, neutral: true, ..HeirTraits::default() };
}

fn offer_rng(l: &LineageState) -> Rng {
    Rng::derive(l.seed, hash_str("heir_traits") ^ (l.heir + 1000 * l.ascension) as u64)
}

/// A fresh draw from the table: a `when` outside `avoid` (half the time the bloodline's, when it is
/// not avoided), a common (70 %) / uncommon (25 %, when the shape ships at that size) / rare twist (5 %).
fn draw(rng: &mut Rng, ship: &[(Shape, bool)], avoid: &[When], bloodline: Option<When>, kin: bool, first: bool) -> Option<Shape> {
    let roll = rng.below(100);
    let twists: Vec<Shape> = ship.iter().map(|(s, _)| *s).filter(|s| s.is_twist() && (s.when != When::Kin || kin) && !avoid.contains(&s.when)).collect();
    if roll >= 95 && !first && !twists.is_empty() {
        return Some(*rng.pick(&twists));
    }
    let lean = bloodline.filter(|w| !avoid.contains(w) && rng.chance(50));
    let pool: Vec<(Shape, bool)> = ship.iter().copied().filter(|(s, _)| !s.is_twist() && !avoid.contains(&s.when) && lean.is_none_or(|w| s.when == w) && (!first || s.cost != Cost::None)).collect();
    let pool = if pool.is_empty() { ship.iter().copied().filter(|(s, _)| !s.is_twist() && !avoid.contains(&s.when)).collect() } else { pool };
    if pool.is_empty() {
        return None;
    }
    let (s, unc) = *rng.pick(&pool);
    Some(if roll >= 70 && unc && !first { s.at(Tier::Uncommon) } else { s })
}

/// The dead heir's born trait, twisted: one part re-rolled to a shipped neighbour.
fn twist_of(rng: &mut Rng, ship: &[(Shape, bool)], parent: Shape, avoid: &[When]) -> Option<Shape> {
    let n: Vec<Shape> = ship
        .iter()
        .map(|(s, _)| *s)
        .filter(|s| !s.is_twist() && !avoid.contains(&s.when) && !s.same_parts(parent))
        .filter(|s| (s.when == parent.when) as u8 + (s.gift == parent.gift) as u8 + (s.cost == parent.cost) as u8 == 2)
        .collect();
    (!n.is_empty()).then(|| rng.pick(&n).at(if parent.tier == Tier::Uncommon { Tier::Uncommon } else { Tier::Common }))
}

/// The tier a blood trait fades to when its gift never went live in a banked run (`None`: gone).
pub fn fade(s: Shape) -> Option<Shape> {
    match s.tier {
        Tier::Uncommon => Some(s.at(Tier::Common)),
        _ => None,
    }
}

/// Cut 30 §3: the next heir wakes. The blood passes (fading a tier when it never went live in a
/// banked run of the parent's; an empty blood slot takes the parent's born trait); the born slot is
/// offered three cards — the dead heir's born trait twisted (or a marked grudge, a kind that killed
/// three heirs running), then two fresh draws, all three with different `when`s — and wears the first
/// until `pick`. Deterministic: `Rng::derive(seed, "heir_traits" ^ heir)`.
pub fn wake(l: &mut LineageState) {
    wake_from(l, &shipped());
}

/// `wake` drawing from `pool` (the shipped table; a test or the measurement passes its own).
pub fn wake_from(l: &mut LineageState, pool: &[(Shape, bool)]) {
    if l.heirs.v == 0 {
        upgrade(l);
    }
    let h = &mut l.heirs;
    h.offer.clear();
    h.faded = None;
    if h.neutral {
        h.blood = None;
        h.born = None;
        return;
    }
    let parent = h.born.take();
    let (blood_live, born_live) = (std::mem::take(&mut h.blood_live), std::mem::take(&mut h.born_live));
    let _ = born_live;
    let was_cut = std::mem::take(&mut h.cut);
    if blood_open(l) {
        let h = &mut l.heirs;
        match h.blood {
            Some(b) if !blood_live => {
                h.blood = fade(b);
                h.faded = Some(format!("{} → {}", b.chip(), h.blood.map(|s| s.chip()).unwrap_or_else(|| "gone".into())));
            }
            Some(_) => {}
            None if !was_cut => h.blood = parent.filter(|p| p.when != When::Kin),
            None => {}
        }
    }
    if !arrived(l) {
        return;
    }
    let mut rng = offer_rng(l);
    let last_kill = l.graveyard.last().map(|g| g.cause.clone()).filter(|c| crate::defs::MONSTERS.iter().any(|m| m.kind == *c && !m.boss));
    // a marked grudge: the same kind killed the last three heirs
    let marked = l.graveyard.len() >= 3 && l.graveyard[l.graveyard.len() - 3..].iter().all(|g| Some(&g.cause) == last_kill.as_ref());
    let first = !l.facts.iter().any(|f| f.starts_with("trait:")) && parent.is_none();
    let bl = l.heirs.bloodline;
    let mut cards: Vec<Card> = Vec::new();
    let mut avoid: Vec<When> = Vec::new();
    if marked {
        cards.push(Card { shape: Shape::new(When::Kin, Gift::Fury, Cost::None).at(Tier::Marked), source: "marked".into() });
        avoid.push(When::Kin);
    } else if let Some(t) = parent.and_then(|p| twist_of(&mut rng, pool, p, &avoid)) {
        avoid.push(t.when);
        cards.push(Card { shape: t, source: "twist".into() });
    }
    for i in 0..3 {
        if cards.len() >= 3 {
            break;
        }
        let Some(s) = draw(&mut rng, pool, &avoid, bl, last_kill.is_some(), first && i == 0) else { break };
        avoid.push(s.when);
        cards.push(Card { shape: s, source: "fresh".into() });
    }
    // the fresh draws lead: card 1 (the default) is always a fresh draw
    cards.sort_by_key(|c| c.source != "fresh");
    // Cut 118 (owner amendment 2): one card always answers the killer — the last card gives way when none does
    let killer = l.graveyard.last().map(|g| g.cause.clone());
    let want = killer.as_deref().and_then(answer_gift).filter(|_| !l.feats.off);
    if let Some((gift, when)) = want {
        if !cards.iter().any(|c| c.shape.gift == gift) {
            // (the last fresh draw gives way — never a twist nor a marked card; a short offer gains a card)
            let at = if cards.len() >= 3 { cards.iter().rposition(|c| c.source == "fresh") } else { Some(cards.len()) };
            if let Some(i) = at {
                let avoid: Vec<When> = cards.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c.shape.when).collect();
                let n: Vec<Shape> = pool.iter().map(|(s, _)| *s).filter(|s| !s.is_twist() && s.gift == gift && !avoid.contains(&s.when)).collect();
                let pref: Vec<Shape> = n.iter().copied().filter(|s| Some(s.when) == when).collect();
                let pick = if pref.is_empty() { n } else { pref };
                if !pick.is_empty() {
                    let card = Card { shape: *rng.pick(&pick), source: "answer".into() };
                    if i < cards.len() {
                        cards[i] = card;
                    } else {
                        cards.push(card);
                    }
                }
            }
        }
    }
    // the heir order (a standing order, set once): which card succeeds — nobody is prompted
    let chosen = if l.feats.off { 0 } else { crate::feats::heir_choice(l, &cards, want.map(|w| w.0)) };
    let h = &mut l.heirs;
    h.kin = if cards.iter().any(|c| c.shape.when == When::Kin) { last_kill } else { h.kin.take().filter(|_| h.blood.is_some_and(|b| b.when == When::Kin)) };
    h.born = cards.get(chosen).or(cards.first()).map(|c| c.shape);
    h.offer = cards;
}

/// Cut 118 (owner amendment 2): the gift that answers a killer's tags, and the `when` it is best worn under —
/// a boss: Guard on bosses; poison, gas or a drain: Mend; a fast or swift foe: Quick; else Guard.
pub fn answer_gift(killer: &str) -> Option<(Gift, Option<When>)> {
    let d = crate::defs::MONSTERS.iter().find(|m| m.kind == killer)?;
    Some(if d.boss {
        (Gift::Guard, Some(When::Boss))
    } else if d.tags.iter().any(|t| matches!(*t, "gas" | "poison" | "drain" | "paralyse")) {
        (Gift::Mend, Some(When::Hurt))
    } else if d.tags.contains(&"fast") || d.speed > 10 {
        (Gift::Quick, None)
    } else {
        (Gift::Guard, Some(When::Crowded))
    })
}

/// The camp's pick of a card (by chip or head); refused when it is not on offer.
pub fn pick(l: &mut LineageState, name: &str) -> Result<(), String> {
    let c = l.heirs.offer.iter().find(|c| c.shape.chip() == name || c.shape.head() == name).ok_or_else(|| "not on offer".to_string())?;
    l.heirs.born = Some(c.shape);
    Ok(())
}

/// Cut the blood trait: the slot empties for this heir.
pub fn cut_blood(l: &mut LineageState) -> Result<(), String> {
    if l.heirs.blood.is_none() {
        return Err("no blood".into());
    }
    l.heirs.blood = None;
    l.heirs.cut = true;
    Ok(())
}

/// Choose the family's lean (once; heir 12 or an ascension).
pub fn set_bloodline(l: &mut LineageState, when: &str) -> Result<(), String> {
    if !bloodline_open(l) {
        return Err("not yet".into());
    }
    if l.heirs.bloodline.is_some() {
        return Err("chosen".into());
    }
    let w = When::parse(when).filter(|w| WHENS.contains(w)).ok_or_else(|| "no such when".to_string())?;
    l.heirs.bloodline = Some(w);
    l.chronicle_heir_at(&format!("the {} line", w.word()), None, None);
    Ok(())
}

/// The send settles the pick: the offer is spent.
pub fn on_send(l: &mut LineageState) {
    l.heirs.offer.clear();
}

/// A run ended: a gift that acted in a banked run keeps its tier.
pub fn on_run_end(l: &mut LineageState, run: &Run, banked: bool) {
    if banked {
        l.heirs.blood_live |= run.gift.blood > 0;
        l.heirs.born_live |= run.gift.born > 0;
    }
}

// ---------------------------------------------------------------------------------------------
// The run: what the heir wears, and the gift's state.

/// The heir's traits on a run (copied at the send; a replay's neutral counterfactual clears them).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Worn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blood: Option<Shape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub born: Option<Shape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kin: Option<String>,
    /// The max hp `frail` took at the send.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub frail_hp: i32,
    /// A shrine lifted the costs for this run.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub lifted: bool,
}

fn is_zero(n: &i32) -> bool {
    *n == 0
}

impl Worn {
    pub fn is_empty(&self) -> bool {
        self.blood.is_none() && self.born.is_none()
    }
    /// (slot 0 blood, 1 born, shape)
    pub fn slots(&self) -> impl Iterator<Item = (usize, Shape)> + '_ {
        self.blood.iter().map(|s| (0, *s)).chain(self.born.iter().map(|s| (1, *s)))
    }
    pub fn has_cost(&self, c: Cost) -> bool {
        !self.lifted && self.slots().any(|(_, s)| s.cost == c)
    }
    /// The chips worn (`wrathful · frail`), blood first.
    pub fn chips(&self) -> Vec<String> {
        self.slots().map(|(_, s)| s.chip()).collect()
    }
}

/// The gift's per-run state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GiftRun {
    /// A worn `when` holds at this action (`if: gift live`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub live: bool,
    /// Actions the blood / born gift acted on this run.
    #[serde(default, skip_serializing_if = "is_zero_u")]
    pub blood: u32,
    #[serde(default, skip_serializing_if = "is_zero_u")]
    pub born: u32,
    /// The floor whose first live turn was stamped (per slot).
    #[serde(default)]
    pub stamped: [u32; 2],
    /// The mark for this action's trace row (`fury +1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark: Option<String>,
}

fn is_zero_u(n: &u32) -> bool {
    *n == 0
}

/// The live modifiers on the hero (`Hero::atk`, `blunt`, `speed`, `def` read them). Recomputed at
/// every hero action from the worn traits and the context.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Mods {
    #[serde(default)]
    pub fury: i32,
    #[serde(default)]
    pub guard: i32,
    #[serde(default)]
    pub quick: i32,
    #[serde(default)]
    pub slow: i32,
}

impl Mods {
    pub fn is_zero(&self) -> bool {
        *self == Mods::default()
    }
}

/// The send: the heir's traits go on the run; `frail` takes its max hp.
pub fn wear(run: &mut Run, l: &LineageState) {
    run.worn = Worn::default();
    if l.heirs.neutral || l.heirs.v == 0 && l.heirs.born.is_none() {
        return;
    }
    run.worn.blood = l.heirs.blood;
    run.worn.born = l.heirs.born;
    run.worn.kin = l.heirs.kin.clone();
    if run.worn.has_cost(Cost::Frail) {
        let h = &mut run.hero;
        let cut = (h.max_hp * FRAIL_PCT / 100).max(1);
        h.max_hp -= cut;
        h.max_hp_base = h.max_hp;
        h.hp = h.hp.min(h.max_hp);
        run.worn.frail_hp = cut;
    }
    run.hero.gift = if run.worn.has_cost(Cost::Slow) { Mods { slow: 1, ..Mods::default() } } else { Mods::default() };
}

/// The neutral heir's counterfactual on a run in flight (the verdict's replays): the traits off, the
/// frail hp back.
pub fn neutralize(run: &mut Run) {
    let back = run.worn.frail_hp;
    run.hero.max_hp += back;
    run.hero.max_hp_base += back;
    run.hero.hp += back;
    run.worn = Worn::default();
    run.gift = GiftRun::default();
    run.hero.gift = Mods::default();
}

/// Does `when` hold now?
pub fn holds(run: &Run, w: When, kin: Option<&str>, foes: &[usize]) -> bool {
    let h = &run.hero;
    match w {
        When::Hurt => h.hp_pct() < HURT_PCT,
        When::Crowded => foes.iter().filter(|&&i| run.monsters[i].pos.cheb(h.pos) <= 1).count() >= 2,
        When::Boss => foes.iter().any(|&i| run.monsters[i].is_boss()),
        When::Quiet => foes.is_empty(),
        When::Deep => run.depth >= DEEP_FROM,
        When::First => run.floor_turn < FIRST_TURNS * crate::engine::TICKS_PER_TURN,
        When::Always => true,
        When::Kin => kin.is_some_and(|k| foes.iter().any(|&i| run.monsters[i].kind == k)),
    }
}

/// The gift acted: the trace's mark, the fact on its first live turn, the stamp on the floor's first.
pub fn fire(run: &mut Run, cx: &mut Ctx, slot: usize, s: Shape) {
    if slot == 0 {
        run.gift.blood += 1;
    } else {
        run.gift.born += 1;
    }
    let m = s.mark();
    run.gift.mark = Some(match run.gift.mark.take() {
        Some(prev) if prev != m => format!("{prev} · {m}"),
        _ => m,
    });
    crate::facts::learn(run, cx, s.fact());
    if run.gift.stamped[slot.min(1)] != run.depth {
        run.gift.stamped[slot.min(1)] = run.depth;
        crate::chronicle::callout(run, cx, s.gift.stamp());
    }
}

/// Cut 30 §1: at every hero action, before the rows are read — which gifts are live, the hero's live
/// modifiers, `mend`'s hp. Never picks a verb.
pub fn on_action(run: &mut Run, cx: &mut Ctx, foes: &[usize]) {
    run.gift.live = false;
    run.gift.mark = None;
    if run.worn.is_empty() {
        return;
    }
    let mut m = Mods { slow: run.worn.has_cost(Cost::Slow) as i32, ..Mods::default() };
    let kin = run.worn.kin.clone();
    let slots: Vec<(usize, Shape)> = run.worn.slots().collect();
    for (slot, s) in slots {
        if !holds(run, s.when, kin.as_deref(), foes) {
            continue;
        }
        run.gift.live = true;
        let step = s.step();
        match s.gift {
            Gift::Fury => m.fury = m.fury.max(step),
            Gift::Guard => m.guard = m.guard.max(step),
            Gift::Quick => m.quick = m.quick.max(quick_speed(step)),
            Gift::Mend => {
                let h = &mut run.hero;
                if h.hp <= 0 || h.hp >= h.max_hp {
                    continue;
                }
                h.hp = (h.hp + mend_hp(step)).min(h.max_hp);
            }
            // (these act at their verb: `rest_extra`, `after_action`, `after_drink`)
            _ => continue,
        }
        fire(run, cx, slot, s);
    }
    run.hero.gift = m;
}

/// The live shape of `gift` worn now (its slot), if its context holds.
fn live_gift(run: &Run, gift: Gift, foes: &[usize]) -> Option<(usize, Shape)> {
    let kin = run.worn.kin.as_deref();
    run.worn.slots().filter(|(_, s)| s.gift == gift).find(|(_, s)| holds(run, s.when, kin, foes))
}

/// `rested`: the extra hp a rest heals now (0 without the gift live).
pub fn rest_extra_now(run: &mut Run, cx: &mut Ctx) -> i32 {
    if run.worn.is_empty() {
        return 0;
    }
    match live_gift(run, Gift::Rested, &[]) {
        Some((slot, s)) => {
            fire(run, cx, slot, s);
            rest_extra(s.step())
        }
        None => 0,
    }
}

/// `thin`: a heal potion's heal, in percent.
pub fn heal_pct(run: &Run) -> i32 {
    if run.worn.has_cost(Cost::Thin) {
        THIN_PCT
    } else {
        100
    }
}

/// `dim`: sight lost here.
pub fn dim(run: &Run) -> i32 {
    if run.worn.has_cost(Cost::Dim) && run.biome() == crate::descent::Biome::Deep {
        DIM_SIGHT
    } else {
        0
    }
}

/// `sure` and `light hands`, after the action: energy back for a step away / home, or a pick up
/// that took something.
pub fn after_action(run: &mut Run, cx: &mut Ctx, verb: &crate::rules::Verb, took: bool, foes: &[usize]) {
    if run.worn.is_empty() || run.over.is_some() {
        return;
    }
    let away = matches!(verb.v.as_str(), "retreat" | "back_corridor" | "kite") || (run.homeward.is_some() && matches!(verb.v.as_str(), "return" | "bank" | "explore"));
    if away {
        if let Some((slot, s)) = live_gift(run, Gift::Sure, foes) {
            run.hero.energy += crate::engine::ACT_ENERGY * sure_refund(s.step()) / 100;
            fire(run, cx, slot, s);
        }
    }
    if took && verb.v == "pick_up" {
        if let Some((slot, s)) = live_gift(run, Gift::LightHands, foes) {
            run.hero.energy += crate::engine::ACT_ENERGY;
            fire(run, cx, slot, s);
        }
    }
}

/// `iron gut`: a malevolent drink harms half.
pub fn after_drink(run: &mut Run, cx: &mut Ctx, kind: &str) {
    if run.worn.is_empty() || !matches!(kind, "poison" | "confusion") {
        return;
    }
    if let Some((slot, s)) = live_gift(run, Gift::IronGut, &[]) {
        match kind {
            "poison" => run.hero.poison = ((run.hero.poison.0 + 1) / 2, run.hero.poison.1 / 2),
            _ => run.hero.confused /= 2,
        }
        fire(run, cx, slot, s);
    }
}

/// The shrine's `pray trait`: the heir's costs lifted for this run (`None` with no cost worn).
pub fn shrine_lift(run: &mut Run) -> Option<String> {
    if run.worn.lifted || !run.worn.slots().any(|(_, s)| s.cost != Cost::None) {
        return None;
    }
    run.worn.lifted = true;
    let back = std::mem::take(&mut run.worn.frail_hp);
    run.hero.max_hp += back;
    run.hero.max_hp_base += back;
    run.hero.gift.slow = 0;
    Some("cost lifted".into())
}

// ---------------------------------------------------------------------------------------------
// Conditions: `if: trait <head>` (the heir wears it) and `if: gift live` (a worn gift's context
// holds now), each gated by the trait's fact (learned the first time its gift goes live).

pub const COND_TRAIT: &str = "trait";
pub const COND_LIVE: &str = "gift_live";

/// The condition is usable by this lineage (its fact is known).
pub fn cond_usable(facts: &std::collections::BTreeSet<String>, c: &crate::rules::Cond) -> bool {
    match c.k.as_str() {
        COND_TRAIT => c.t.as_deref().is_some_and(|t| facts.contains(&format!("trait:{t}"))),
        COND_LIVE => facts.iter().any(|f| f.starts_with("trait:")),
        _ => true,
    }
}

pub fn cond_holds(run: &Run, cx: &Ctx, c: &crate::rules::Cond) -> bool {
    if !cond_usable(cx.facts, c) {
        return false;
    }
    match c.k.as_str() {
        COND_TRAIT => c.t.as_deref().is_some_and(|t| run.worn.slots().any(|(_, s)| s.head() == t)),
        COND_LIVE => run.gift.live,
        _ => false,
    }
}

/// The row's reason when the condition does not hold (≤ 3 words).
pub fn cond_reason(cx: &Ctx, c: &crate::rules::Cond) -> String {
    if !cond_usable(cx.facts, c) {
        return "locked cond".into();
    }
    match c.k.as_str() {
        COND_TRAIT => format!("no {}", c.t.as_deref().unwrap_or("trait")),
        _ => "no gift live".into(),
    }
}

/// The editor's tokens: `trait <head>` per learned trait, `gift live` once any is.
pub fn vocab_conds(l: &LineageState, conds: &mut Vec<crate::rules::Cond>) {
    let mut any = false;
    for f in &l.facts {
        if let Some(head) = f.strip_prefix("trait:") {
            conds.push(crate::rules::Cond::t(COND_TRAIT, head));
            any = true;
        }
    }
    if any {
        conds.push(crate::rules::Cond::flag(COND_LIVE));
    }
}

// ---------------------------------------------------------------------------------------------
// The wire.

/// A card or a worn trait, as the camp shows it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TraitCard {
    /// `wrathful · frail`
    pub chip: String,
    /// `wrathful`
    pub head: String,
    /// `[hurt] → fury +1 · frail`, or `[hurt] → fury ? · frail` while its fact is unknown
    pub formula: String,
    pub when: String,
    pub gift: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub cost: String,
    /// common · uncommon · rare · marked (the chip's rim)
    pub tier: String,
    /// fresh · twist · marked (a card), blood · born (worn)
    pub source: String,
    /// The gift's size is a known fact.
    pub learned: bool,
}

/// `Lineage.heir_traits`: the heir's slots, the wake's cards, the family's lean.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HeirTraitsWire {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blood: Option<TraitCard>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub born: Option<TraitCard>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub offer: Vec<TraitCard>,
    /// The blood slot is open (heir 5); the bloodline may be chosen (heir 12 / an ascension).
    pub blood_open: bool,
    pub bloodline_open: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bloodline: Option<String>,
    /// The last wake's fade (`wrathful · frail → gone`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faded: Option<String>,
    /// A grudge trait's kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kin: Option<String>,
}

pub fn card(s: Shape, source: &str, facts: &std::collections::BTreeSet<String>) -> TraitCard {
    let learned = facts.contains(&s.fact());
    TraitCard {
        chip: s.chip(),
        head: s.head().into(),
        formula: s.formula(learned),
        when: s.when.word().into(),
        gift: s.gift.word().into(),
        cost: s.cost.word().into(),
        tier: format!("{:?}", s.tier).to_lowercase(),
        source: source.into(),
        learned,
    }
}

/// `None` for the neutral heir with nothing to show.
pub fn wire(l: &LineageState) -> Option<HeirTraitsWire> {
    let h = &l.heirs;
    if h.neutral || (h.blood.is_none() && h.born.is_none() && h.offer.is_empty() && !arrived(l)) {
        return None;
    }
    Some(HeirTraitsWire {
        blood: h.blood.map(|s| card(s, "blood", &l.facts)),
        born: h.born.map(|s| card(s, "born", &l.facts)),
        offer: h.offer.iter().map(|c| card(c.shape, &c.source, &l.facts)).collect(),
        blood_open: blood_open(l),
        bloodline_open: bloodline_open(l),
        bloodline: h.bloodline.map(|w| w.word().into()),
        faded: h.faded.clone(),
        kin: h.kin.clone(),
    })
}

/// The heir's chip for the strip (`Lineage.trait`): the born trait's, else the blood's; "" neutral.
pub fn chip(l: &LineageState) -> String {
    l.heirs.born.or(l.heirs.blood).map(|s| s.chip()).unwrap_or_default()
}

/// A stable text of the heir's traits (the forecast's cache key; the vs line's `heir` part).
pub fn key(l: &LineageState) -> String {
    let h = &l.heirs;
    format!("{:?}|{:?}|{:?}|{}", h.blood, h.born, h.kin, h.neutral)
}
