//! Rust-owned numerical tuning. Shipping uses DEFAULT; dev profiles are immutable
//! for the lifetime of a native process, so forecasts never mix two profiles.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Balance {
    pub level_runs: [u32; 4],
    pub scar_pct: u32,
    pub scar_cap: u32,
    pub drill_meeting: u32,
    pub drill_days: u32,
    pub deep_drill_days: u32,
    pub steady_heal: [i32; 2],
    pub hunter_heal: [i32; 2],
    pub guarded_heal: [i32; 2],
    pub bold_heal: i32,
    pub steady_home: i32,
    pub steady_dry: [i32; 2],
    pub steady_rest: [i32; 2],
    pub guarded_home: [i32; 2],
    pub guarded_dry: [i32; 2],
}
pub const DEFAULT: Balance = Balance {
    level_runs: [10, 40, 120, 220], scar_pct: 5, scar_cap: 30,
    drill_meeting: 2, drill_days: 3, deep_drill_days: 6,
    steady_heal: [30,35], hunter_heal: [30,35], guarded_heal: [40,45], bold_heal: 25,
    steady_home: 25, steady_dry: [40,35], steady_rest: [40,50],
    guarded_home: [30,25], guarded_dry: [45,44],
};
pub fn parse(text: &str) -> Result<Balance, String> {
    let b: Balance = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if b.level_runs[0] == 0 || b.level_runs.windows(2).any(|w| w[0] >= w[1]) || b.level_runs[3] > 1_000_000
        || b.scar_pct == 0 || b.scar_pct > b.scar_cap || b.scar_cap > 100
        || [b.drill_meeting,b.drill_days,b.deep_drill_days].iter().any(|n| *n == 0 || *n > 365)
        || b.deep_drill_days < b.drill_days
        || b.steady_heal.iter().chain(&b.hunter_heal).chain(&b.guarded_heal).chain(&b.steady_dry)
            .chain(&b.steady_rest).chain(&b.guarded_home).chain(&b.guarded_dry)
            .chain([b.bold_heal,b.steady_home].iter()).any(|n| !(1..=100).contains(n)) {
        return Err("invalid balance bounds or progression ordering".into());
    }
    Ok(b)
}
#[cfg(feature = "dev-balance")]
static PROFILE: std::sync::OnceLock<Balance> = std::sync::OnceLock::new();
#[inline]
pub fn get() -> &'static Balance {
    #[cfg(feature = "dev-balance")]
    if let Some(b) = PROFILE.get() { return b; }
    &DEFAULT
}
/// Call before constructing any game; a second installation is an error.
pub fn configure_from_env() -> Result<(), String> {
    if let Ok(text) = std::env::var("RIDDLE_BALANCE_JSON") {
        #[cfg(feature = "dev-balance")]
        { return PROFILE.set(parse(&text)?).map_err(|_| "balance already installed".into()); }
        #[cfg(not(feature = "dev-balance"))]
        { let _ = text; return Err("balance overrides require --features dev-balance".into()); }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shipping_template_and_bounds() {
        let text = include_str!("../../../tuning/balance.json");
        assert_eq!(parse(text).unwrap(), DEFAULT);
        assert!(parse(&text.replace("\"scar_pct\": 5", "\"scar_pct\": 0")).is_err());
        assert!(parse(&text.replace("10, 40, 120, 220", "10, 10, 120, 220")).is_err());
        assert!(parse(&text.replace("\"scar_pct\"", "\"typo\"")).is_err());
    }
}
