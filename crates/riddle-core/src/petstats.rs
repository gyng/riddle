//! Cut 119's eval framework (`examples/pets_eval.rs`): cheap per-thread counters on the companion system.
//! Write-only instrumentation: the game never reads them, so no outcome moves (the 307dbed send hash holds).
//! A probe reads `snapshot()` before and after work it runs on its own thread (parallel sims off).
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Stat {
    /// Damage landed on hostile foes (capped at the foe's remaining hp): the hero's, a companion's, another ally's.
    HeroDealt,
    PetDealt,
    AllyDealt,
    /// Blows a hostile foe swung (`ai::monster_attack`): at the hero, at a companion, at another ally.
    BlowsHero,
    BlowsPet,
    BlowsAlly,
    /// D1–D4: hero steps toward loot (`ai::nearest_item_step`), their ticks (×1000), and every tick spent there.
    PickupSteps,
    PickupMilliTicks,
    EarlyTicks,
    /// A companion's acts (`ai::companion_act`): every blow or verb, and those of its rows (called out).
    PetActs,
    PetRowActs,
    /// Live games only (never a sim): runs finished, companions that fell (and those whose fall names a cause), loss-eggs
    /// hatched, strays re-tamed, tames.
    Runs,
    PetsFell,
    PetsFellNamed,
    LossHatched,
    StraysRetamed,
    Tamed,
}

pub const N: usize = Stat::Tamed as usize + 1;

thread_local! {
    static COUNTS: [Cell<u64>; N] = const { [const { Cell::new(0) }; N] };
}

#[inline]
pub fn bump(s: Stat, n: u64) {
    COUNTS.with(|c| c[s as usize].set(c[s as usize].get().wrapping_add(n)));
}

/// This thread's counters.
pub fn snapshot() -> [u64; N] {
    COUNTS.with(|c| std::array::from_fn(|i| c[i].get()))
}

/// `after − before`, per counter.
pub fn delta(before: &[u64; N], after: &[u64; N]) -> [u64; N] {
    std::array::from_fn(|i| after[i].wrapping_sub(before[i]))
}
