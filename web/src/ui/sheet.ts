// Bottom sheets, stackable (a companion's rule sheet opens token sheets on top of it).
// Dismissed by backdrop tap or Escape; closeAll() on screen change.
// QA on 952e306 ("▸ on chronicle opens the OLD death screen, no back/close, Escape inert"): with no sheet open, Escape
// falls through to `onEscapeIdle` (the app: a kept death back to the camp).
import { h } from "./dom";

const stack: HTMLElement[] = [];
let idle: (() => void) | null = null;

export function closeSheet(): void { stack.pop()?.remove(); }
export function closeAllSheets(): void { while (stack.length) closeSheet(); }
export const sheetOpen = (): boolean => stack.length > 0;
/** What Escape does when no sheet is open (one handler; the app registers it). */
export function onEscapeIdle(fn: (() => void) | null): void { idle = fn; }

export function openSheet(build: (close: () => void) => Node): void {
  const panel = h("div", { class: "sheet", role: "dialog" });
  const wrap = h("div", { class: "sheet-wrap", onclick: (e) => { if (e.target === wrap) close(); } });
  const close = (): void => { const i = stack.indexOf(wrap); if (i >= 0) { stack.splice(i, 1); wrap.remove(); } };
  wrap.appendChild(panel);
  panel.appendChild(build(close));
  document.body.appendChild(wrap);
  stack.push(wrap);
}
window.addEventListener("keydown", (e) => { if (e.key !== "Escape") return; if (stack.length) closeSheet(); else idle?.(); });
