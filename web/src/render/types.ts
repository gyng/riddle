// The renderer's view of the wire types. The client track owns web/src/engine/types.ts
// (transcribed from docs/CUT1.md incl. addenda); this module re-exports the subset the
// renderer consumes so render/** never imports engine internals directly. Kinds the engine
// file does not carry yet are declared here; the unions simply overlap once it does.
import type { Ev as EngineEv } from "../engine/types";
export type { Entity, FloorItem, InvItem, Overlay, Snapshot, Tile, Verb } from "../engine/types";

// Addendum E: projectiles travel 1 tile per tick along `path` before the attack/use lands.
export type ProjectileEv = { t: number; k: "projectile"; src: number; dst: number; path: [number, number][] };

export type Ev = EngineEv | ProjectileEv;
