//! Riddle core: deterministic roguelike sim, rule engine, facts, forecast, offline batch,
//! chronicle, sifter, meta. All game truth lives here. See docs/CUT1.md.
pub mod ai;
pub mod chronicle;
pub mod defs;
pub mod descent;
pub mod engine;
pub mod facts;
pub mod fold;
pub mod divergence;
pub mod forecast;
pub mod gen;
pub mod geom;
pub mod hero;
pub mod item;
pub mod kit;
pub mod meta;
pub mod monster;
pub mod oath;
pub mod offline;
pub mod probes;
pub mod provenance;
pub mod rng;
pub mod rules;
pub mod save;
pub mod sifter;
pub mod situations;
pub mod systems;
pub mod wall;
pub mod meters;
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
    /// Cut 28 §1: swear a standing oath (`oath::swear`: pays its price; one at a time).
    pub fn swear_oath(&mut self, id: &str) -> Result<(), String> {
        oath::swear(self, id)
    }
    /// Cut 28 §1: forswear the sworn oath (half its price back).
    pub fn forswear_oath(&mut self) -> Result<(), String> {
        oath::forswear(self)
    }
    /// Cut 29 §1: forswear the sworn oath `id` (either slot).
    pub fn forswear_oath_id(&mut self, id: &str) -> Result<(), String> {
        oath::forswear_id(self, id)
    }
    /// Cut 29 §1: an oath draw (◆2, from T2): a fresh standing oath; its id.
    pub fn draw_oath(&mut self) -> Result<String, String> {
        oath::draw(self)
    }
    /// Cut 29 §5: commission the lineage's next work with gold; its label.
    pub fn commission(&mut self) -> Result<String, String> {
        kit::commission(self)
    }
    /// Cut 29 §1 (E1): at a wall (the best depth held `wall::WALL_DAYS` days), the edit that passes
    /// it — searched once a day of the lineage's clock (`wall::search`, seconds natively, far more in
    /// wasm: the client asks it on the report, off the foreground), then the cached offer
    /// (`Lineage.wall`) until a new best clears it. `None` off a wall or when no edit passes.
    pub fn wall_edit(&mut self) -> Option<crate::wire::WallEdit> {
        if !crate::wall::at_wall(&self.lineage) {
            return None;
        }
        if self.lineage.wall_day != Some(self.lineage.day) {
            self.lineage.wall_day = Some(self.lineage.day);
            self.lineage.wall_offer = crate::wall::search(self);
        }
        self.lineage.wall_offer.clone()
    }
    /// Cut 29 §2: the camp has shown the systems opened since it last looked (the glint is spent).
    pub fn seen_systems(&mut self) {
        self.lineage.systems_new.clear();
    }
    /// Cut 29 §4: the standing orders at once (`keep`, `cage`, `start`, `repeat`, `insure`); each
    /// through its own setter, so each keeps its rules (an unknown keep is refused; the repeat
    /// off refunds the shelf).
    pub fn set_orders(&mut self, o: &crate::wire::StandingOrders) -> Result<(), String> {
        self.set_keep_pref(&o.keep)?;
        self.set_vault_pref(&o.cage)?;
        if o.start != self.lineage.start.max(1) {
            self.set_start(o.start)?;
        }
        self.set_restock(o.repeat);
        self.lineage.orders.insure = o.insure;
        Ok(())
    }
    /// Cut 28 §2: the camp's move against the set sent, attributed to state and rows
    /// (`forecast::forecast_move`); `None` when no send was recorded.
    pub fn forecast_move(&self, prev: &RuleSet) -> Option<ForecastMove> {
        forecast::forecast_move(self, prev)
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
#[cfg(test)]
mod tests_cut27;
#[cfg(test)]
mod tests_cut27_seams;
#[cfg(test)]
mod tests_cut28;
#[cfg(test)]
mod tests_cut29;
