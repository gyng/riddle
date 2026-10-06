//! Controlled combat/wire fixtures. No claim of earned Gunner progression.
use riddle_core::{Game,bloodlines::Session,geom::Pos,item::Item,monster::Monster,
    gen::Floor,tiles::{Map,Tile},rules::{RuleSet,Row,Cond,Verb},wire::Ev};
use std::path::Path;
fn fixture(kind:&str)->Session {
    let mut s=Session::new(13);s.active=Game::new_literal(13);
    s.active.start_run(Some(77));
    s.active.set_rules_raw(RuleSet {name:None,route:vec![],rows:vec![
        Row::new(vec![],Verb::new("reload")),
        Row::new(vec![Cond::n("foes>=",1)],Verb::new("fire")),
        Row::new(vec![],Verb::new("hold")),
    ]}).unwrap();
    let r=s.active.run.as_mut().unwrap();
    let mut map=Map::new(16,12,Tile::Wall);
    for y in 1..11 {for x in 1..15 {map.set(Pos::new(x,y),Tile::Floor);}}
    let up=Pos::new(1,1);let down=Pos::new(14,10);
    map.set(up,Tile::StairsUp);map.set(down,Tile::StairsDown);map.compute_corridors(&[]);
    r.floor=Floor{map,stairs_up:up,stairs_down:down,rooms:vec![],vision:8};
    r.hero.pos=Pos::new(4,5);r.hero.weapon=Some(Item::new(900,kind)).into();
    r.hero.inv.clear();r.hero.armour=None.into();r.hero.gift=Default::default();r.hero.str_bonus=0;
    r.monsters.clear();r.items.clear();r.overlays.clear();r.hero_dist_pos=None;
    let mut foe=Monster::spawn(901,"goblin",Pos::new(6,5),1);
    foe.hp=10000;foe.max_hp=10000;foe.awake=true;foe.stun=10000;r.monsters.push(foe);
    r.floor.map.update_vision(r.hero.pos,8);
    s.active.events.clear();s.active.history.clear();
    Session::load(&s.save()).unwrap()
}
fn main() {
    riddle_core::chronicle::use_shipping_words();
    let arg=std::env::args().nth(1).expect("new output directory");let dir=Path::new(&arg);
    std::fs::create_dir(dir).expect("new directory");
    let mut rows=vec![];
    for kind in ["long_gun","short_gun"] {
        let mut s=fixture(kind);let first=s.step(21);
        assert!(first.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="fire")));
        assert!(s.active.run.as_ref().unwrap().gun_reload.is_some());
        let before=s.save();std::fs::write(dir.join(format!("{kind}-before.json")),&before).unwrap();
        let step=s.step(61);let after=s.save();
        let shots=step.events.iter().filter(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="fire")).count();
        let completions=step.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="loaded")).count();
        assert!(shots>0&&completions>0);
        let mut sliced=Session::load(&before).unwrap();
        for (i,ticks) in [3,7,1,10,5,17,18].into_iter().enumerate() {
            let step=sliced.step(ticks);
            std::fs::write(dir.join(format!("{kind}-slice{i}-step.json")),serde_json::to_string(&step).unwrap()).unwrap();
            std::fs::write(dir.join(format!("{kind}-slice{i}-after.json")),sliced.save()).unwrap();
            sliced=Session::load(&sliced.save()).unwrap();
        }
        assert_eq!(sliced.save(),after,"whole/sliced/reloaded diagnostic {kind}");
        std::fs::write(dir.join(format!("{kind}-step.json")),serde_json::to_string(&step).unwrap()).unwrap();
        std::fs::write(dir.join(format!("{kind}-after.json")),after).unwrap();
        let row=serde_json::json!({"weapon":kind,"controlled_arena":true,"progression_proof":false,"ticks":61,"shots":shots,"reload_completions":completions,"native_sliced_save_exact":true});
        println!("{row}");rows.push(row);
    }
    std::fs::write(dir.join("results.json"),serde_json::to_string_pretty(&rows).unwrap()).unwrap();
}
