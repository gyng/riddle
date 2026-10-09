//! The persistent descent: biome order, bosses, grudge monsters. Floors regenerate per run.
use serde::{Deserialize, Serialize};

/// Cut 7: reaching D34's stairs is the ending (Cut 3: D31; D16 was v1's placeholder). The
/// Warrens run to D8 so the wall arrives when the player has a policy, not a preset.
pub const ENDING_DEPTH: u32 = 34;
/// Cut 7 §1: the lieutenant's floor — the Goblin Captain at D5 (rallies once, no shield wall).
pub const LIEUTENANT_DEPTH: u32 = 5;
/// Cut 7 §3: the band situations (thief's den, gas lock, captive gate, crypt's hunger).
pub const SITUATION_DEPTHS: [(&str, u32); 4] = [("den", 3), ("lock", 6), ("captive", 9), ("hunger", 12)];
/// Cut 3: the hero's sight radius in a lit biome; the Deep is dark (`vision_for`).
pub const VISION_LIT: i32 = 7;
pub const VISION_DARK: i32 = 4;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Biome {
    Warrens,
    /// Cut 16 §3: the Warrens' deep end, D5–8 (the Captain's floor opens it, the Warlord's
    /// closes it): red clay and torches, monkeys and archers, no rats.
    Burrows,
    Fens,
    Crypt,
    // Cut 3: biomes 4–6, each breaking the program that cleared the last one.
    Foundry,
    Deep,
    Sanctum,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Warrens => "warrens",
            Biome::Burrows => "burrows",
            Biome::Fens => "fens",
            Biome::Crypt => "crypt",
            Biome::Foundry => "foundry",
            Biome::Deep => "deep",
            Biome::Sanctum => "sanctum",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Biome::Warrens => "the Warrens",
            Biome::Burrows => "the Burrows",
            Biome::Fens => "the Fens",
            Biome::Crypt => "the Crypt",
            Biome::Foundry => "the Foundry",
            Biome::Deep => "the Deep",
            Biome::Sanctum => "the Sanctum",
        }
    }
    pub const ALL: [Biome; 7] = [Biome::Warrens, Biome::Burrows, Biome::Fens, Biome::Crypt, Biome::Foundry, Biome::Deep, Biome::Sanctum];
    /// Cave floors (cellular, with water): the Fens and the Deep.
    pub fn is_cave(self) -> bool {
        matches!(self, Biome::Fens | Biome::Deep)
    }
    /// Sight radius on this biome's floors (Cut 3: the Deep is dark).
    pub fn vision(self) -> i32 {
        match self {
            Biome::Deep => VISION_DARK,
            _ => VISION_LIT,
        }
    }
}

/// Cut 7: Warrens D1–8 (captain D5, Warlord D8); Cut 16 §3: D5–8 are the Burrows; Fens D9–13, Crypt D14–18, Foundry D19–23,
/// Deep D24–28, Sanctum D29–33, the bottom at D34.
pub fn biome_for(depth: u32) -> Biome {
    match depth {
        0..=4 => Biome::Warrens,
        5..=8 => Biome::Burrows,
        9..=13 => Biome::Fens,
        14..=18 => Biome::Crypt,
        19..=23 => Biome::Foundry,
        24..=28 => Biome::Deep,
        _ => Biome::Sanctum,
    }
}

/// The first floor of a biome.
pub fn biome_first(biome: Biome) -> u32 {
    match biome {
        Biome::Warrens => 1,
        Biome::Burrows => 5,
        Biome::Fens => 9,
        Biome::Crypt => 14,
        Biome::Foundry => 19,
        Biome::Deep => 24,
        Biome::Sanctum => 29,
    }
}

/// The boss floors in order (Cut 6 §5: `Lineage.counters` walks them).
pub const BOSS_DEPTHS: [(&str, u32); 6] = [("goblin_warlord", 8), ("bloat_mother", 13), ("lich", 18), ("foundry_master", 23), ("lurker_queen", 28), ("mirror_king", 33)];

pub fn boss_for(depth: u32) -> Option<&'static str> {
    BOSS_DEPTHS.iter().find(|(_, d)| *d == depth).map(|(k, _)| *k)
}

