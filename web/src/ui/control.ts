// Take control (owner 2026-10-08, a secondary mode): in the watched run the player can drive the hero by hand. The core waits at
// each of his turns (`Snapshot.awaiting`) for one action (`engine.act`): a step (a foe on the tile is attacked), a verb as the rows
// write it (drink heal · descend · return · attack lowest), or a wait; `Release` hands him back to the rules (`takeControl(false)`).
// The panel is the watch's: a toggle, an 8-way pad and the turn's actions.
//   keys (owner 2026-10-10: "keyboard works? and consider mobile/touch"): arrows / WASD / numpad step (diagonals 7·9·1·3), `.` or
//     numpad 5 waits, `f` attack · `g` attack until · `q` drink · `t` throw · `>` or Enter descend · `r` return — only while in hand;
//     `c` takes and releases him. Never inside a field, never under a sheet. Each tile names its key in its tooltip, and (a mouse)
//     in its corner.
//   touch: a held arrow repeats (REPEAT_MS) until released, refused or no longer awaited; a tap on the map steps one tile toward it,
//     a long-press walks there; a refused order buzzes (8 ms).
//   mouse (owner: "d3 style mouse controls"): a click on the floor walks there (a path over the tiles he knows, one `step` per turn)
//     until he arrives, a new foe shows or closes in, an order is refused, or another click; held, he follows the cursor; a click
//     on a foe walks to it and fights it (`attack until`); a right-click throws fire if he holds it, else `attack until`; Escape or a
//     click on him stops. The pathing is the client's (the core's `step` stays the one order); a core `walk to` would spare a round
//     trip per tile.
import { h } from "./dom";
import { tile } from "./frame";
import { sheetOpen } from "./sheet";
import type { ManualAct, Snapshot } from "../engine/types";

export type ControlEngine = { takeControl?(on: boolean): Promise<void> | void; act?(a: ManualAct): Promise<void> | void };
/** What the map input reads off the viewer: its scale, the hero's render tile, and his drawn rect (canvas CSS px). */
export type MapView = { stats?(): unknown; debugRects?(): unknown };
type ViewStats = { k: number; dpr: number; hero: [number, number] };
type Box = { x: number; y: number; w: number; h: number };
type HeroRect = Box & { id: number; hero: boolean; drawn?: Box };
export type Control = { el: HTMLElement; paint(s: Snapshot | null): void; on(): boolean; dispose(): void;
  /** the map's taps and clicks (the watch's canvas): consumed while in hand */
  bindMap(canvas: HTMLElement, view: () => MapView | null): void };

const DIRS: [number, number, string, string][] = [[-1, -1, "↖", "7"], [0, -1, "↑", "W"], [1, -1, "↗", "9"], [-1, 0, "←", "A"], [0, 0, "·", "."], [1, 0, "→", "D"], [-1, 1, "↙", "1"], [0, 1, "↓", "S"], [1, 1, "↘", "3"]];
const KEYS: Record<string, [number, number]> = {
  ArrowUp: [0, -1], ArrowDown: [0, 1], ArrowLeft: [-1, 0], ArrowRight: [1, 0], w: [0, -1], s: [0, 1], a: [-1, 0], d: [1, 0],
  Numpad8: [0, -1], Numpad2: [0, 1], Numpad4: [-1, 0], Numpad6: [1, 0], Numpad7: [-1, -1], Numpad9: [1, -1], Numpad1: [-1, 1], Numpad3: [1, 1],
};

/** `attack until`'s hp line (%): the order ends when his hp drops under it. */
export const UNTIL_HP = 30;
/** a held arrow's repeat (ms) */
export const REPEAT_MS = 220;
/** a walk's pace: at most one order per this many ms (the picture keeps up) */
const WALK_MS = 140;
/** a mouse press held this long follows the cursor; a touch held this long walks to the tile */
const HOLD_MS = 250, LONG_MS = 450;
/** a drive gives up when he has not awaited an order this long (the run ended, a long paralysis) */
const STALL_MS = 1500;
const TILE_TEXELS = 8;

