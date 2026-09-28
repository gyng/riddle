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
        js(&self.inner.vocabulary_wire())
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

    /// One slice of a chunked absence (`runOfflineQuick`); a slice before the `last` carries no
    /// stall verdict (the client reads the last slice's: its patch forecasts were most of an
    /// absence's time on a plateaued lineage — 3 of the D11 fixture's 16 slices paid ~2 s each).
    #[wasm_bindgen(js_name = runOfflineSlice)]
    pub fn run_offline_slice(&mut self, elapsed_s: f64, last: bool) -> String {
        let secs = elapsed_s.max(0.0) as u64;
        js(&if last { riddle_core::offline::run_offline_quick(&mut self.inner, secs) } else { riddle_core::offline::run_offline_counts(&mut self.inner, secs) })
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

    /// QA on 308f045: take an item out of the vault — salvaged at a bank's share (the lineage).
    #[wasm_bindgen(js_name = sellVault)]
    pub fn sell_vault(&mut self, id: u32) -> Result<String, JsError> {
        self.inner.sell_vault(id).map_err(err)?;
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

    /// Hero looks: the heirs' cosmetic look (`male | female | cat`); returns the Lineage.
    #[wasm_bindgen(js_name = setLook)]
    pub fn set_look(&mut self, look: &str) -> Result<String, JsError> {
        self.inner.set_look(look).map_err(err)?;
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
    /// QA on 524827b: `refined` — measure on the camp's refined pass (the sheet's current option
    /// is then the camp's number); absent, the lineage's own read (`forecast::camp_sims`).
    #[wasm_bindgen(js_name = cageForecast)]
    pub fn cage_forecast(&self, refined: Option<bool>) -> String {
        match refined {
            Some(r) => js(&self.inner.cage_forecast_refined(r)),
            None => js(&self.inner.cage_forecast()),
        }
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
    pub fn start_forecast(&self, refined: Option<bool>) -> String {
        // QA on 308f045: `refined` — the camp's pass (the cage tablet's rule); absent, the camp's own
        match refined {
            Some(r) => js(&self.inner.start_forecast_refined(r)),
            None => js(&self.inner.start_forecast()),
        }
    }

    /// Cut 26 §2: the fork chip's option tablet — both stairs of the fork at `fork` (5, 9, 14, 19,
    /// 24) for the active set: `ForkOption[]` (`biome`, `far`, `current`, `route` — the set's
    /// route with that stair —, `depth` the band's last floor, `reach`/`reach_delta`, `bank`,
    /// `gold`, `death`, `delta`, `pm`). One camp panel (the other stair's); memoised. A route
    /// edit itself is `setRules` with the set's `route` (fork depths of its far stairs).
    #[wasm_bindgen(js_name = forkForecast)]
    pub fn fork_forecast(&self, fork: u32, refined: Option<bool>) -> String {
        // QA on 308f045: `refined` — the camp's pass (the cage tablet's rule); absent, the camp's own
        match refined {
            Some(r) => js(&self.inner.fork_forecast_refined(fork, r)),
            None => js(&self.inner.fork_forecast(fork)),
        }
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

    /// Cut 27 §1: right after `send` — play the floors the set clears ≥ 95 % and return the fold
    /// line (`FoldLine`: `from`, `to`, `clear`, `gold`, `beats`, `chips`, `floors[]` with each
    /// floor's snapshot and events, `step` as one `step()`). Nothing folded: `to < from`.
    pub fn fold(&mut self) -> String {
        js(&self.inner.fold())
    }

    /// Cut 27 §2: the edit as a scene against `prev` (JSON RuleSet: the sent set) — `Divergence`
    /// or `null` when the sets play alike on every seed tried. Reads the camp's paired panels
    /// (cached by the forecast and its `vs`); call it after the forecast's `vs`.
    pub fn divergence(&self, prev: &str) -> Result<String, JsError> {
        let prev = riddle_core::RuleSet::parse(prev).map_err(err)?;
        Ok(js(&self.inner.divergence(&prev)))
    }

    /// Cut 28 §2: the camp's move against `prev` (JSON RuleSet: the set sent), attributed to the
    /// state since the send and the rows (`ForecastMove`: `whole`, `parts[{kind, text, move}]`,
    /// `lead`, `rows`, `state`), or `null` when no send was recorded. 2–6 camp panels: call it on
    /// a background lane after the forecast.
    #[wasm_bindgen(js_name = forecastMove)]
    pub fn forecast_move(&self, prev: &str) -> Result<String, JsError> {
        let prev = riddle_core::RuleSet::parse(prev).map_err(err)?;
        Ok(js(&self.inner.forecast_move(&prev)))
    }

    /// Cut 28 §1: swear the standing oath `id` (pays its price); returns the Lineage.
    #[wasm_bindgen(js_name = swearOath)]
    pub fn swear_oath(&mut self, id: &str) -> Result<String, JsError> {
        self.inner.swear_oath(id).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 28 §1: forswear the sworn oath (half its price back); returns the Lineage.
    #[wasm_bindgen(js_name = forswearOath)]
    pub fn forswear_oath(&mut self) -> Result<String, JsError> {
        self.inner.forswear_oath().map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 29 §1: forswear the sworn oath `id` (either slot); returns the Lineage.
    #[wasm_bindgen(js_name = forswearOathId)]
    pub fn forswear_oath_id(&mut self, id: &str) -> Result<String, JsError> {
        self.inner.forswear_oath_id(id).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 29 §1: an oath draw (◆2): a fresh standing oath; returns the Lineage.
    #[wasm_bindgen(js_name = drawOath)]
    pub fn draw_oath(&mut self) -> Result<String, JsError> {
        self.inner.draw_oath().map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 29 §5: commission the next work with gold; returns the Lineage.
    pub fn commission(&mut self) -> Result<String, JsError> {
        self.inner.commission().map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 29 §1 (E1): at a wall, the one-row edit that passes it (`WallEdit` JSON, or `null`) —
    /// searched once a day (seconds to minutes in wasm: call it on the report, off the foreground),
    /// the cached offer after (`Lineage.wall`).
    #[wasm_bindgen(js_name = wallEdit)]
    pub fn wall_edit(&mut self) -> String {
        js(&self.inner.wall_edit())
    }

    /// Cut 29 §2: the camp showed the newly opened systems (clears `Lineage.systems[].new`); returns the Lineage.
    #[wasm_bindgen(js_name = seenSystems)]
    pub fn seen_systems(&mut self) -> String {
        self.inner.seen_systems();
        js(&self.inner.lineage())
    }

    /// Cut 29 §4: set the standing orders (a `StandingOrders` JSON); returns the Lineage.
    #[wasm_bindgen(js_name = setOrders)]
    pub fn set_orders(&mut self, orders_json: &str) -> Result<String, JsError> {
        let o: riddle_core::wire::StandingOrders = serde_json::from_str(orders_json).map_err(|e| JsError::new(&e.to_string()))?;
        self.inner.set_orders(&o).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 23 §1: buy the next forge step of `weapon | armour | pack`; returns the Lineage.
    #[wasm_bindgen(js_name = buyKit)]
    pub fn buy_kit(&mut self, slot: &str) -> Result<String, JsError> {
        riddle_core::kit::buy(&mut self.inner, slot).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }

    /// Cut 23 §1: the forge's ladders with each next step's paired forecast move.
    #[wasm_bindgen(js_name = kitDeltas)]
    pub fn kit_deltas(&self) -> String {
        js(&riddle_core::kit::deltas(&self.inner))
    }

    // ---- Cut 3: ascension

    /// After the ending: a new lineage under `variant` (`no_rest | short_list | bones_only |
    /// hunted`); returns the Lineage.
    pub fn ascend(&mut self, variant: &str) -> Result<String, JsError> {
        self.inner.ascend(variant).map_err(err)?;
        Ok(js(&self.inner.lineage()))
    }
}