/// The depth of a boss kind.
pub fn boss_depth(kind: &str) -> Option<u32> {
    BOSS_DEPTHS.iter().find(|(k, _)| *k == kind).map(|(_, d)| *d)
}

/// Cut 7 §1: the lieutenant on a floor (the Goblin Captain at D5), placed like a boss but
/// no boss: no counter fact, no marks, a bestiary entry like any other kind.
pub fn lieutenant_for(depth: u32) -> Option<&'static str> {
    (depth == LIEUTENANT_DEPTH).then_some("goblin_captain")
}

/// Cut 7: the content depth — the Cut 3–6 tuning (spawn groups, stat growth, item budgets,
/// item depths) was written against Warrens D1–5 / Fens D6–10 / …; the Warrens' three new
/// floors sit at the old D5's numbers and everything below keeps its old tuning depth.
pub fn tier_depth(depth: u32) -> u32 {
    match depth {
        0..=5 => depth,
        6..=8 => 5,
        _ => depth - 3,
    }
}

/// Cut 26 §1: the six bands below the Warrens (first floor, last floor), and the base order of
/// their biomes (the order every lineage played before Cut 26, and the default route's).
pub const BANDS: [(u32, u32); 6] = [(5, 8), (9, 13), (14, 18), (19, 23), (24, 28), (29, 33)];
pub const BASE_ORDER: [Biome; 6] = [Biome::Burrows, Biome::Fens, Biome::Crypt, Biome::Foundry, Biome::Deep, Biome::Sanctum];
/// Cut 26 §1: the forks — the stairs into a band (D4's stairs open the D5 fork): the near stair
/// (the band's own biome) or the far one (the next band's, early).
pub const FORKS: [u32; 5] = [5, 9, 14, 19, 24];

/// Cut 26 §3 (the contract's fallback: "if no tuning holds at D9 in the budget, ship the D5 fork
/// only and record it"): the forks a hero can see and a set can take. The descent's routes are
/// all playable (the sims, the gate table's samples).
/// Cut 116 §2 (the owner: vary the descent): the D9 and D14 forks open beside D5, so heirs take
/// different biome orders (`examples/varied_descent.rs` measures each new fork's lanes).
pub const OPEN_FORKS: [u32; 3] = [5, 9, 14];

/// Cut 29 §1: the forks a lineage sees — `OPEN_FORKS`, and the D9 fork once an oath gave `route2`.
pub fn fork_open_for(route2: bool, fork: u32) -> bool {
    OPEN_FORKS.contains(&fork) || (route2 && fork == 9)
}

/// Cut 116 §1: a band boss's affix — drawn per heir from the lineage seed, shown on the floor chart
/// and the boss bar before the fight (`Warlord · armoured`). Each changes which answer beats the wall
/// and pays for itself in the boss's hp, so an answered affix is no harder than the plain boss and an
/// unanswered one is a wall the scars and drills still wear down.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Affix {
    /// +1 armour: poison and fire go round it.
    Armoured,
    /// +2 speed: slowed, or kept at range.
    Swift,
    /// +1 hp a second while awake and unpoisoned: poison stops it, focus outpaces it.
    Regenerating,
    /// A full guard of escorts: a corridor or a pack break thins it.
    Brood,
    /// +2 damage, +3 speed under half hp: burst him through it, or heal early.
    Enraged,
    /// Melee hits heal him: poison or range.
    Vampiric,
}

