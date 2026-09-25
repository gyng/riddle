//! Thin wasm-bindgen bridge over riddle-core. JSON strings in and out. See docs/CUT1.md:
//! one method per `Engine` member (camelCased), plus the addenda A–D methods and a few
//! additions listed in crates/riddle-core/README.md.
use riddle_core::Game as Core;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0".into()
}

fn js<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".into())
}

fn err(e: String) -> JsError {
    JsError::new(&e)
}

fn ids(text: &str) -> Result<Vec<u32>, JsError> {
    serde_json::from_str::<Vec<u32>>(text).map_err(|e| err(e.to_string()))
}

#[wasm_bindgen]
pub struct Game {
    inner: Core,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Game {
        Game { inner: Core::new(seed as u64) }
    }

    /// Start a fresh lineage in place; returns the Lineage.
    #[wasm_bindgen(js_name = newLineage)]
    pub fn new_lineage(&mut self, seed: u32) -> String {
        self.inner = Core::new(seed as u64);
        js(&self.inner.lineage())
    }

    /// Replace the state from a save; returns the Lineage.
    pub fn load(&mut self, save: &str) -> Result<String, JsError> {
        self.inner = Core::load(save).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// A new Game from a save (static).
    #[wasm_bindgen(js_name = fromSave)]
    pub fn from_save(save: &str) -> Result<Game, JsError> {
        Ok(Game { inner: Core::load(save).map_err(err)? })
    }

    pub fn save(&self) -> String {
        self.inner.save()
    }

    pub fn vocabulary(&self) -> String {
        js(&self.inner.vocabulary())
    }

    #[wasm_bindgen(js_name = setRules)]
    pub fn set_rules(&mut self, set: &str) -> Result<(), JsError> {
        let set = riddle_core::RuleSet::parse(set).map_err(err)?;
        self.inner.set_rules(set).map_err(err)
    }

    /// `itemIds`: JSON array of vault item ids to bring.
    pub fn loadout(&mut self, item_ids: &str) -> Result<(), JsError> {
        let v = ids(item_ids)?;
        self.inner.loadout(v);
        Ok(())
    }

    pub fn forecast(&self) -> String {
        js(&self.inner.forecast())
    }

    /// Cut 6 §9: the same forecast at 100 sims (same seeds first), the client's refine pass.
    #[wasm_bindgen(js_name = forecastRefine)]
    pub fn forecast_refine(&self) -> String {
        js(&self.inner.forecast_refine())
    }

    pub fn send(&mut self) -> String {
        js(&self.inner.send())
    }

    pub fn step(&mut self, turns: u32) -> String {
        js(&self.inner.step(turns))
    }

    #[wasm_bindgen(js_name = runOffline)]
    pub fn run_offline(&mut self, elapsed_s: f64) -> String {
        js(&self.inner.run_offline(elapsed_s.max(0.0) as u64))
    }

    /// Offline batch without the worst-death verdict (report carries `worst_death_id`).
    #[wasm_bindgen(js_name = runOfflineQuick)]
    pub fn run_offline_quick(&mut self, elapsed_s: f64) -> String {
        js(&riddle_core::offline::run_offline_quick(&mut self.inner, elapsed_s.max(0.0) as u64))
    }

    pub fn death(&mut self, run_id: u32) -> Result<String, JsError> {
        self.inner.death(run_id).map(|d| js(&d)).ok_or_else(|| err(format!("no death record for run {run_id}")))
    }

    /// QA on 23ed91f: the death's shown patches with the camp's own reach deltas (seconds:
    /// four camp panels); `death` answers first with them `camp_pending`.
    #[wasm_bindgen(js_name = deathDeltas)]
    pub fn death_deltas(&mut self, run_id: u32) -> Result<String, JsError> {
        self.inner.death_deltas(run_id).map(|d| js(&d)).ok_or_else(|| err(format!("no death record for run {run_id}")))
    }

    pub fn buy(&mut self, unlock: &str) -> Result<String, JsError> {
        self.inner.buy(unlock).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 15 §2: buy with gold at `UnlockInfo.gold` (marks untouched); returns the lineage.
    #[wasm_bindgen(js_name = buyUnlockGold)]
    pub fn buy_unlock_gold(&mut self, unlock: &str) -> Result<String, JsError> {
        self.inner.buy_unlock_gold(unlock).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    pub fn lineage(&self) -> String {
        js(&self.inner.lineage())
    }

    #[wasm_bindgen(js_name = exportRules)]
    pub fn export_rules(&self) -> String {
        self.inner.export_rules()
    }

    #[wasm_bindgen(js_name = importRules)]
    pub fn import_rules(&self, text: &str) -> Result<String, JsError> {
        self.inner.import_rules(text).map(|r| js(&r)).map_err(err)
    }

    // ---- additions (see README): unlock catalogue, class, saved sets

    pub fn unlocks(&self) -> String {
        js(&self.inner.unlocks())
    }

    /// Cut 4 §9: the catalogue with `delta` simulated for every open card and verb (memoised
    /// per lineage state; `unlocks()` carries the same deltas afterwards without sims).
    #[wasm_bindgen(js_name = unlockDeltas)]
    pub fn unlock_deltas(&self) -> String {
        js(&self.inner.unlock_deltas())
    }

    #[wasm_bindgen(js_name = setClass)]
    pub fn set_class(&mut self, class: &str) -> Result<String, JsError> {
        self.inner.set_class(class).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    #[wasm_bindgen(js_name = selectSet)]
    pub fn select_set(&mut self, index: u32) -> String {
        self.inner.select_set(index as usize);
        js(&self.inner.lineage())
    }

    /// Cut 13 §2: pick the new heir's trait from `Lineage.trait_offer` (before the first
    /// send; a send without a pick keeps the first); returns the Lineage.
    #[wasm_bindgen(js_name = setTrait)]
    pub fn set_trait(&mut self, name: &str) -> Result<String, JsError> {
        self.inner.set_trait(name).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    // ---- Addendum A: companions

    #[wasm_bindgen(js_name = setParty)]
    pub fn set_party(&mut self, ids_json: &str) -> Result<String, JsError> {
        let v = ids(ids_json)?;
        self.inner.set_party(v).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    #[wasm_bindgen(js_name = setCompanionRules)]
    pub fn set_companion_rules(&mut self, id: u32, set: &str) -> Result<(), JsError> {
        let set = riddle_core::RuleSet::parse(set).map_err(err)?;
        self.inner.set_companion_rules(id, set).map_err(err)
    }

    pub fn breed(&mut self, a: u32, b: u32) -> Result<String, JsError> {
        self.inner.breed(a, b).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    pub fn hatch(&mut self, egg_id: u32) -> Result<String, JsError> {
        self.inner.hatch(egg_id).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    #[wasm_bindgen(js_name = companionVocabulary)]
    pub fn companion_vocabulary(&self, id: u32) -> Result<String, JsError> {
        self.inner.companion_vocabulary(id).map(|v| js(&v)).map_err(err)
    }

    // ---- Addendum B: gold and supplies

    #[wasm_bindgen(js_name = buySupply)]
    pub fn buy_supply(&mut self, kind: &str) -> Result<String, JsError> {
        self.inner.buy_supply(kind).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    #[wasm_bindgen(js_name = dropSupply)]
    pub fn drop_supply(&mut self, id: u32) -> Result<String, JsError> {
        self.inner.drop_supply(id).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    #[wasm_bindgen(js_name = clearSupplies)]
    pub fn clear_supplies(&mut self) -> String {
        self.inner.clear_supplies();
        js(&self.inner.lineage())
    }

    #[wasm_bindgen(js_name = supplyCatalogue)]
    pub fn supply_catalogue(&self) -> String {
        js(&self.inner.supply_catalogue())
    }

    // ---- Addendum D: vault keep at exit

    /// `ids`: JSON array of item ids from `StepResult.exit_pending.items` to vault.
    /// QA on 23ed91f: resolve the pending exit by the keep preference and owned automations
    /// (`ExitPending.auto_keep`, replacing a weaker vault item of the same category when the
    /// vault is full) — the skipped keep sheet's call (`keep([])` keeps nothing).
    #[wasm_bindgen(js_name = autoKeep)]
    pub fn auto_keep(&mut self) -> String {
        self.inner.auto_keep();
        js(&self.inner.lineage())
    }

    pub fn keep(&mut self, ids_json: &str) -> Result<String, JsError> {
        let v = ids(ids_json)?;
        self.inner.keep(v).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Insure a vault item against loss on death (25% of salvage value ×10).
    pub fn insure(&mut self, id: u32) -> Result<String, JsError> {
        self.inner.insure(id).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    #[wasm_bindgen(js_name = setKeepPref)]
    pub fn set_keep_pref(&mut self, pref: &str) -> Result<String, JsError> {
        self.inner.set_keep_pref(pref).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    // ---- Cut 5: bail, the vault choice, the vault preference

    /// §5: queue a `return` for the hero's next action (the rules untouched).
    pub fn bail(&mut self) {
        self.inner.bail();
    }

    /// §4: take one item of the opened vault (`Snapshot.vault_choice.items[].id`); returns
    /// the Snapshot.
    pub fn choose(&mut self, item_id: u32) -> Result<String, JsError> {
        self.inner.choose(item_id).map_err(err)?;
        Ok(js(&self.inner.snapshot()))
    }

    /// §4: what an unwatched vault choice takes (`weapon | armour | potion | scroll`).
    #[wasm_bindgen(js_name = setVaultPref)]
    pub fn set_vault_pref(&mut self, pref: &str) -> Result<String, JsError> {
        self.inner.set_vault_pref(pref).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    // ---- Cut 19

    /// §1: each cage preference measured for the active set — `CageOption[]` (`pref`,
    /// `current`, `depth`, `reach`/`reach_delta`, `bank`/`bank_delta`, `gold`/`gold_delta`,
    /// `delta` — the picker's headline — and `pm`). Four camp panels (three new): call it when
    /// the cage tablet's picker opens, or after the refine; memoised like the forecast.
    #[wasm_bindgen(js_name = cageForecast)]
    pub fn cage_forecast(&self) -> String {
        js(&self.inner.cage_forecast())
    }

    /// §3: the loadout's repeat on or off (off refunds the re-packed shelf); returns the Lineage.
    #[wasm_bindgen(js_name = setRestock)]
    pub fn set_restock(&mut self, on: bool) -> String {
        self.inner.set_restock(on);
        js(&self.inner.lineage())
    }

    // ---- Cut 21

    /// §1: the floor the next sends start on — 1 or a lit waystone (`Lineage.waystones`);
    /// refused when not lit; returns the Lineage (`start`, `start_toll`).
    #[wasm_bindgen(js_name = setStart)]
    pub fn set_start(&mut self, depth: u32) -> Result<String, JsError> {
        self.inner.set_start(depth).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// §1: each start the tablet offers (D1 and every lit waystone) measured for the active
    /// set — `StartOption[]` (`depth`, `current`, `toll`, `short`, `bar`, `reach`/`reach_delta`,
    /// `bank`/`bank_delta`, `gold`, `net`/`net_delta`, `pm`). One camp panel per option (seconds
    /// in wasm): call it when the tablet opens, or after the refine; memoised like the forecast.
    #[wasm_bindgen(js_name = startForecast)]
    pub fn start_forecast(&self) -> String {
        js(&self.inner.start_forecast())
    }

    /// Cut 22 §3: the edit's paired move — the active set's camp panel minus `prev`'s (JSON
    /// RuleSet: the set as it was at the last painted forecast) on the same seeds —
    /// `ForecastVs` (`depths[{depth, delta, pm, abs_pm}]`, `bank`/`death`/`return`/`gold` as
    /// `{delta, pm}`, `sims`). The active panel is the forecast's own (cached); the previous
    /// set's is usually cached from its paint. Call it after the forecast's first paint.
    #[wasm_bindgen(js_name = forecastVs)]
    pub fn forecast_vs(&self, prev: &str) -> Result<String, JsError> {
        let prev = riddle_core::RuleSet::parse(prev).map_err(err)?;
        Ok(js(&self.inner.forecast_vs(&prev)))
    }

    // ---- Cut 3: ascension

    /// After the ending: a new lineage under `variant` (`no_rest | short_list | bones_only |
    /// hunted`); returns the Lineage.
    pub fn ascend(&mut self, variant: &str) -> Result<String, JsError> {
        self.inner.ascend(variant).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }
}
