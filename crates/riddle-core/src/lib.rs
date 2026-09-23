//! Riddle core: deterministic roguelike sim, rule engine, facts, forecast, offline batch,
//! chronicle, sifter, meta. All game truth lives here. See docs/CUT1.md.
pub mod ai;
pub mod chronicle;
pub mod defs;
pub mod descent;
pub mod engine;
pub mod facts;
pub mod forecast;
pub mod gen;
pub mod geom;
pub mod hero;
pub mod item;
pub mod meta;
pub mod monster;
pub mod offline;
pub mod probes;
pub mod provenance;
pub mod rng;
pub mod rules;
pub mod save;
pub mod sifter;
pub mod situations;
pub mod tiles;
pub mod tokens;
pub mod trace;
pub mod turn;
pub mod wire;

pub use engine::Game;
pub use rules::{Cond, Row, RuleSet, Verb, Vocabulary};
pub use wire::*;

impl Game {
    pub fn forecast(&self) -> Forecast {
        forecast::forecast(self)
    }
    /// Cut 6 §9: the forecast at 100 sims (the same seeds first), for the client's second pass.
    pub fn forecast_refine(&self) -> Forecast {
        forecast::forecast_refine(self)
    }
    pub fn run_offline(&mut self, elapsed_s: u64) -> ReturnReport {
        offline::run_offline(self, elapsed_s)
    }
    pub fn death(&mut self, run_id: u32) -> Option<Death> {
        trace::death(self, run_id)
    }
    pub fn buy(&mut self, unlock: &str) -> Result<(), String> {
        meta::buy(self, unlock)
    }
    /// Cut 15 §2: buy an unlock with gold at its climbing price (`UnlockInfo.gold`); marks untouched.
    pub fn buy_unlock_gold(&mut self, unlock: &str) -> Result<(), String> {
        meta::buy_gold(self, unlock)
    }
    /// The unlock catalogue; Cut 4 §9: cards and verbs carry `delta` once `unlock_deltas`
    /// computed them for this lineage state (no sims here).
    pub fn unlocks(&self) -> Vec<UnlockInfo> {
        meta::catalogue_with_deltas(self, false)
    }
    /// The catalogue with every open card's and verb's `delta` simulated (memoised: a camp
    /// visit pays once; `unlocks()` returns the same numbers afterwards).
    pub fn unlock_deltas(&self) -> Vec<UnlockInfo> {
        meta::catalogue_with_deltas(self, true)
    }
    pub fn save(&self) -> String {
        save::save(self)
    }
    pub fn load(text: &str) -> Result<Game, String> {
        save::load(text)
    }
    pub fn export_rules(&self) -> String {
        self.lineage.rules().to_text()
    }
    pub fn import_rules(&self, text: &str) -> Result<RuleSet, String> {
        RuleSet::parse(text)
    }
}

#[cfg(test)]
mod tests;