type Drive = { k: "repeat"; dx: number; dy: number }
  | { k: "walk"; tx: number; ty: number; foe?: number; seen: Set<number>; adj: Set<number> }
  | { k: "follow"; tx: number; ty: number };

const hostile = (e: { ally?: boolean; kind: string; tags: string[] }): boolean => !e.ally && e.kind !== "bones" && e.kind !== "captive" && !e.tags.includes("captive") && !e.tags.includes("ally");
const cheb = (ax: number, ay: number, bx: number, by: number): number => Math.max(Math.abs(ax - bx), Math.abs(ay - by));
/** the foes he can see now (the core's own `visible`) */
const foesInView = (s: Snapshot): Snapshot["entities"] => s.entities.filter((e) => hostile(e) && !e.remembered && e.hp > 0 && !!s.visible[e.y * s.w + e.x]);
const buzz = (): void => { try { navigator.vibrate?.(8); } catch { /* no haptics */ } };

/** The next step of a path from the hero toward (tx, ty) over the tiles he has seen (the core's `can_step`: no wall corner cut,
 *  no one else's tile but the goal's); unreachable → toward the reachable tile nearest it. null: he is there, or boxed in. */
export function pathStep(s: Snapshot, tx: number, ty: number): [number, number] | null {
  const { w, h: hh } = s, hx = s.hero.x, hy = s.hero.y;
  if (hx === tx && hy === ty) return null;
  const at = (x: number, y: number): string | undefined => (x < 0 || y < 0 || x >= w || y >= hh ? undefined : s.tiles[y * w + x]);
  const open = (x: number, y: number): boolean => { const t = at(x, y); return t !== undefined && t !== "wall" && t !== "chasm" && !!s.seen[y * w + x]; };
  const taken = new Set<number>();
  for (const e of s.entities) if (!e.remembered && e.id !== s.hero.id) taken.add(e.y * w + e.x);
  const prev = new Int32Array(w * hh).fill(-1), start = hy * w + hx;
  prev[start] = start;
  const q = [start];
  let best = start, bestD = cheb(hx, hy, tx, ty);
  for (let qi = 0; qi < q.length; qi++) {
    const i = q[qi]!, x = i % w, y = (i - x) / w;
    const d = cheb(x, y, tx, ty);
    if (d < bestD) { best = i; bestD = d; }
    if (d === 0) break;
    for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) {
      if (!dx && !dy) continue;
      const nx = x + dx, ny = y + dy, j = ny * w + nx;
      if (!open(nx, ny) || prev[j] !== -1) continue;
      if (dx && dy && (at(x, ny) === "wall" || at(nx, y) === "wall")) continue;
      if (taken.has(j) && !(nx === tx && ny === ty)) continue;
      prev[j] = i; q.push(j);
    }
  }
  if (best === start) return null;
  let i = best;
  while (prev[i] !== start) i = prev[i]!;
  const x = i % w;
  return [x - hx, (i - x) / w - hy];
}

