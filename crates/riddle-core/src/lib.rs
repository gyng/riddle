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
pub mod rng;
pub mod rules;
pub mod save;
pub mod sifter;
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
    pub fn run_offline(&mut self, elapsed_s: u64) -> ReturnReport {
        offline::run_offline(self, elapsed_s)
    }
    pub fn death(&mut self, run_id: u32) -> Option<Death> {
        trace::death(self, run_id)
    }
    pub fn buy(&mut self, unlock: &str) -> Result<(), String> {
        meta::buy(self, unlock)
    }
    pub fn unlocks(&self) -> Vec<UnlockInfo> {
        meta::catalogue(&self.lineage)
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