impl Affix {
    pub const ALL: [Affix; 6] = [Affix::Armoured, Affix::Swift, Affix::Regenerating, Affix::Brood, Affix::Enraged, Affix::Vampiric];
    /// The label word (`Warlord · armoured`).
    pub fn word(self) -> &'static str {
        match self {
            Affix::Armoured => "armoured",
            Affix::Swift => "swift",
            Affix::Regenerating => "regenerating",
            Affix::Brood => "brood",
            Affix::Enraged => "enraged",
            Affix::Vampiric => "vampiric",
        }
    }
    pub fn from_word(w: &str) -> Option<Affix> {
        Affix::ALL.into_iter().find(|a| a.word() == w)
    }
    /// What it does (a tooltip).
    pub fn effect(self) -> &'static str {
        match self {
            Affix::Armoured => "+1 armour · less hp",
            Affix::Swift => "+2 speed · less hp",
            Affix::Regenerating => "heals while unpoisoned · less hp",
            Affix::Brood => "full escort · less hp",
            Affix::Enraged => "hits harder under half hp · less hp",
            Affix::Vampiric => "melee hits heal him · less hp",
        }
    }
    /// What answers it (a tooltip; the tactics and items that arrive by its wall).
    /// (Cut 117 §3: the packages that break it — `affix_breakers`; the Warlord's are stances)
    pub fn counter(self) -> &'static str {
        match self {
            Affix::Armoured => "boss focus · Hunter",
            Affix::Swift => "corridor fighting · kite archers",
            Affix::Regenerating => "Hunter · gas step",
            Affix::Brood => "pack break · corridor fighting",
            Affix::Enraged => "kite archers · Hunter",
            Affix::Vampiric => "kite archers · gas step",
        }
    }
    /// The share of his max hp the affix leaves (its price).
    pub fn hp_pct(self) -> i32 {
        match self {
            Affix::Armoured => AFFIX_HP[0],
            Affix::Swift => AFFIX_HP[1],
            Affix::Regenerating => AFFIX_HP[2],
            Affix::Brood => AFFIX_HP[3],
            Affix::Enraged => AFFIX_HP[4],
            Affix::Vampiric => AFFIX_HP[5],
        }
    }
}

/// Cut 117 §3 (Cut 116's gap: the affix changed the best answer on 1 of 20 boss × affix pairs): each affix is
/// broken by its own answers — never the plain walls' usual best (Guarded, Bold) past the Warlord — and a boss whose
/// affix stands unbroken is warded (`AFFIX_WARD`: the hero's blows do less) and furious (`AFFIX_FURY`: his land
/// harder); broken, he takes `AFFIX_BREAK_DEALT` % more and deals `AFFIX_BREAK_TAKEN` % less (`turn::affix_dealt`,
/// `turn::affix_taken`). The first answer is the affix's own, the second the one that arrives when it has not:
/// armoured → boss focus · Hunter (the blows find the seams), swift → corridor fighting · kite archers (no room to
/// circle), brood → pack break · corridor fighting (the escort split), vampiric → kite archers · gas step (range or
/// poison through the drain), regenerating → Hunter · gas step (marked, burned, he cannot knit), enraged → kite
/// archers · Hunter (kept at range through his frenzy). At the Warlord no tactic has arrived yet (they come with his
/// fall): a stance breaks his — Hunter (armoured, enraged), Guarded (swift), Bold (brood); never Steady, the default.
pub fn affix_breakers(boss: &str, a: Affix) -> &'static [&'static str] {
    if boss == "goblin_warlord" {
        return match a {
            Affix::Armoured | Affix::Enraged => &["hunter"],
            Affix::Swift => &["guarded"],
            _ => &["bold"],
        };
    }
    match a {
        Affix::Armoured => &["boss_focus", "hunter"],
        Affix::Swift => &["corridor_fighting", "kite_archers"],
        Affix::Brood => &["pack_break", "corridor_fighting"],
        Affix::Vampiric => &["kite_archers", "gas_step"],
        Affix::Regenerating => &["hunter", "gas_step"],
        Affix::Enraged => &["kite_archers", "hunter"],
    }
}
pub const AFFIX_BREAK_DEALT: i32 = 250;
pub const AFFIX_BREAK_TAKEN: i32 = 65;
/// Cut 117 §3: an affixed boss's ward while his affix stands unbroken — the hero's blows on him do this % less.
pub const AFFIX_WARD: i32 = 20;
/// Cut 117 §3: and his blows land this % harder while it stands.
pub const AFFIX_FURY: i32 = 20;

/// Cut 116 §1: each affix's hp price (percent of max hp kept), in `Affix::ALL` order.
pub const AFFIX_HP: [i32; 6] = [95, 95, 95, 85, 95, 95];
/// Cut 116 §1: the brood's escort chance per tile round the boss (a boss's own guard is 40 %).
pub const BROOD_GUARD: u32 = 75;

