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

/** Shows the bar with a label (copy surface: label, one word) until the returned function is called. */
export function showBusy(text: string): () => void {
  const id = next++;
  active.set(id, text);
  paint();
  return () => { active.delete(id); paint(); };
}
