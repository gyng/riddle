// The renderer's view of the wire types. The client track owns web/src/engine/types.ts
// (transcribed from docs/CUT1.md incl. addenda, mirrored by crates/riddle-core/src/wire.rs);
// this module re-exports the subset the renderer consumes so render/** never imports engine
// internals directly.
export type { Entity, GunSnap, Ev, FloorItem, InvItem, Overlay, Snapshot, Tile, Verb } from "../engine/types";
