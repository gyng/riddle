// Take control (owner 2026-10-08, a secondary mode): in the watched run the player can drive the hero by hand. The core waits at
// each of his turns (`Snapshot.awaiting`) for one action (`engine.act`): a step (a foe on the tile is attacked), a verb as the rows
// write it (drink heal · descend · return · attack lowest), or a wait; `Release` hands him back to the rules (`takeControl(false)`).
// The panel is the watch's: a toggle, an 8-way pad and the turn's actions; arrows / WASD / numpad and `.` drive it too.
import { h } from "./dom";
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
  const toggle = h("button", { class: "chip ctl-toggle", "aria-pressed": "false", "data-ctl": "toggle" }, /* copy:button */ "Take control");
  const pad = h("div", { class: "ctl-pad", role: "group", "aria-label": "move" });
  const verbs = h("div", { class: "ctl-verbs", role: "group", "aria-label": "act" });
  const el = h("div", { class: "ctl", "data-on": "0" }, toggle, pad, verbs);
  const send = async (a: ManualAct): Promise<void> => {
    if (!mine || busy || !engine.act) return;
    busy = true;
    try { await engine.act(a); } catch { /* not his turn: the next paint says so */ } finally { busy = false; }
    kick();
  };
  for (const [dx, dy, glyph] of DIRS) {
    const b = h("button", { class: "chip ctl-dir", "data-dx": dx, "data-dy": dy, "aria-label": dx || dy ? `step ${glyph}` : "wait" }, glyph);
    b.onclick = () => void send(dx || dy ? { k: "step", dx, dy } : { k: "wait" });
    pad.appendChild(b);
  }
  const verb = (label: string, v: string, a?: string): HTMLElement => {
    const b = h("button", { class: "chip mini ctl-verb", "data-verb": a ? `${v} ${a}` : v }, label);
    b.onclick = () => void send({ k: "verb", verb: a ? { v, a } : { v } });
    return b;
  };
  toggle.onclick = async () => {
    if (!engine.takeControl) return;
    try { await engine.takeControl(!mine); mine = !mine; } catch { mine = false; }
    paint(snap); kick();
  };
  const onKey = (e: KeyboardEvent): void => {
    if (!mine || e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const d = KEYS[e.key] ?? KEYS[e.code];
    if (d) { e.preventDefault(); void send({ k: "step", dx: d[0], dy: d[1] }); }
    else if (e.key === "." || e.code === "Numpad5") { e.preventDefault(); void send({ k: "wait" }); }
  };
  window.addEventListener("keydown", onKey);
  function paint(s: Snapshot | null): void {
    snap = s;
    const live = !!s && !!engine.takeControl;
    el.hidden = !live;
    if (s) mine = !!s.manual;   // (the core leaves `manual` out when false)
    el.dataset.on = mine ? "1" : "0";
    el.dataset.awaiting = s?.awaiting ? "1" : "0";
    toggle.setAttribute("aria-pressed", String(mine));
    toggle.textContent = mine ? /* copy:button */ "Release" : /* copy:button */ "Take control";
    pad.hidden = verbs.hidden = !mine;
    if (!mine || !s) return;
    // the turn's actions: what he holds and where he stands decide what shows
    const inv = s.hero.inv ?? [];
    const kids: HTMLElement[] = [verb(/* copy:button */ "attack", "attack", "nearest")];
    if (inv.some((i) => i.kind === "heal" && i.known)) kids.push(verb(/* copy:button */ "drink heal", "drink", "heal"));
    const fire = inv.find((i) => i.kind === "fire" && i.known);
    if (fire) kids.push(verb(/* copy:button */ "throw fire", "throw", "fire"));
    kids.push(verb(/* copy:button */ "descend", "descend"), verb(/* copy:button */ "return", "return"));
    verbs.replaceChildren(...kids);
  }
  return { el, paint, on: () => mine, dispose: () => window.removeEventListener("keydown", onKey) };
}
