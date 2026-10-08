//! Cut 109: print generated floors as text, one per line block — `cargo run -q --profile fast -p riddle-core --example floors -- <biome> <depth> <seeds>`
//! (`… json` prints the renderer's wire floor per line for `render-demo.html?plan=`; `#` wall, `.` floor, `+` door, `~` water, `^` chasm, `<` `>` stairs). A look at layout variety, not a gate.
use riddle_core::descent::Biome;
use riddle_core::gen::generate;
use riddle_core::rng::Rng;
use riddle_core::tiles::Tile;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let biome = match a.get(1).map(String::as_str).unwrap_or("warrens") {
        "burrows" => Biome::Burrows,
        "fens" => Biome::Fens,
        "crypt" => Biome::Crypt,
        "foundry" => Biome::Foundry,
        "deep" => Biome::Deep,
        "sanctum" => Biome::Sanctum,
        _ => Biome::Warrens,
    };
    let depth: u32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
    let seeds: u64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(4);
    let json = a.get(4).is_some_and(|s| s == "json");
    for seed in 0..seeds {
        let f = generate(&mut Rng::new(seed * 7919 + 11), biome, depth);
        if json {
            // the renderer's wire tiles (`render-demo.html?plan=`)
            let tiles: Vec<String> = f.map.tiles.iter().map(|t| serde_json::to_value(t).unwrap().as_str().unwrap_or("floor").to_string()).collect();
            let name = a.get(1).map(String::as_str).unwrap_or("warrens");
            println!("{}", serde_json::json!({ "w": f.map.w, "h": f.map.h, "tiles": tiles, "biome": name, "depth": depth, "up": [f.stairs_up.x, f.stairs_up.y] }));
            continue;
        }
        println!("seed {seed}");
        for y in 0..f.map.h {
            let row: String = (0..f.map.w)
                .map(|x| match f.map.tiles[f.map.idx(riddle_core::geom::Pos::new(x, y))] {
                    Tile::Wall => '#',
                    Tile::Door => '+',
                    Tile::Water => '~',
                    Tile::Chasm => '^',
                    Tile::StairsUp => '<',
                    Tile::StairsDown => '>',
                    _ => '.',
                })
                .collect();
            println!("{row}");
        }
    }
}
