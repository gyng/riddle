// Bottom sheets, stackable (Cut 17 §2: iron-framed parchment panels with a close stud, unfolding over the console) (a companion's rule sheet opens token sheets on top of it).
// Dismissed by backdrop tap or Escape; closeAll() on screen change.
// QA on 952e306 ("▸ on chronicle opens the OLD death screen, no back/close, Escape inert"): with no sheet open, Escape
// falls through to `onEscapeIdle` (the app: a kept death back to the camp).
import { h } from "./dom";
import { stud } from "./frame";
import { AUTO, autoDismiss } from "./autodismiss";
import { sheetGhost } from "../juice";   // juice pass 2: the close eases out (a ghost of the panel; the sheet itself goes at once)

const stack: HTMLElement[] = [];
/** Cut 25 §6 (AN: "a sheet over a sheet to buy a row"): one sheet at a time — a sheet opened from a sheet replaces it (the one under it
 *  hides, kept as it was) and carries a back (`‹`); back, Escape and the backdrop return to it, the `×` closes both. */
const closers = new Map<HTMLElement, () => void>();
const parentOf = new Map<HTMLElement, HTMLElement>();
let idle: (() => void) | null = null;
let panelEscape: (() => boolean) | null = null;
/** Cut 17 §2: the camp's open panel takes Escape before `onEscapeIdle` (returns whether it closed one). */
export function setPanelEscape(fn: (() => boolean) | null): void { panelEscape = fn; }

export function closeSheet(): void { const top = stack[stack.length - 1]; if (!top) return; const c = closers.get(top); if (c) c(); else { stack.pop(); top.remove(); } }
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
export function openSheet(build: (close: () => void) => Node, opts: { modeless?: boolean; anchor?: HTMLElement | null; stay?: boolean; center?: boolean } = {}): void {
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
  // QA 524827b (qaAA: with the SUPPLIES sheet up, a tap on the `unlocks` tile only closed the sheet — two taps for one): a tap on a
  // console tile under the backdrop closes the sheet and reaches the tile (the command card is never "beneath" a sheet; the gem is)
  const tileUnder = (e: MouseEvent): HTMLElement | null => {
    wrap.style.pointerEvents = "none"; const under = document.elementFromPoint(e.clientX, e.clientY); wrap.style.pointerEvents = "";
    const t = under?.closest<HTMLElement>("footer.console button.tile");
    return t && !(t as HTMLButtonElement).disabled ? t : null;
  };
  const wrap = h("div", { class: `sheet-wrap${opts.modeless ? " modeless" : ""}`, onclick: (e) => { if (e.target !== wrap) return; const pass = passThrough(e as MouseEvent) ?? tileUnder(e as MouseEvent); close(); pass?.click(); } });
  let ro: ResizeObserver | null = null;
  let unplace: (() => void) | undefined;
  // Cut 25 §6: the sheet this one replaces (the top non-modeless one), hidden until this one goes
  const parent = opts.modeless ? undefined : [...stack].reverse().find((w) => !w.classList.contains("modeless"));
  const close = (): void => {
    const i = stack.indexOf(wrap); if (i < 0) return;
    stack.splice(i, 1); sheetGhost(wrap); wrap.remove(); ro?.disconnect(); unplace?.(); opts.anchor?.classList.remove("sheet-anchor"); closers.delete(wrap); parentOf.delete(wrap);
    if (parent && stack.includes(parent)) { parent.hidden = false; parent.classList.remove("under"); }
  };
  /** The `×` of a sheet that replaced another closes the chain (this one and every one it replaced). */
  const closeChain = (): void => { let w: HTMLElement | undefined = wrap; while (w) { const up = parentOf.get(w); closers.get(w)?.(); w = up; } };
  closers.set(wrap, close);
  if (parent) { parentOf.set(wrap, parent); parent.hidden = true; parent.classList.add("under"); }
  wrap.appendChild(panel);
  const content = build(close);
  panel.appendChild(content);
  // Cut 17 §2: every sheet is a panel with a close stud (×) top-right — its own `closeX` when the body carries one
  if (!panel.querySelector(".sheet-x")) panel.prepend(stud(close));
  if (parent) {
    // the `×` (the body's own `closeX`, or the stud) is rebuilt without its listener: it closes the chain
    for (const x of panel.querySelectorAll<HTMLElement>(".sheet-x, .close-stud")) { const nx = x.cloneNode(true) as HTMLElement; nx.onclick = (e: Event) => { e.stopPropagation(); closeChain(); }; x.replaceWith(nx); }
    panel.prepend(h("button", { class: "sheet-back stud", "aria-label": "back", onclick: () => close() }, "‹"));
  }
  document.body.appendChild(wrap);
  stack.push(wrap);
  const x = panel.querySelector<HTMLElement>(".sheet-x, .close-stud");   // docs/UI.md §7: the close stud drains, then closes it
  if (x && !opts.modeless && !opts.stay && document.getElementById("app")?.dataset.screen !== "watch") autoDismiss(x, { ms: AUTO.panel, scope: panel, onExpire: close });
  const anchor = opts.anchor;
  // the wide frame (desktop): the panel stands beside what opened it — the anchor, else the control last tapped
  const beside = wideNow() && !opts.modeless ? (anchor?.isConnected ? anchor : lastTap?.isConnected && !wrap.contains(lastTap) ? lastTap : null) : null;
  if (opts.center && !opts.modeless) {
    wrap.classList.add("centered");
    panel.classList.add("game-window");
    const place = (): void => {
      const bar = document.querySelector("main.frame > .topbar")?.getBoundingClientRect();
      const console = document.querySelector("main.frame > .console")?.getBoundingClientRect();
      wrap.style.setProperty("--window-top", `${Math.max(0, bar?.bottom ?? 0)}px`);
      wrap.style.setProperty("--window-bottom", `${Math.max(0, console ? innerHeight - console.top : 0)}px`);
    };
    place();
    window.addEventListener("resize", place);
    unplace = () => window.removeEventListener("resize", place);
    if (typeof ResizeObserver !== "undefined") { ro = new ResizeObserver(place); ro.observe(content instanceof Element ? content : panel); }
  } else if (beside) {
    const place = (): void => { if (beside.isConnected) placeWide(wrap, panel, beside); };
    place();
    if (typeof ResizeObserver !== "undefined") { ro = new ResizeObserver(() => place()); ro.observe(content instanceof Element ? content : panel); }
    if (anchor) anchor.classList.add("sheet-anchor");
  } else if (anchor && anchor.isConnected) {
    anchor.classList.add("sheet-anchor");
    const place = (): void => { if (anchor.isConnected) placeBeside(wrap, panel, anchor); };
    place();
    if (typeof ResizeObserver !== "undefined") { ro = new ResizeObserver(() => place()); ro.observe(content instanceof Element ? content : panel); }
    // Cut 29 (cut23's flake: the camp repainted under an open verb sheet — a line above the rows went, R1 rose 63 px — and the
    // sheet hung above R1 covered it): re-placed whenever its anchor moves, while it is open
    let last = anchor.getBoundingClientRect().top;
    const follow = (): void => {
      if (!stack.includes(wrap) || !anchor.isConnected) return;
      const t = anchor.getBoundingClientRect().top;
      if (Math.abs(t - last) > 0.5) { last = t; place(); }
      requestAnimationFrame(follow);
    };
    requestAnimationFrame(follow);
  }
}
/** The wide frame (wide.css): a tap's control, kept so a sheet it opens can stand beside it. */
let lastTap: HTMLElement | null = null;
if (typeof document !== "undefined") document.addEventListener("pointerdown", (e) => { const t = e.target instanceof Element ? e.target.closest<HTMLElement>("button, [role=button], .row, .tile, .shaft") : null; if (t && !t.closest(".sheet-wrap")) lastTap = t; }, true);
const wideNow = (): boolean => typeof matchMedia !== "undefined" && matchMedia("(min-width: 1024px)").matches;
/** Desktop: the panel beside `el` — right of it when it sits in the left half, left of it in the right half, over it when it is a
 *  console tile (the bottom); its top level with the element's, kept between the bar and the console, never over the element. */
