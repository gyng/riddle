// Bottom sheets, stackable (Cut 17 §2: iron-framed parchment panels with a close stud, unfolding over the console) (a companion's rule sheet opens token sheets on top of it).
// Dismissed by backdrop tap or Escape; closeAll() on screen change.
// QA on 952e306 ("▸ on chronicle opens the OLD death screen, no back/close, Escape inert"): with no sheet open, Escape
// falls through to `onEscapeIdle` (the app: a kept death back to the camp).
import { h } from "./dom";
import { stud } from "./frame";

const stack: HTMLElement[] = [];
let idle: (() => void) | null = null;
let panelEscape: (() => boolean) | null = null;
/** Cut 17 §2: the camp's open panel takes Escape before `onEscapeIdle` (returns whether it closed one). */
export function setPanelEscape(fn: (() => boolean) | null): void { panelEscape = fn; }

export function closeSheet(): void { stack.pop()?.remove(); }
export function closeAllSheets(): void { while (stack.length) closeSheet(); }
export const sheetOpen = (): boolean => stack.length > 0;
/** QA 23ed91f (L: the settings sheet opened over the open UNLOCKS panel — two studs): every sheet and the camp's panel closed. */
export function closeEverything(): void { closeAllSheets(); panelEscape?.(); }
/** A sheet's explicit close: `×` at the end of its title line (Escape and the backdrop close it too). */
export const closeX = (close: () => void): HTMLElement => h("button", { class: "x sheet-x stud", "aria-label": "close", onclick: () => close() }, "×");
/** What Escape does when no sheet is open (one handler; the app registers it). */
export function onEscapeIdle(fn: (() => void) | null): void { idle = fn; }

/** `modeless`: the backdrop lets taps through to the screen under it (the watch's HUD around the cage sheet — QA on 3d71c33:
 *  "⏸ and ▶▶| don't work while the cage sheet waits"); only the panel takes taps, Escape still closes it. */
/** `anchor` (Cut 23 §4, AI/AJ: "the option sheet covers the chips it edits"): the element the sheet edits (a row's chips) — the panel
 *  unfolds on whichever side of it has more room and never over it (its height capped to that side), re-placed as its body changes.
 *  A tap outside the panel (the backdrop, the anchor's row under it) only closes the sheet: it never reaches what lies beneath. */
export function openSheet(build: (close: () => void) => Node, opts: { modeless?: boolean; anchor?: HTMLElement | null } = {}): void {
  const panel = h("div", { class: "sheet", role: "dialog" });
  // Cut 24 §5 (AK: "the chip tap didn't open the verb sheet a second time"): a tap on another chip of the anchor's own row (the verb
  // while the cond sheet is up) closes this sheet and opens that one — the row being edited stays live; anywhere else a tap only closes
  const opener = opts.anchor && document.activeElement instanceof HTMLElement && opts.anchor.contains(document.activeElement) ? document.activeElement : null;
  const passThrough = (e: MouseEvent): HTMLElement | null => {
    const a = opts.anchor; if (!a || !a.isConnected) return null;
    const r = a.getBoundingClientRect();
    if (e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom) return null;
    wrap.style.pointerEvents = "none"; const under = document.elementFromPoint(e.clientX, e.clientY); wrap.style.pointerEvents = "";
    const btn = under?.closest<HTMLElement>("button.chip");
    return btn && a.contains(btn) && btn !== opener ? btn : null;
  };
  const wrap = h("div", { class: `sheet-wrap${opts.modeless ? " modeless" : ""}`, onclick: (e) => { if (e.target !== wrap) return; const pass = passThrough(e as MouseEvent); close(); pass?.click(); } });
  let ro: ResizeObserver | null = null;
  const close = (): void => { const i = stack.indexOf(wrap); if (i >= 0) { stack.splice(i, 1); wrap.remove(); ro?.disconnect(); opts.anchor?.classList.remove("sheet-anchor"); } };
  wrap.appendChild(panel);
  panel.appendChild(build(close));
  // Cut 17 §2: every sheet is a panel with a close stud (×) top-right — its own `closeX` when the body carries one
  if (!panel.querySelector(".sheet-x")) panel.prepend(stud(close));
  document.body.appendChild(wrap);
  stack.push(wrap);
  const anchor = opts.anchor;
  if (anchor && anchor.isConnected) {
    anchor.classList.add("sheet-anchor");
    const place = (): void => { if (anchor.isConnected) placeBeside(wrap, panel, anchor); };
    place();
    if (typeof ResizeObserver !== "undefined") { ro = new ResizeObserver(() => place()); ro.observe(panel.firstElementChild ?? panel); }
  }
}
/** The side of `anchor` with more room (inside the backdrop) takes the panel: below it (the bottom sheet, its height capped so its
 *  top stays under the anchor) or above it (hung from the backdrop's top, its height capped so it ends over the anchor). */
function placeBeside(wrap: HTMLElement, panel: HTMLElement, anchor: HTMLElement): void {
  const a = anchor.getBoundingClientRect(), w = wrap.getBoundingClientRect(), GAP = 6;
  const below = w.bottom - a.bottom - GAP, above = a.top - w.top - GAP;
  const top = above > below;
  wrap.classList.toggle("anchored-top", top);
  wrap.classList.add("anchored");
  panel.style.maxHeight = `${Math.max(120, Math.floor(top ? above : below))}px`;
  wrap.dataset.side = top ? "above" : "below";
}
window.addEventListener("keydown", (e) => { if (e.key !== "Escape") return; if (stack.length) closeSheet(); else if (!panelEscape?.()) idle?.(); });
