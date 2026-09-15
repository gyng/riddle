// Thin progress bar at the top of the viewport while the engine worker is busy (offline batch,
// forecast, verdict). Indeterminate: the worker cannot report progress from inside a wasm call.
import { h } from "./dom";

let bar: HTMLElement | null = null;
let label: HTMLElement | null = null;
const active = new Map<number, string>();
let next = 1;

function paint(): void {
  const texts = [...active.values()];
  if (!texts.length) { bar?.remove(); bar = null; label = null; return; }
  if (!bar) {
    label = h("span", { class: "busy-label" });
    bar = h("div", { class: "busy", role: "progressbar" }, h("span", { class: "busy-fill" }), label);
    document.body.appendChild(bar);
  }
  label!.textContent = texts[texts.length - 1];
}

export type Busy = { done(): void; set(text: string): void };
/** Shows the bar with a label (copy surface: label) until `done()`; `set` updates the label. */
export function showBusy(text: string): Busy {
  const id = next++;
  active.set(id, text);
  paint();
  return { done: () => { active.delete(id); paint(); }, set: (t) => { if (active.has(id)) { active.set(id, t); paint(); } } };
}