function placeWide(wrap: HTMLElement, panel: HTMLElement, el: HTMLElement): void {
  wrap.classList.add("wide-beside"); wrap.classList.remove("anchored", "anchored-top");
  const a = el.getBoundingClientRect(), w = wrap.getBoundingClientRect(), GAP = 12;
  const top0 = (document.querySelector("main.frame > .topbar")?.getBoundingClientRect().bottom ?? 0) + GAP - w.top;
  const bot = w.height - GAP;
  const pw = panel.offsetWidth, room = Math.max(160, bot - top0);
  panel.style.maxHeight = `${Math.floor(room)}px`;
  const ph = Math.min(panel.scrollHeight, room);
  const inConsole = !!el.closest("footer.console");
  let left: number, top: number;
  if (inConsole) {
    left = a.left + a.width / 2 - pw / 2 - w.left;
    top = bot - ph;
  } else {
    left = a.left + a.width / 2 < innerWidth / 2 ? a.right + GAP - w.left : a.left - GAP - pw - w.left;
    top = a.top - w.top;
  }
  left = Math.max(GAP, Math.min(w.width - pw - GAP, left));
  top = Math.max(top0, Math.min(bot - ph, top));
  panel.style.left = `${Math.round(left)}px`; panel.style.top = `${Math.round(top)}px`;
  wrap.dataset.side = inConsole ? "above" : "beside";
}
/** The side of `anchor` with more room (inside the backdrop) takes the panel: below it (the bottom sheet, its height capped so its
 *  top stays under the anchor) or above it (hung from the backdrop's top, its height capped so it ends over the anchor). */
function placeBeside(wrap: HTMLElement, panel: HTMLElement, anchor: HTMLElement): void {
  const a = anchor.getBoundingClientRect(), w = wrap.getBoundingClientRect(), GAP = 6;
  const below = w.bottom - a.bottom - GAP, above = a.top - w.top - GAP;
  const top = above > below;
  wrap.classList.toggle("anchored-top", top);
  wrap.classList.add("anchored");
  const room = Math.max(120, Math.floor(top ? above : below));
  panel.style.maxHeight = `${room}px`;
  // gfx raters ("anchor the WHY sheet to the tablet that opened it"): the panel meets its anchor — hung above it, its foot on the anchor's
  // top edge; below it, its head under the anchor's foot — not at the backdrop's far edge
  const ph = Math.min(Math.max(panel.scrollHeight, panel.offsetHeight), room), slack = Math.max(0, room - ph);
  panel.style.marginTop = top ? `${Math.floor(slack)}px` : "";
  panel.style.marginBottom = top ? "" : `${Math.floor(slack)}px`;
  wrap.dataset.side = top ? "above" : "below";
}
window.addEventListener("keydown", (e) => { if (e.key !== "Escape") return; if (stack.length) closeSheet(); else if (!panelEscape?.()) idle?.(); });

/** Full game windows center in the playable area; token pickers keep openSheet's anchor behavior. */
export function openWindow(build: (close: () => void) => Node, opts: Parameters<typeof openSheet>[1] = {}): void {
  openSheet(build, { ...opts, center: true });
}