/// Cut 116 §1: the affixes a band boss can draw (each boss's own ground: the Mother is slow and
/// gassy, the Lich reflects arrows, the Master reflects blades, the Queen already broods). The Mirror
/// King draws none: the bottom stays the bottom.
pub fn affix_pool(boss: &str) -> &'static [Affix] {
    match boss {
        "goblin_warlord" => &[Affix::Armoured, Affix::Swift, Affix::Brood, Affix::Enraged],
        "bloat_mother" => &[Affix::Armoured, Affix::Swift, Affix::Regenerating, Affix::Brood],
        "lich" => &[Affix::Regenerating, Affix::Enraged, Affix::Vampiric, Affix::Swift],
        "foundry_master" => &[Affix::Armoured, Affix::Regenerating, Affix::Enraged, Affix::Brood],
        "lurker_queen" => &[Affix::Vampiric, Affix::Swift, Affix::Regenerating, Affix::Enraged],
        _ => &[],
    }
}

/// Cut 116 §1: the affix heir `heir` of the lineage seeded `seed` meets on `boss` — a pure draw (no
/// rng stream advances): the same heir always meets the same descent, the next heir another.
pub fn boss_affix(seed: u64, heir: u32, boss: &str) -> Option<Affix> {
    let pool = affix_pool(boss);
    if pool.is_empty() {
        return None;
    }
    let i = BOSS_DEPTHS.iter().position(|(k, _)| *k == boss).unwrap_or(0) as u64;
    let h = crate::rng::splitmix(seed ^ (u64::from(heir) << 24) ^ (i << 56) ^ 0xA77_1C5E5);
    Some(pool[(h % pool.len() as u64) as usize])
}

/// Cut 116 §1: every band boss's affix for this heir (descent order of the base route).
pub fn heir_affixes(seed: u64, heir: u32) -> Vec<(String, Affix)> {
    BOSS_DEPTHS.iter().filter_map(|(k, _)| boss_affix(seed, heir, k).map(|a| (k.to_string(), a))).collect()
}

/// Cut 116 §3: a wandering champion — once per band at most, on a non-boss floor, a named foe from
/// another band's roster (the band above's, a foe the heir has beaten the like of): a story beat, a
/// grudge if it kills, never a wall.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guest {
    pub kind: String,
    pub name: String,
    pub depth: u32,
}

/// Cut 116 §3: the share of heirs × bands that meet a guest.
pub const GUEST_PCT: u64 = 50;

/// The guest roster of the band whose biome is `b`: kinds from another band (no captives, no water
/// kinds, none native to `b`).
pub fn guest_roster(b: Biome) -> &'static [&'static str] {
    match b {
        Biome::Burrows => &["skeleton", "pink_jelly"],
        Biome::Fens => &["monkey", "goblin_conjurer", "skeleton"],
        Biome::Crypt => &["goblin_archer", "pink_jelly", "jackal"],
        Biome::Foundry => &["wraith", "ghoul", "ogre"],
        Biome::Deep => &["smith", "bell_sentinel", "wraith"],
        Biome::Sanctum => &["cave_troll", "siren", "smith"],
        Biome::Warrens => &[],
    }
}

/// Cut 116 §3: the guest heir `heir` meets in the band at `band` (0..6) on `route`, if any: a floor
/// strictly inside the band (not its first, not the boss's), a kind from another band's roster, a
/// name — a pure draw like the affixes.
pub fn guest_for(seed: u64, heir: u32, route: Route, band: usize) -> Option<Guest> {
    let (first, last) = *BANDS.get(band)?;
    let h = crate::rng::splitmix(seed ^ (u64::from(heir) << 24) ^ ((band as u64) << 48) ^ 0x6E57_C4A3);
    if h % 100 >= GUEST_PCT {
        return None;
    }
    let roster = guest_roster(route.order()[band]);
    if roster.is_empty() || last <= first + 1 {
        return None;
    }
    let depth = first + 1 + ((h >> 8) % u64::from(last - first - 1)) as u32;
    let kind = roster[((h >> 16) % roster.len() as u64) as usize];
    let mut rng = crate::rng::Rng::new(h >> 24);
    Some(Guest { kind: kind.into(), name: grudge_name(&mut rng), depth })
}

