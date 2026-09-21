// Thin progress bar at the top of the viewport while the engine worker is busy (offline batch,
// forecast, verdict). Indeterminate: the worker cannot report progress from inside a wasm call.
import { h } from "./dom";

let bar: HTMLElement | null = null;
let label: HTMLElement | null = null;
let host: HTMLElement | null = null;
const active = new Map<number, string>();
let next = 1;

function paint(): void {
  const texts = [...active.values()];
  const text = texts.length ? texts[texts.length - 1] : "";
  // a mounted host (the camp's own strip) carries the label; the fixed bar keeps only its line there
  if (host) host.textContent = text;
  if (!texts.length) { bar?.remove(); bar = null; label = null; return; }
  if (!bar) {
    label = h("span", { class: "busy-label" });
    bar = h("div", { class: "busy", role: "progressbar" }, h("span", { class: "busy-fill" }), label);
    document.body.appendChild(bar);
  }
  label!.textContent = text;
  label!.hidden = !!host;
}
/** A screen's own strip for the label (the camp's, between its header and tabs — the fixed corner label drew over `D4 ★0` and a
 *  forecast bar: QA on 50bb162); null hands the label back to the fixed bar. */
export function setBusyHost(el: HTMLElement | null): void {
  if (host && host !== el) host.textContent = "";
  host = el;
  paint();
}

export type Busy = { done(): void; set(text: string): void };
/** Shows the bar with a label (copy surface: label) until `done()`; `set` updates the label. */
export function showBusy(text: string): Busy {
  const id = next++;
  active.set(id, text);
  paint();
  return { done: () => { active.delete(id); paint(); }, set: (t) => { if (active.has(id)) { active.set(id, t); paint(); } } };
}
