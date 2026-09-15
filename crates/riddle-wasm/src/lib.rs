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

    pub fn death(&mut self, run_id: u32) -> Result<String, JsError> {
        self.inner.death(run_id).map(|d| js(&d)).ok_or_else(|| err(format!("no death record for run {run_id}")))
    }

    pub fn buy(&mut self, unlock: &str) -> Result<String, JsError> {
        self.inner.buy(unlock).map_err(err)?;
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
    pub fn keep(&mut self, ids_json: &str) -> Result<String, JsError> {
        let v = ids(ids_json)?;
        self.inner.keep(v).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    #[wasm_bindgen(js_name = setKeepPref)]
    pub fn set_keep_pref(&mut self, pref: &str) -> Result<String, JsError> {
        self.inner.set_keep_pref(pref).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }
}