/// A biome's boss (on the last floor of the band it sits in).
pub fn biome_boss(b: Biome) -> Option<&'static str> {
    match b {
        Biome::Warrens => None,
        Biome::Burrows => Some("goblin_warlord"),
        Biome::Fens => Some("bloat_mother"),
        Biome::Crypt => Some("lich"),
        Biome::Foundry => Some("foundry_master"),
        Biome::Deep => Some("lurker_queen"),
        Biome::Sanctum => Some("mirror_king"),
    }
}

/// The biome whose boss `kind` is.
pub fn boss_biome(kind: &str) -> Option<Biome> {
    Biome::ALL.into_iter().find(|b| biome_boss(*b) == Some(kind))
}

/// Cut 26 §1: the descent's route — the base order with non-overlapping adjacent swaps, one bit
/// per fork (`FORKS[i]`: band `i` and band `i + 1` trade biomes). Taking a far stair defers the
/// near biome to the next band, which then has no fork: no two adjacent bits. 13 routes; every
/// biome within one band of its own. `Route(0)` is the base order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Route(pub u8);

impl Route {
    pub const BASE: Route = Route(0);
    pub fn is_base(&self) -> bool {
        self.0 == 0
    }
    pub fn valid(self) -> bool {
        self.0 < 1 << FORKS.len() && self.0 & (self.0 >> 1) == 0
    }
    /// Every route, the base first.
    pub fn all() -> Vec<Route> {
        (0..1u8 << FORKS.len()).map(Route).filter(|r| r.valid()).collect()
    }
    /// The route whose far stairs are taken at these fork depths.
    pub fn from_forks(forks: &[u32]) -> Result<Route, String> {
        let mut bits = 0u8;
        for f in forks {
            let i = FORKS.iter().position(|x| x == f).ok_or_else(|| format!("no fork at D{f}"))?;
            bits |= 1 << i;
        }
        let r = Route(bits);
        if !r.valid() {
            return Err("forks overlap".into());
        }
        Ok(r)
    }
    /// The fork depths whose far stair this route takes, ascending.
    pub fn forks(self) -> Vec<u32> {
        FORKS.iter().enumerate().filter(|(i, _)| self.0 & (1 << i) != 0).map(|(_, f)| *f).collect()
    }
    pub fn far_at(self, fork: u32) -> bool {
        FORKS.iter().position(|f| *f == fork).is_some_and(|i| self.0 & (1 << i) != 0)
    }
    /// Whether the stairs into `fork` offer a choice on this route: not when the fork above
    /// took its far stair (the deferred biome is taken here).
    pub fn fork_open(self, fork: u32) -> bool {
        match FORKS.iter().position(|f| *f == fork) {
            Some(0) => true,
            Some(i) => self.0 & (1 << (i - 1)) == 0,
            None => false,
        }
    }
    /// This route with the stair at `fork` set (`far`): a far stair drops the neighbours' far
    /// stairs it overlaps.
    pub fn with(self, fork: u32, far: bool) -> Route {
        let Some(i) = FORKS.iter().position(|f| *f == fork) else { return self };
        let mut b = self.0 & !(1 << i);
        if far {
            b |= 1 << i;
            if i > 0 {
                b &= !(1 << (i - 1));
            }
            b &= !(1 << (i + 1));
        }
        Route(b & ((1 << FORKS.len()) - 1))
    }
    /// The biomes of the six bands in order.
    pub fn order(self) -> [Biome; 6] {
        let mut o = BASE_ORDER;
        for i in 0..FORKS.len() {
            if self.0 & (1 << i) != 0 {
                o.swap(i, i + 1);
            }
        }
        o
    }
    /// The band (0..6) a depth sits in, below the Warrens.
    pub fn band_of(depth: u32) -> Option<usize> {
        BANDS.iter().position(|(a, b)| (*a..=*b).contains(&depth)).or((depth > BANDS[5].1).then_some(5))
    }
    pub fn biome(self, depth: u32) -> Biome {
        match Route::band_of(depth) {
            None => Biome::Warrens,
            Some(i) => self.order()[i],
        }
    }
    /// The band a biome sits in on this route: (first, last).
    pub fn span(self, b: Biome) -> (u32, u32) {
        match self.order().iter().position(|x| *x == b) {
            Some(i) => BANDS[i],
            None => (1, 4),
        }
    }
    pub fn first(self, b: Biome) -> u32 {
        self.span(b).0
    }
    /// The boss on this floor: the band's biome's, on the band's last floor.
    pub fn boss(self, depth: u32) -> Option<&'static str> {
        let i = Route::band_of(depth)?;
        (BANDS[i].1 == depth).then(|| biome_boss(self.order()[i])).flatten()
    }
    pub fn boss_depth(self, kind: &str) -> Option<u32> {
        boss_biome(kind).map(|b| self.span(b).1)
    }
    /// The bosses in descent order (the walls a run meets).
    pub fn bosses(self) -> Vec<(&'static str, u32)> {
        self.order().iter().enumerate().filter_map(|(i, b)| biome_boss(*b).map(|k| (k, BANDS[i].1))).collect()
    }
    /// The Burrows' lieutenant holds their first floor wherever they sit.
    pub fn lieutenant(self, depth: u32) -> Option<&'static str> {
        (depth == self.first(Biome::Burrows)).then_some("goblin_captain")
    }
    /// The route as far as the floor at `depth` (the forks at or above it): what a waystone
    /// there is lit for.
    pub fn prefix(self, depth: u32) -> Route {
        let mut b = 0u8;
        for (i, f) in FORKS.iter().enumerate() {
            if *f <= depth {
                b |= self.0 & (1 << i);
            }
        }
        Route(b)
    }
    /// `D5 fens · D14 crypt` — each far stair and the biome it opens (the chip line).
    pub fn label(self) -> String {
        self.forks().iter().map(|f| format!("D{f} {}", self.biome(*f).name())).collect::<Vec<_>>().join(" · ")
    }
    /// `D5–8 · the Fens` for each band (the report's lanes).
    pub fn lanes(self) -> Vec<(u32, u32, Biome)> {
        BANDS.iter().zip(self.order()).map(|((a, b), o)| (*a, *b, o)).collect()
    }
}

