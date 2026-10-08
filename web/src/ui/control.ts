// Take control (owner 2026-10-08, a secondary mode): in the watched run the player can drive the hero by hand. The core waits at
// each of his turns (`Snapshot.awaiting`) for one action (`engine.act`): a step (a foe on the tile is attacked), a verb as the rows
// write it (drink heal · descend · return · attack lowest), or a wait; `Release` hands him back to the rules (`takeControl(false)`).
// The panel is the watch's: a toggle, an 8-way pad and the turn's actions; arrows / WASD / numpad and `.` drive it too.
import { h } from "./dom";
import { tile } from "./frame";
import type { ManualAct, Snapshot } from "../engine/types";

export type ControlEngine = { takeControl?(on: boolean): Promise<void> | void; act?(a: ManualAct): Promise<void> | void };
export type Control = { el: HTMLElement; paint(s: Snapshot | null): void; on(): boolean; dispose(): void };

const DIRS: [number, number, string][] = [[-1, -1, "↖"], [0, -1, "↑"], [1, -1, "↗"], [-1, 0, "←"], [0, 0, "·"], [1, 0, "→"], [-1, 1, "↙"], [0, 1, "↓"], [1, 1, "↘"]];
const KEYS: Record<string, [number, number]> = {
  ArrowUp: [0, -1], ArrowDown: [0, 1], ArrowLeft: [-1, 0], ArrowRight: [1, 0], w: [0, -1], s: [0, 1], a: [-1, 0], d: [1, 0],
  Numpad8: [0, -1], Numpad2: [0, 1], Numpad4: [-1, 0], Numpad6: [1, 0], Numpad7: [-1, -1], Numpad9: [1, -1], Numpad1: [-1, 1], Numpad3: [1, 1],
};

/** The watch's take-control panel; `kick` asks the watch to step the world once an action is queued. */
export function controlPanel(engine: ControlEngine, kick: () => void): Control {
  let mine = false, snap: Snapshot | null = null, busy = false;
  // the console's own pieces (frame.ts `tile`: the carved tile, its icon, the pressed frame on tap), set in a stone slab
  const toggleSlot = h("div", { class: "cmd ctl-toggle-slot" });
  const pad = h("div", { class: "cmd ctl-pad", role: "group", "aria-label": "move" });
  const verbs = h("div", { class: "cmd ctl-verbs", role: "group", "aria-label": "act" });
  const el = h("div", { class: "ctl ctl-slab", "data-on": "0" }, toggleSlot, pad, verbs);
  const send = async (a: ManualAct): Promise<void> => {
    if (!mine || busy || !engine.act) return;
    busy = true;
    try { await engine.act(a); } catch { /* not his turn: the next paint says so */ } finally { busy = false; }
    kick();
  };
  const ROT: Record<string, number> = { "0,-1": 0, "1,-1": 45, "1,0": 90, "1,1": 135, "0,1": 180, "-1,1": 225, "-1,0": 270, "-1,-1": 315 };
  for (const [dx, dy, glyph] of DIRS) {
    const wait = !dx && !dy;
    const b = tile({ id: wait ? "ctl-wait" : `ctl-${dx}${dy}`, label: wait ? /* copy:button */ "wait" : glyph, icon: wait ? "ctl_wait" : "ctl_arrow", glyph: wait ? "⧗" : "▲",
      cls: `ctl-dir${wait ? " ctl-wait" : ""}`, onclick: () => void send(wait ? { k: "wait" } : { k: "step", dx, dy }) });
    b.dataset.dx = String(dx); b.dataset.dy = String(dy);
    b.setAttribute("aria-label", wait ? "wait" : `step ${glyph}`);
    if (!wait) b.style.setProperty("--rot", `${ROT[`${dx},${dy}`]}deg`);
    pad.appendChild(b);
  }
  const verb = (label: string, icon: string, v: string, a?: string): HTMLElement => {
    const b = tile({ id: `ctl-${v}`, label, icon, cls: "ctl-verb", onclick: () => void send({ k: "verb", verb: a ? { v, a } : { v } }) });
    b.dataset.verb = a ? `${v} ${a}` : v;
    return b;
  };
  const paintToggle = (): void => {
    const t = tile({ id: "ctl-toggle", label: mine ? /* copy:button */ "release" : /* copy:button */ "control", icon: mine ? "ctl_release" : "ctl_hand", glyph: "✋", cls: "ctl-toggle", on: mine,
      onclick: async () => {
        if (!engine.takeControl) return;
        try { await engine.takeControl(!mine); mine = !mine; } catch { mine = false; }
        paint(snap); kick();
      } });
    t.setAttribute("aria-pressed", String(mine));
    toggleSlot.replaceChildren(t);
  };
  const onKey = (e: KeyboardEvent): void => {
    if (!mine || e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const d = KEYS[e.key] ?? KEYS[e.code];
    if (d) { e.preventDefault(); void send({ k: "step", dx: d[0], dy: d[1] }); }
    else if (e.key === "." || e.code === "Numpad5") { e.preventDefault(); void send({ k: "wait" }); }
  };
  window.addEventListener("keydown", onKey);
  let painted = "";
  function paint(s: Snapshot | null): void {
    snap = s;
    const live = !!s && !!engine.takeControl;
    el.hidden = !live;
    if (s) mine = !!s.manual;   // (the core leaves `manual` out when false)
    el.dataset.on = mine ? "1" : "0";
    el.dataset.awaiting = s?.awaiting ? "1" : "0";
    pad.hidden = verbs.hidden = !mine;
    // the turn's actions: what he holds decides what shows (repainted only when that changes)
    const inv = s?.hero.inv ?? [];
    const heal = inv.some((i) => i.kind === "heal" && i.known), fire = inv.some((i) => i.kind === "fire" && i.known);
    const key = `${mine}|${heal}|${fire}`;
    if (key === painted) return;
    painted = key;
    paintToggle();
    if (!mine) return;
    const kids: HTMLElement[] = [verb(/* copy:button */ "attack", "v_attack", "attack", "nearest")];
    if (heal) kids.push(verb(/* copy:button */ "drink", "v_drink", "drink", "heal"));
    if (fire) kids.push(verb(/* copy:button */ "throw", "v_throw", "throw", "fire"));
    kids.push(verb(/* copy:button */ "descend", "v_descend", "descend"), verb(/* copy:button */ "return", "bail", "return"));
    verbs.replaceChildren(...kids);
  }
  return { el, paint, on: () => mine, dispose: () => window.removeEventListener("keydown", onKey) };
}
