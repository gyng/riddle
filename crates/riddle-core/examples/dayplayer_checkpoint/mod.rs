//! Internal memento: player saves intentionally omit runtime state, which a
//! harness checkpoint must retain (including a live run's tallies and fire).
//! Replay capsules are disabled by this harness; map visibility cache rebuilds.
use riddle_core::{Game, engine::Run};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_json::Value;
macro_rules! fields {
    ($o:expr; $($f:ident),+ $(,)?) => { serde_json::json!({$(stringify!($f): &$o.$f),+}) };
}
macro_rules! restore {
    ($o:expr, $v:expr; $($f:ident),+ $(,)?) => { $(
        $o.$f = serde_json::from_value($v[stringify!($f)].take()).map_err(|e| e.to_string())?;
    )+ };
}
fn run_extra(r: &Run) -> Value {
    let mut v = fields!(r; hero_dist, hero_dist_pos, hero_flood, hero_flood_head,
        last_visible, loop_causes, blocked_now, dens, raiding, acting_row, sleepers, next_twist);
    v["spread"] = serde_json::json!(r.overlays.iter().map(|o| o.spread).collect::<Vec<_>>()); v
}
fn restore_run(r: &mut Run, mut v: Value) -> Result<(), String> {
    restore!(r, v; hero_dist, hero_dist_pos, hero_flood, hero_flood_head,
        last_visible, loop_causes, blocked_now, dens, raiding, acting_row, sleepers, next_twist);
    let spreads: Vec<bool> = serde_json::from_value(v["spread"].take()).map_err(|e| e.to_string())?;
    if spreads.len() != r.overlays.len() { return Err("checkpoint overlay count mismatch".into()); }
    for (o, spread) in r.overlays.iter_mut().zip(spreads) { o.spread = spread; }
    Ok(())
}
pub fn serialize<S: Serializer>(g: &Game, s: S) -> Result<S::Ok, S::Error> {
    if !g.capsules.0.is_empty() { return Err(serde::ser::Error::custom("harness capsules must be disabled")); }
    let mut v = fields!(g; stall_cache, forecast_cache, panel_cache, bounty_seen, row_tally,
        passage, fold_plan, tap, clock_at, advance_rem);
    v["game"] = serde_json::to_value(g).map_err(serde::ser::Error::custom)?;
    v["batch_loop_causes"] = serde_json::json!(g.batch.loop_causes);
    v["run"] = g.run.as_ref().map(run_extra).unwrap_or(Value::Null);
    v["history"] = Value::Array(g.history.iter().map(|(r,_)|run_extra(r)).collect());
    v["floor_start"] = g.floor_start.as_ref().map(|(r,_)|run_extra(r)).unwrap_or(Value::Null);
    v["deaths"] = Value::Object(g.deaths.iter().map(|(id,d)| (id.to_string(), serde_json::json!({
        "floor": d.floor, "floor_extra": d.floor.as_ref().map(|(r,_)|run_extra(r)),
        "t10": d.t10.as_ref().map(run_extra),
    }))).collect());
    v.serialize(s)
}
fn decode(mut v: Value) -> Result<Game, String> {
    let mut g: Game = serde_json::from_value(v["game"].take()).map_err(|e| e.to_string())?;
    restore!(g, v; stall_cache, forecast_cache, panel_cache, bounty_seen, row_tally,
        passage, fold_plan, tap, clock_at, advance_rem);
    g.batch.loop_causes = serde_json::from_value(v["batch_loop_causes"].take()).map_err(|e| e.to_string())?;
    if let Some(r) = g.run.as_mut() { restore_run(r, v["run"].take())?; }
    let history = v["history"].as_array_mut().ok_or("checkpoint history missing")?;
    if history.len() != g.history.len() { return Err("checkpoint history mismatch".into()); }
    for ((r,_), extra) in g.history.iter_mut().zip(history) { restore_run(r, extra.take())?; }
    if let Some((r,_)) = g.floor_start.as_mut() { restore_run(r, v["floor_start"].take())?; }
    for (id, d) in &mut g.deaths {
        let extra = &mut v["deaths"][id.to_string()];
        d.floor = serde_json::from_value(extra["floor"].take()).map_err(|e| e.to_string())?;
        if let Some((r,_)) = &mut d.floor { restore_run(r, extra["floor_extra"].take())?; }
        if let Some(r) = &mut d.t10 { restore_run(r, extra["t10"].take())?; }
    }
    Ok(g)
}
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Game, D::Error> {
    decode(Value::deserialize(d)?).map_err(serde::de::Error::custom)
}