/// A named monster that killed an heir; lives on the floor it killed on, +10% stats.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Grudge {
    pub kind: String,
    pub name: String,
    pub depth: u32,
    pub heir: u32,
    /// Cut 19 §5: killed once (`X is avenged.`); a later kill of the named foe reads `X slain.`
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub avenged: bool,
    /// Cut 29 §6 (AX: `Greth is avenged` while Greth was his ally): tamed — the grudge closes as
    /// tamed, never avenged; it lives on no floor any more.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tamed: bool,
    /// Cut 26 §1: the biome of the floor it killed on — it lives on that biome's floor at that
    /// depth, whatever route sends a run there (`None`: the base order's, a save from before).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biome: Option<Biome>,
}

impl Grudge {
    /// Cut 26 §1: whether the grudge lives on this floor (its depth, its biome).
    pub fn lives_on(&self, depth: u32, biome: Biome) -> bool {
        self.depth == depth && self.biome.unwrap_or(biome_for(self.depth)) == biome
    }
}

const SYL_A: [&str; 10] = ["Gr", "Sk", "Vr", "Th", "Mor", "Ash", "Ul", "Kr", "Zel", "Dr"];
const SYL_B: [&str; 8] = ["ak", "ix", "ul", "eth", "og", "ar", "im", "usk"];

