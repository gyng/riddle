// Rust-owned played gun state, rendered at the picture's clock.
import type { GunSnap } from "../engine/types";
import { h } from "./dom";
import { detailHost } from "./tips";
export function gunStatus(): { el: HTMLElement; paint(g: GunSnap | null, tick: number): void } {
  const text = h("span", { class: "gun-count num" });
  const fill = h("span", { class: "gun-reload-fill" });
  const track = h("span", { class: "gun-reload-track" }, fill);
  const el = h("span", { class: "gun-hud", hidden: true }, text, track);
  let details = "";
  detailHost(el, () => [h("p", null, details)]);
  return { el, paint(g, tick) {
    el.hidden = !g;
    if (!g) return;
    const left = g.reload_until === undefined ? g.reload_left : Math.max(0, g.reload_until - tick);
    const reloading = left > 0;
    text.textContent = `${g.loaded}/${g.capacity} · ${reloading ? "Reload" : g.aiming ? "Aim" : "Ready"}`;
    el.dataset.state = reloading ? "reload" : g.aiming ? "aim" : "ready";
    track.hidden = !reloading;
    fill.style.width = `${Math.max(0, Math.min(1, 1 - left / Math.max(1, g.reload_ticks))) * 100}%`;
    details = `${g.kind === "long_gun" ? "Long gun" : "Short gun"} · ${g.damage[0]}–${g.damage[1]} damage · range ${g.range} tiles · ${g.reload_ticks / 10}s reload${g.armour_piercing ? ` · ignores ${g.armour_piercing} armour` : ""}`;
    el.setAttribute("aria-label", details);
  } };
}
