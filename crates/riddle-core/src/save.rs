//! Save and load: the whole game as versioned JSON.
use crate::engine::{Game, SAVE_VERSION};

pub fn save(game: &Game) -> String {
    serde_json::to_string(game).unwrap_or_default()
}

pub fn load(text: &str) -> Result<Game, String> {
    let g: Game = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if g.version != SAVE_VERSION {
        return Err(format!("save version {} unsupported", g.version));
    }
    Ok(g)
}