/** The watch's take-control panel; `kick` asks the watch to step the world once an action is queued. */
export function controlPanel(engine: ControlEngine, kick: () => void): Control {
  let mine = false, snap: Snapshot | null = null, busy = false;
  // a standing order the panel drives a turn at a time (a held arrow, a walk, the cursor followed); `owed`: an order sent whose
  // snapshot has not landed yet (the next waits for it)
  let drive: Drive | null = null, owed = false, owedAt = 0, lastSend = 0, idleSince = 0, timer = 0, acts = 0;
  // the console's own pieces (frame.ts `tile`: the carved tile, its icon, the pressed frame on tap), set in a stone slab
  const toggleSlot = h("div", { class: "cmd ctl-toggle-slot" });
  const pad = h("div", { class: "cmd ctl-pad", role: "group", "aria-label": "move" });
  const verbs = h("div", { class: "cmd ctl-verbs", role: "group", "aria-label": "act" });
  // blind b58b431 (A: arrows and `descend` inert in gas — no response at all): the last order's outcome, the core's words
  const note = h("div", { class: "ctl-note num", role: "status", "aria-live": "polite" });
  const el = h("div", { class: "ctl ctl-slab", "data-on": "0" }, toggleSlot, pad, verbs, note);
  const refused = (why: string): void => { note.textContent = why; note.dataset.refused = "1"; buzz(); stopDrive(); };
  const send = async (a: ManualAct, driven = false): Promise<void> => {
    if (!mine || busy || !engine.act) return;
    if (!driven) stopDrive("order");
    busy = true;
    note.dataset.pending = "1";
    try { await engine.act(a); owed = true; owedAt = performance.now(); acts++; el.dataset.acts = String(acts); }
    catch (e) { refused(/* copy:callout */ "refused"); note.title = e instanceof Error ? e.message : ""; }
    finally { busy = false; lastSend = performance.now(); }
    kick();
  };
  // --- the standing order ---
  function stopDrive(why = ""): void { if (drive && why && !el.dataset.stop) el.dataset.stop = `${drive.k} ${why}`; drive = null; clearInterval(timer); timer = 0; idleSince = 0; el.dataset.drive = ""; }
  function startDrive(d: Drive): void {
    delete el.dataset.stop;
    drive = d; el.dataset.drive = d.k; idleSince = 0;
    if (!timer) timer = window.setInterval(driveTick, 40);
    driveTick();
  }
  function walkTo(tx: number, ty: number, foe?: number): void {
    if (!snap) return;
    const s = snap, hx = s.hero.x, hy = s.hero.y, seen = foesInView(s);
    startDrive({ k: "walk", tx, ty, foe, seen: new Set(seen.map((e) => e.id)), adj: new Set(seen.filter((e) => cheb(e.x, e.y, hx, hy) <= 1).map((e) => e.id)) });
  }
  /** the drive's next order: an act, `wait` (nothing this turn), or null (the drive is over) */
  function nextAct(d: Drive, s: Snapshot): ManualAct | "hold" | null {
    if (d.k === "repeat") return { k: "step", dx: d.dx, dy: d.dy };
    const hx = s.hero.x, hy = s.hero.y;
    if (d.k === "walk" && d.foe !== undefined) {
      const f = foesInView(s).find((e) => e.id === d.foe);
      if (!f) { el.dataset.stop = "walk foe gone"; return null; }   // fallen, or out of sight: the player decides what next
      if (cheb(f.x, f.y, hx, hy) <= 1) { drive = null; return { k: "attack_until", hp: UNTIL_HP }; }
      d.tx = f.x; d.ty = f.y;
    } else if (d.k === "walk") {
      // a new foe in view, or one come up beside him, ends the walk (the player decides what next)
      for (const f of foesInView(s)) if (!d.seen.has(f.id) || (cheb(f.x, f.y, hx, hy) <= 1 && !d.adj.has(f.id))) { el.dataset.stop = "walk foe"; return null; }
    }
    const st = pathStep(s, d.tx, d.ty);
    if (!st) return d.k === "follow" ? "hold" : null;
    return { k: "step", dx: st[0], dy: st[1] };
  }
  function driveTick(): void {
    const d = drive;
    if (!d) { stopDrive(); return; }
    if (!mine || !snap) { stopDrive(); return; }
    const now = performance.now();
    if (owed && now - owedAt > 3000) owed = false;   // (a snapshot lost to the run's end: the drive goes on reading)
    if (busy || owed) return;
    if (!snap.awaiting) { if (!idleSince) idleSince = now; else if (now - idleSince > STALL_MS) stopDrive("stall"); return; }
    idleSince = 0;
    if (now - lastSend < (d.k === "repeat" ? REPEAT_MS : WALK_MS)) return;
    const a = nextAct(d, snap);
    if (a === "hold") return;
    if (!a) { stopDrive("over"); return; }
    if (!drive) stopDrive();   // (a last order: `attack until` on the foe reached)
    void send(a, true);
  }
  // --- the pad ---
  let padPtr = false, padHeld = false;   // a pointer press already sent its step (the click that follows sends nothing); still held
  const ROT: Record<string, number> = { "0,-1": 0, "1,-1": 45, "1,0": 90, "1,1": 135, "0,1": 180, "-1,1": 225, "-1,0": 270, "-1,-1": 315 };
  /** a tile's key: in its tooltip, and in its corner where a mouse can hover (CSS) */
  const keyed = <E extends HTMLElement>(b: E, key: string, tip: string): E => {
    b.title = tip;
    b.appendChild(h("kbd", { class: "ctl-key", "aria-hidden": "true" }, key));
    b.setAttribute("aria-keyshortcuts", key === "." ? "." : key);
    return b;
  };
  for (const [dx, dy, glyph, key] of DIRS) {
    const wait = !dx && !dy;
    const act: ManualAct = wait ? { k: "wait" } : { k: "step", dx, dy };
    const b = tile({ id: wait ? "ctl-wait" : `ctl-${dx}${dy}`, label: wait ? /* copy:button */ "wait" : glyph, icon: wait ? "ctl_wait" : "ctl_arrow", glyph: wait ? "⧗" : "▲",
      cls: `ctl-dir${wait ? " ctl-wait" : ""}`, onclick: () => { if (padPtr) { padPtr = false; return; } void send(act); } });
    b.dataset.dx = String(dx); b.dataset.dy = String(dy);
    b.setAttribute("aria-label", wait ? "wait" : `step ${glyph}`);
    keyed(b, key, wait ? /* copy:tooltip */ "wait · 5 or ." : `${glyph} · ${key}`);
    if (!wait) {
      b.style.setProperty("--rot", `${ROT[`${dx},${dy}`]}deg`);
      // a held arrow repeats while held (touch, or the mouse): the press steps now, the drive repeats
      b.addEventListener("pointerdown", (e: PointerEvent) => {
        if (e.button !== 0 || !mine) return;
        padPtr = padHeld = true;
        try { b.setPointerCapture(e.pointerId); } catch { /* a synthetic pointer */ }
        void send(act).then(() => { if (padHeld && mine) { lastSend = performance.now(); startDrive({ k: "repeat", dx, dy }); } });
      });
      const up = (): void => { padHeld = false; if (drive?.k === "repeat") stopDrive(); };
      for (const ev of ["pointerup", "pointercancel", "lostpointercapture"]) b.addEventListener(ev, up);
    }
    pad.appendChild(b);
  }
  // the up after a press lands after the press's step resolved: the repeat must not outlive it
  pad.addEventListener("pointerup", () => { padHeld = false; if (drive?.k === "repeat") stopDrive(); setTimeout(() => { padPtr = false; }, 0); });
  pad.addEventListener("contextmenu", (e) => e.preventDefault());
  const verb = (label: string, icon: string, v: string, a: string | undefined, key: string, tip: string): HTMLElement => {
    const b = tile({ id: `ctl-${v}`, label, icon, cls: "ctl-verb", onclick: () => void send({ k: "verb", verb: a ? { v, a } : { v } }) });
    b.dataset.verb = a ? `${v} ${a}` : v;
    b.setAttribute("aria-label", label);
    return keyed(b, key, tip);
  };
  async function toggle(): Promise<void> {
    if (!engine.takeControl) return;
    stopDrive();
    try { await engine.takeControl(!mine); mine = !mine; } catch { mine = false; }
    painted = ""; paint(snap); kick();
  }
  const paintToggle = (): void => {
    const t = tile({ id: "ctl-toggle", label: mine ? /* copy:button */ "release" : /* copy:button */ "control", icon: mine ? "ctl_release" : "ctl_hand", glyph: "✋", cls: "ctl-toggle", on: mine,
      onclick: () => void toggle() });
    t.setAttribute("aria-pressed", String(mine));
    keyed(t, "C", mine ? /* copy:tooltip */ "back to the rules · C" : /* copy:tooltip */ "drive him by hand · C");
    toggleSlot.replaceChildren(t);
  };
  // --- keys ---
  const typing = (t: EventTarget | null): boolean => t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));
  const modal = (): boolean => sheetOpen() || !!document.querySelector("dialog[open], [aria-modal=true]");
  const VERB_KEYS: Record<string, ManualAct> = {
    f: { k: "verb", verb: { v: "attack", a: "nearest" } }, g: { k: "attack_until", hp: UNTIL_HP }, q: { k: "verb", verb: { v: "drink", a: "heal" } },
    t: { k: "verb", verb: { v: "throw", a: "fire" } }, ">": { k: "verb", verb: { v: "descend" } }, r: { k: "verb", verb: { v: "return" } },
  };
  const onKey = (e: KeyboardEvent): void => {
    if (e.ctrlKey || e.metaKey || e.altKey || el.hidden || !el.isConnected || typing(e.target) || modal()) return;
    if (e.key === "Escape") { if (drive) stopDrive(); return; }
    const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    if (k === "c") { if (e.repeat) return; e.preventDefault(); void toggle(); return; }
    if (!mine) return;
    const d = KEYS[k] ?? KEYS[e.code];
    if (d) { e.preventDefault(); void send({ k: "step", dx: d[0], dy: d[1] }); return; }
    if (k === "." || e.code === "Numpad5") { e.preventDefault(); void send({ k: "wait" }); return; }
    // Enter descends, unless it is pressing the control that has focus
    if (k === "Enter" || e.code === "NumpadEnter") { if (e.target instanceof HTMLButtonElement || e.target instanceof HTMLAnchorElement) return; e.preventDefault(); void send(VERB_KEYS[">"]!); return; }
    const a = VERB_KEYS[k];
    if (a) { e.preventDefault(); void send(a); }
  };
  window.addEventListener("keydown", onKey);
  let painted = "";
  function paint(s: Snapshot | null): void {
    // the snapshot an order was waiting for: a refusal buzzes and ends any standing order
    if (s && owed) {
      owed = false;
      if (s.order?.startsWith("can't")) { buzz(); stopDrive("refused"); }
    }
    snap = s;
    const live = !!s && !!engine.takeControl;
    el.hidden = !live;
    if (s) mine = !!s.manual;   // (the core leaves `manual` out when false)
    if (!live || !mine) { stopDrive("off"); hideHover(); }
    el.dataset.on = mine ? "1" : "0";
    el.dataset.awaiting = s?.awaiting ? "1" : "0";
    pad.hidden = verbs.hidden = !mine;
    note.hidden = !mine;
    if (s?.order !== undefined && s.order !== note.textContent) { note.textContent = s.order; note.dataset.refused = s.order.startsWith("can't") ? "1" : "0"; }
    if (s && (s.awaiting || s.order)) delete note.dataset.pending;
    if (drive && s?.awaiting) queueMicrotask(driveTick);
    // the turn's actions: what he holds decides what shows (repainted only when that changes)
    const inv = s?.hero.inv ?? [];
    const heal = inv.some((i) => i.kind === "heal" && i.known), fire = inv.some((i) => i.kind === "fire" && i.known);
    holdsFire = fire;
    const key = `${mine}|${heal}|${fire}`;
    if (key === painted) return;
    painted = key;
    paintToggle();
    if (!mine) return;
    const kids: HTMLElement[] = [verb(/* copy:button */ "attack", "v_attack", "attack", "nearest", "F", /* copy:tooltip */ "attack nearest · F")];
    // blind b58b431 (B: 40 identical `attack` taps on the Warlord): one order fights on — the same foe until it falls or hp < 30%
    const until = tile({ id: "ctl-until", label: /* copy:button */ "attack until", icon: "v_attack", cls: "ctl-verb ctl-until", onclick: () => void send({ k: "attack_until", hp: UNTIL_HP }) });
    until.dataset.verb = "attack until"; until.setAttribute("aria-label", "attack until");
    keyed(until, "G", /* copy:tooltip */ `same foe until it falls or hp < ${UNTIL_HP}% · G`);
    kids.push(until);
    if (heal) kids.push(verb(/* copy:button */ "drink", "v_drink", "drink", "heal", "Q", /* copy:tooltip */ "drink heal · Q"));
    if (fire) kids.push(verb(/* copy:button */ "throw", "v_throw", "throw", "fire", "T", /* copy:tooltip */ "throw fire · T"));
    kids.push(verb(/* copy:button */ "descend", "v_descend", "descend", undefined, ">", /* copy:tooltip */ "descend · > or Enter"),
      verb(/* copy:button */ "return", "bail", "return", undefined, "R", /* copy:tooltip */ "return home · R"));
    verbs.replaceChildren(...kids);
  }
  // --- the map: taps (touch) and clicks (a mouse) ---
  let holdsFire = false;
  let hover: HTMLElement | null = null, mapEl: HTMLElement | null = null, unbind: (() => void) | null = null;
  function hideHover(): void { if (hover) hover.hidden = true; if (mapEl) mapEl.dataset.ctlCursor = ""; }
  function bindMap(canvas: HTMLElement, view: () => MapView | null): void {
    mapEl = canvas;
    hover = h("div", { class: "ctl-hover", hidden: true, "aria-hidden": "true" });
    canvas.insertAdjacentElement("afterend", hover);
    /** the tile under a point, and where that tile is drawn (CSS px, client) */
    const pick = (cx: number, cy: number): { x: number; y: number; px: number; left: number; top: number } | null => {
      const v = view(); if (!v || !snap) return null;
      const st = v.stats?.() as ViewStats | undefined, r = (v.debugRects?.() as HeroRect[] | undefined)?.find((q) => q.hero);
      if (!st?.hero || !r) return null;
      const px = (TILE_TEXELS * st.k) / st.dpr;
      if (!(px > 0)) return null;
      const c = canvas.getBoundingClientRect();
      // his tile's centre: his drawn feet stand 3 texels under it (render/index.ts `feet`)
      const hx = c.left + r.x + r.w / 2, hy = c.top + r.y + r.h - (3 * st.k) / st.dpr;
      const fx = st.hero[0] + (cx - hx) / px, fy = st.hero[1] + (cy - hy) / px;
      const x = Math.round(fx), y = Math.round(fy);
      if (x < 0 || y < 0 || x >= snap.w || y >= snap.h) return null;
      return { x, y, px, left: hx + (x - st.hero[0]) * px - px / 2, top: hy + (y - st.hero[1]) * px - px / 2 };
    };
    /** a point on his drawn sprite (a click there stops him, wherever the walk has taken him this frame) */
    const onHero = (cx: number, cy: number): boolean => {
      const r = ((view()?.debugRects?.() as HeroRect[] | undefined) ?? []).find((q) => q.hero), c = canvas.getBoundingClientRect();
      if (!r) return false; const b = r.drawn ?? r, x = cx - c.left, y = cy - c.top;
      return x >= b.x && x <= b.x + b.w && y >= b.y && y <= b.y + b.h;
    };
    /** the foe under a point: its drawn sprite (taller than its tile), else the one standing on the picked tile */
    const foeAt = (cx: number, cy: number, p: { x: number; y: number }): Snapshot["entities"][number] | undefined => {
      if (!snap) return undefined;
      const foes = foesInView(snap), c = canvas.getBoundingClientRect(), x = cx - c.left, y = cy - c.top;
      const rects = (view()?.debugRects?.() as HeroRect[] | undefined) ?? [];
      for (const r of rects) { const b = r.drawn ?? r; if (!r.hero && x >= b.x && x <= b.x + b.w && y >= b.y && y <= b.y + b.h) { const f = foes.find((e) => e.id === r.id); if (f) return f; } }
      return foes.find((e) => e.x === p.x && e.y === p.y);
    };
    const paintHover = (cx: number, cy: number): void => {
      const p = mine ? pick(cx, cy) : null;
      if (!p || !hover) { hideHover(); return; }
      const foe = !!foeAt(cx, cy, p), base = (hover.offsetParent ?? canvas.parentElement)?.getBoundingClientRect();
      canvas.dataset.ctlCursor = foe ? "attack" : "move";
      hover.dataset.foe = foe ? "1" : "0";
      Object.assign(hover.style, { left: `${p.left - (base?.left ?? 0)}px`, top: `${p.top - (base?.top ?? 0)}px`, width: `${p.px}px`, height: `${p.px}px` });
      hover.hidden = false;
    };
    let down: { x: number; y: number; at: number; mouse: boolean; moved: boolean; long: boolean } | null = null, lp = 0;
    const onDown = (e: PointerEvent): void => {
      if (!mine || !snap) return;
      const mouse = e.pointerType === "mouse";
      down = { x: e.clientX, y: e.clientY, at: performance.now(), mouse, moved: false, long: false };
      if (!mouse) {
        clearTimeout(lp);
        if (drive) { stopDrive(); down.long = true; return; }   // a tap while he walks stops him
        lp = window.setTimeout(() => { const p = down && pick(down.x, down.y); if (!down || down.moved || !p) return; down.long = true; walkTo(p.x, p.y); }, LONG_MS);
        return;
      }
      const p = pick(e.clientX, e.clientY);
      if (e.button === 2) {
        e.preventDefault();
        void send(holdsFire ? { k: "verb", verb: { v: "throw", a: "fire" } } : { k: "attack_until", hp: UNTIL_HP });
        down = null; return;
      }
      if (e.button !== 0 || !p) return;
      try { canvas.setPointerCapture(e.pointerId); } catch { /* a synthetic pointer */ }
      // a foe's sprite first (a tall foe in front of him), then him (stop), then the floor (walk)
      const f = foeAt(e.clientX, e.clientY, p);
      if (f) { walkTo(f.x, f.y, f.id); return; }
      if ((p.x === snap.hero.x && p.y === snap.hero.y) || onHero(e.clientX, e.clientY)) { stopDrive(); return; }
      walkTo(p.x, p.y);
    };
    const onMove = (e: PointerEvent): void => {
      if (e.pointerType === "mouse") paintHover(e.clientX, e.clientY); else hideHover();
      if (!down) return;
      if (Math.hypot(e.clientX - down.x, e.clientY - down.y) > 12) down.moved = true;
      // held, he follows the cursor (a press on a foe stays a fight)
      if (down.mouse && performance.now() - down.at > HOLD_MS && !(drive?.k === "walk" && drive.foe !== undefined)) {
        const p = pick(e.clientX, e.clientY);
        if (p) { if (drive?.k === "follow") { drive.tx = p.x; drive.ty = p.y; } else startDrive({ k: "follow", tx: p.x, ty: p.y }); }
      }
    };
    const onUp = (e: PointerEvent): void => {
      clearTimeout(lp);
      const d = down; down = null;
      if (!d || !mine || !snap) return;
      if (d.mouse) {
        if (drive?.k === "follow") stopDrive();
        else if (performance.now() - d.at > HOLD_MS && drive?.k === "walk" && drive.foe === undefined) {
          const p = pick(e.clientX, e.clientY); if (p) walkTo(p.x, p.y);   // a held press let go: the walk ends where it was let go
        }
        return;
      }
      // a tap: one step toward the tile (a foe there is attacked by the step)
      if (d.long || d.moved || !snap.awaiting) return;
      const p = pick(e.clientX, e.clientY); if (!p) return;
      const dx = Math.sign(p.x - snap.hero.x), dy = Math.sign(p.y - snap.hero.y);
      if (dx || dy) void send({ k: "step", dx, dy });
    };
    const onMenu = (e: Event): void => { if (mine) e.preventDefault(); };
    const onLeave = (): void => hideHover();
    canvas.addEventListener("pointerdown", onDown);
    canvas.addEventListener("pointermove", onMove);
    canvas.addEventListener("pointerup", onUp);
    canvas.addEventListener("pointercancel", onUp);
    canvas.addEventListener("pointerleave", onLeave);
    canvas.addEventListener("contextmenu", onMenu);
    unbind = () => {
      for (const [ev, fn] of [["pointerdown", onDown], ["pointermove", onMove], ["pointerup", onUp], ["pointercancel", onUp], ["pointerleave", onLeave], ["contextmenu", onMenu]] as const) canvas.removeEventListener(ev, fn as EventListener);
      hover?.remove();
    };
  }
  return { el, paint, on: () => mine, bindMap,
    dispose: () => { window.removeEventListener("keydown", onKey); stopDrive(); unbind?.(); } };
}