pub fn grudge_name(rng: &mut crate::rng::Rng) -> String {
    format!("{}{}", rng.pick(&SYL_A), rng.pick(&SYL_B))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn biome_order_fixed() {
        assert_eq!(biome_for(1), Biome::Warrens);
        assert_eq!(biome_for(4), Biome::Warrens);
        for d in 5..=8 {
            assert_eq!(biome_for(d), Biome::Burrows);
        }
        assert_eq!(biome_first(Biome::Burrows), LIEUTENANT_DEPTH);
        assert_eq!(Biome::Burrows.title(), "the Burrows");
        assert_eq!(biome_for(9), Biome::Fens);
        assert_eq!(biome_for(14), Biome::Crypt);
        assert_eq!(boss_for(8), Some("goblin_warlord"));
        assert_eq!(boss_for(13), Some("bloat_mother"));
        assert_eq!(boss_for(18), Some("lich"));
        assert_eq!(boss_for(5), None);
        assert_eq!(lieutenant_for(5), Some("goblin_captain"));
        assert_eq!(lieutenant_for(8), None);
        assert_eq!(biome_for(19), Biome::Foundry);
        assert_eq!(biome_for(24), Biome::Deep);
        assert_eq!(biome_for(33), Biome::Sanctum);
        assert_eq!(boss_for(23), Some("foundry_master"));
        assert_eq!(boss_for(28), Some("lurker_queen"));
        assert_eq!(boss_for(33), Some("mirror_king"));
        assert_eq!(boss_depth("lich"), Some(18));
        assert_eq!(ENDING_DEPTH, 34);
        for b in Biome::ALL {
            assert_eq!(biome_for(biome_first(b)), b);
            assert!(biome_first(b) == 1 || biome_for(biome_first(b) - 1) != b);
        }
        for (k, d) in BOSS_DEPTHS {
            assert_eq!(boss_for(d), Some(k));
            assert!(d + 1 == ENDING_DEPTH || biome_for(d + 1) != biome_for(d), "{k} guards its biome's last floor");
        }
        assert_eq!((tier_depth(5), tier_depth(6), tier_depth(8), tier_depth(9), tier_depth(33)), (5, 5, 5, 6, 30));
        assert_eq!(Biome::Deep.vision(), 4);
        assert_eq!(Biome::Sanctum.vision(), 7);
    }
    /// Cut 26 §1: 13 routes of non-overlapping adjacent swaps; the base is the old descent; every
    /// biome within one band of its own; each band's boss on its last floor; a far stair closes
    /// the next fork.
    #[test]
    fn routes_enumerate() {
        let all = Route::all();
        assert_eq!(all.len(), 13);
        assert_eq!(all[0], Route::BASE);
        for d in 1..=ENDING_DEPTH {
            assert_eq!(Route::BASE.biome(d), biome_for(d), "D{d}");
            assert_eq!(Route::BASE.boss(d), boss_for(d), "D{d}");
            assert_eq!(Route::BASE.lieutenant(d), lieutenant_for(d), "D{d}");
        }
        for r in &all {
            let o = r.order();
            for (i, b) in o.iter().enumerate() {
                let home = BASE_ORDER.iter().position(|x| x == b).unwrap();
                assert!(home.abs_diff(i) <= 1, "{r:?}: {b:?} at band {i}");
            }
            let mut seen = o.to_vec();
            seen.sort_by_key(|b| *b as u8);
            seen.dedup();
            assert_eq!(seen.len(), 6);
            for (k, d) in r.bosses() {
                assert_eq!(r.boss(d), Some(k));
                assert_eq!(r.boss_depth(k), Some(d));
                assert!(BANDS.iter().any(|(_, last)| *last == d));
            }
            assert_eq!(Route::from_forks(&r.forks()), Ok(*r));
            for f in r.forks() {
                assert!(r.fork_open(f));
            }
        }
        let fens = Route::from_forks(&[5]).unwrap();
        assert_eq!((fens.biome(5), fens.biome(8), fens.biome(9), fens.biome(14)), (Biome::Fens, Biome::Fens, Biome::Burrows, Biome::Crypt));
        assert_eq!((fens.boss(8), fens.boss(13)), (Some("bloat_mother"), Some("goblin_warlord")));
        assert_eq!((fens.lieutenant(9), fens.lieutenant(5)), (Some("goblin_captain"), None));
        assert!(!fens.fork_open(9) && fens.fork_open(14));
        assert!(Route::from_forks(&[5, 9]).is_err());
        assert!(Route::from_forks(&[6]).is_err());
        assert_eq!(fens.with(9, true), Route::from_forks(&[9]).unwrap());
        assert_eq!(Route::from_forks(&[5, 14]).unwrap().label(), "D5 fens · D14 foundry");
        assert_eq!(Route::from_forks(&[5, 14]).unwrap().prefix(9), fens);
        assert_eq!(fens.prefix(4), Route::BASE);
    }
}
