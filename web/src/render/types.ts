// The renderer's view of the wire types. The client track owns web/src/engine/types.ts
// (transcribed from docs/CUT1.md); this module re-exports the subset the renderer consumes so
// render/** never imports engine internals directly.
export type { Entity, Ev, FloorItem, InvItem, Overlay, Snapshot, Tile, Verb } from "../engine/types";
