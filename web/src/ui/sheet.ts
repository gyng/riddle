// Bottom sheet: one at a time, thumb-reachable, dismissed by backdrop tap or Escape.
import { h } from "./dom";

let open: HTMLElement | null = null;

export function closeSheet(): void { open?.remove(); open = null; }

export function openSheet(build: (close: () => void) => Node): void {
  closeSheet();
  const panel = h("div", { class: "sheet", role: "dialog" });
  const wrap = h("div", { class: "sheet-wrap", onclick: (e) => { if (e.target === wrap) closeSheet(); } }, panel);
  panel.appendChild(build(closeSheet));
  document.body.appendChild(wrap);
  open = wrap;
  const onKey = (e: KeyboardEvent): void => { if (e.key === "Escape") { closeSheet(); window.removeEventListener("keydown", onKey); } };
  window.addEventListener("keydown", onKey);
}
