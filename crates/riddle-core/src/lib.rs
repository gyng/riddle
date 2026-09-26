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
pub mod kit;
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
    /// Cut 19 §1: each cage preference's forecast for the active set (`forecast::cage_forecast`).
    pub fn cage_forecast(&self) -> Vec<CageOption> {
        forecast::cage_forecast(self)
    }
    /// QA on 524827b: the options on the camp's own pass (`refined`: the 100-sim panels).
    pub fn cage_forecast_refined(&self, refined: bool) -> Vec<CageOption> {
        forecast::cage_forecast_at(self, if refined { forecast::REFINE_SIMS } else { forecast::FORECAST_SIMS })
    }
    /// Cut 21 §1: each start's forecast for the active set (`forecast::start_forecast`).
    pub fn start_forecast(&self) -> Vec<StartOption> {
        forecast::start_forecast(self)
    }
    /// QA on 308f045: the starts on the camp's own pass (`refined`: the 100-sim panels).
    pub fn start_forecast_refined(&self, refined: bool) -> Vec<StartOption> {
        forecast::start_forecast_at(self, if refined { forecast::REFINE_SIMS } else { forecast::FORECAST_SIMS })
    }
    /// Cut 26 §2: both stairs of the fork at `fork` for the active set (`forecast::fork_forecast`).
    pub fn fork_forecast(&self, fork: u32) -> Vec<ForkOption> {
        forecast::fork_forecast(self, fork)
    }
    /// QA on 308f045: the stairs on the camp's own pass (`refined`: the 100-sim panels).
    pub fn fork_forecast_refined(&self, fork: u32, refined: bool) -> Vec<ForkOption> {
        forecast::fork_forecast_at(self, fork, if refined { forecast::REFINE_SIMS } else { forecast::FORECAST_SIMS })
    }
    /// Cut 22 §3: the active set's paired move against `prev` (`forecast::forecast_vs`).
    pub fn forecast_vs(&self, prev: &RuleSet) -> ForecastVs {
        forecast::forecast_vs(self, prev)
    }
    pub fn run_offline(&mut self, elapsed_s: u64) -> ReturnReport {
        offline::run_offline(self, elapsed_s)
    }
    pub fn death(&mut self, run_id: u32) -> Option<Death> {
        trace::death(self, run_id)
    }
    /// QA on 23ed91f: the death's shown patches with the camp's reach deltas (`trace::death_deltas`).
    pub fn death_deltas(&mut self, run_id: u32) -> Option<Vec<Patch>> {
        trace::death_deltas(self, run_id)
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
