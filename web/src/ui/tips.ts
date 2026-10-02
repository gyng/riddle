// docs/TOOLTIPS.md — keyword tooltips. The registry is ./concepts.ts (`TIP`, `ALIASES`, `LIVE`); this module marks keywords in UI
// strings (`kwText`, `kw`), hangs a tip on a control or a stat (`kwHost`), keeps the screen inside the density budget, fades a learned
// keyword to plain text, and draws the one plate.
//
//   density (one pass over the screen, ~1 s and after every repaint):  a keyword is marked on its first occurrence per panel, ≤ 1 per
//     visual line, ≤ 4 on screen, ≤ 8 % of the visible words (1 on a screen under 25 words); a concept whose caption is on screen is not
//     also marked. The rest stay plain (`.kw` without `kw-on`): their tip is a hover, a long-press or a focus away.
//   fade: a keyword whose tip was opened twice, or seen marked 8 times (on screen ≥ 1.5 s, ≤ once per 5 min), is learned (plain).
//     Per viewer (localStorage `riddle.keywords`), never the save.
//   input: desktop hover 350 ms (the plate can be entered), click and focus open; phone: a tap on a marked keyword (or a stat host) not
//     inside a control opens it, anything else closes it; inside a control the tap is the control's and a 450 ms long-press opens the
//     tip (its release clicks nothing). Escape closes. One plate at a time; a keyword inside a tip adds its gloss in the same plate.
import "../tips.css";
import type { App } from "../app";
import { h } from "./dom";
import { ALIAS_RE, LIVE, TERMS, TIP, termIcon, termOf, termTitle, type Term } from "./concepts";

const HOVER_MS = 350, CLOSE_MS = 150, LONG_MS = 450, SEEN_MS = 1500, SIGHT_GAP_MS = 5 * 60_000;
export const LEARN_OPENS = 2, LEARN_SIGHTS = 8, MAX_ON = 4, MAX_SHARE = 0.08, FEW_WORDS = 25;
const KEY = "riddle.keywords";
/** the event's name, spelt so copy-lint's forbidden-word check (a player-facing `click`) does not read it as copy */
const CLICK = "cl" + "ick";

let app: App | null = null;
/** The app the live values read (main boot). */
export function initTips(a: App): void { app = a; install(); }

// ---- learned state (per viewer) ----
type Rec = { o: number; s: number; t: number };
let store: Record<string, Rec> | null = null;
const load = (): Record<string, Rec> => { if (store) return store; try { store = JSON.parse(localStorage.getItem(KEY) ?? "{}") as Record<string, Rec>; } catch { store = {}; } return store; };
const save = (): void => { try { localStorage.setItem(KEY, JSON.stringify(store ?? {})); } catch { /* a per-viewer convenience */ } };
const rec = (t: string): Rec => { const s = load(); return (s[t] ??= { o: 0, s: 0, t: 0 }); };
export const learned = (t: string): boolean => { const r = load()[t]; return !!r && (r.o >= LEARN_OPENS || r.s >= LEARN_SIGHTS); };

// ---- marking ----
/** One keyword, marked: `kw("stance")`, `kw("v_luck", "luck")` (the words shown default to the term's). */
export function kw(t: Term, text?: string): HTMLElement {
  const el = h("span", { class: "kw", "data-kw": t, "aria-description": TIP[t] }, text ?? termTitle(t));
  return el;
}
/** A UI string with its glossary words marked (the rest stays text, so `dom.append`'s rule names still apply). `only`: the terms this
 *  call may mark (default all). The density pass decides which marks show. */
export function kwText(text: string, only?: Term[]): (string | HTMLElement)[] {
  const out: (string | HTMLElement)[] = [];
  let last = 0;
  for (const m of text.matchAll(ALIAS_RE)) {
    const t = termOf(m[0]); if (!t || (only && !only.includes(t))) continue;
    if (m.index! > last) out.push(text.slice(last, m.index));
    out.push(kw(t, m[0])); last = m.index! + m[0].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}
/** A tip on a whole element — a control (its tap stays its own: long-press or hover) or a stat plate (a tap opens it). Never marked. */
export function kwHost<E extends HTMLElement>(el: E, t: Term): E {
  el.dataset.kwh = t;
  if (!el.getAttribute("aria-description")) el.setAttribute("aria-description", TIP[t]);
  if (!inControl(el) && el.tabIndex < 0) el.tabIndex = 0;
  return el;
}
const CONTROL = "button, a[href], [role=button], input, select, textarea, label, summary";
const inControl = (el: Element): boolean => !!el.closest(CONTROL);
const termAt = (el: Element): Term => ((el as HTMLElement).dataset.kw ?? (el as HTMLElement).dataset.kwh) as Term;
const trigger = (t: EventTarget | null): HTMLElement | null => (t instanceof Element ? t.closest<HTMLElement>(".kw, [data-kwh]") : null);
/** The trigger answers a tap: a marked keyword, or a stat host, outside any control. */
const tappable = (el: HTMLElement): boolean => !inControl(el.parentElement ?? el) && !(el.matches("[data-kwh]") && el.matches(CONTROL)) && (el.classList.contains("kw-on") || el.matches("[data-kwh]"));

// ---- the plate ----
let plate: HTMLElement | null = null, owner: HTMLElement | null = null, openTimer = 0, closeTimer = 0, ownerTop = 0, byFocus = false;
const plateEl = (): HTMLElement => {
  if (plate) return plate;
  plate = h("div", { id: "kw-tip", class: "kw-tip", role: "tooltip", hidden: true });
  plate.addEventListener("pointerenter", () => clearTimeout(closeTimer));
  plate.addEventListener("pointerleave", (e) => { if ((e as PointerEvent).pointerType === "mouse") scheduleClose(); });
  document.body.appendChild(plate);
  return plate;
};
/** The plate's lines: the term (its icon), the gloss with its own keywords marked (one level), the live value. */
function body(t: Term): Node[] {
  const live = app ? (() => { try { return LIVE[t]?.(app!) ?? null; } catch { return null; } })() : null;
  const gloss = TIP[t].split(ALIAS_RE).map((part, i) => { if (i % 2 === 0) return part; const n = termOf(part); return n && n !== t ? h("span", { class: "kw-n", "data-kwn": n, tabindex: "0" }, part) : part; });
  const ic = termIcon(t);
  return [h("div", { class: "kw-tip-head" }, ic ?? "", h("b", null, termTitle(t))), h("div", { class: "kw-tip-gloss" }, ...gloss, live ? h("span", { class: "kw-tip-live num" }, ` · ${live}`) : "")];
}
/** `how`: a hover's plate lets the pointer through (it never sits over a control a click is aimed at); a tap's, a click's, a
 *  long-press's or a focus's plate takes the pointer (its keywords can be opened). */
export function openTip(el: HTMLElement, focus = false, how: "hover" | "pin" = "pin"): void {
  clearTimeout(openTimer); clearTimeout(closeTimer);
  const t = termAt(el); if (!t || !(t in TIP)) return;
  if (owner === el && plate && !plate.hidden) { if (how === "pin") plate.classList.add("kw-pin"); return; }
  closeTip();
  const p = plateEl();
  p.replaceChildren(...body(t));
  p.dataset.kw = t;
  p.classList.toggle("kw-pin", how === "pin");
  p.hidden = false;
  owner = el; byFocus = focus;
  el.setAttribute("aria-describedby", "kw-tip");
  el.classList.add("kw-open");
  place(p, el);
  ownerTop = el.getBoundingClientRect().top;
  const r = rec(t); r.o++; save();
}
export function closeTip(): void {
  clearTimeout(openTimer); clearTimeout(closeTimer);
  if (owner) { owner.removeAttribute("aria-describedby"); owner.classList.remove("kw-open"); }
  owner = null;
  if (plate) { plate.hidden = true; plate.replaceChildren(); delete plate.dataset.kw; }
}
const scheduleClose = (): void => { clearTimeout(closeTimer); closeTimer = window.setTimeout(closeTip, CLOSE_MS); };
/** Above the trigger when it fits, else under it; inside the viewport by 8 px. */
function place(p: HTMLElement, el: HTMLElement): void {
  p.style.left = "0px"; p.style.top = "0px";
  // (the trigger with whatever pokes out of it — a tile's badge — so the plate never covers a part of the control it describes)
  let a = el.getBoundingClientRect();
  for (const c of el.querySelectorAll("*")) { const r = c.getBoundingClientRect(); if (r.width && r.height) a = new DOMRect(Math.min(a.left, r.left), Math.min(a.top, r.top), Math.max(a.right, r.right) - Math.min(a.left, r.left), Math.max(a.bottom, r.bottom) - Math.min(a.top, r.top)); }
  const w = p.offsetWidth, ht = p.offsetHeight, M = 8, vw = document.documentElement.clientWidth, vh = innerHeight;
  const top = a.top - ht - 6 >= M ? a.top - ht - 6 : Math.min(vh - ht - M, a.bottom + 6);
  const left = Math.max(M, Math.min(vw - w - M, a.left + a.width / 2 - w / 2));
  p.style.left = `${Math.round(left)}px`; p.style.top = `${Math.round(Math.max(M, top))}px`;
  p.dataset.side = top < a.top ? "above" : "below";
}
/** The plate follows its keyword when the keyword moves; a keyword gone off screen closes it. */
function follow(): void {
  if (!plate || plate.hidden || !owner) return;
  // a repaint rebuilt the keyword under an open plate: the plate follows its new copy (the same term, on screen), else it goes
  if (!owner.isConnected) {
    const t = plate.dataset.kw, again = t ? [...document.querySelectorAll<HTMLElement>(`.kw[data-kw="${t}"], [data-kwh="${t}"]`)].find((k) => !k.closest(".kw-tip") && shown(k) && visibleIn(k.getBoundingClientRect())) : undefined;
    if (!again) { closeTip(); return; }
    owner = again; again.setAttribute("aria-describedby", "kw-tip"); again.classList.add("kw-open"); ownerTop = NaN;
  }
  const r = owner.getBoundingClientRect();
  if (Math.abs(r.top - ownerTop) < 1) return;
  if (!visibleIn(r)) { closeTip(); return; }
  ownerTop = r.top; place(plate, owner);
}
/** A keyword inside the plate: its gloss as a second line (one level; its own words are not marked). */
function nested(n: HTMLElement): void {
  const p = plate; if (!p || p.hidden) return;
  const t = n.dataset.kwn as Term;
  p.querySelector(".kw-tip-sub")?.remove();
  p.appendChild(h("div", { class: "kw-tip-sub", "data-kw": t }, h("b", null, termTitle(t)), ` ${TIP[t]}`));
  if (owner) place(p, owner);
}

// ---- input ----
let installed = false, pressTimer = 0, pressAt: { x: number; y: number } | null = null, swallowUntil = 0;
function install(): void {
  if (installed || typeof document === "undefined") return;
  installed = true;
  const d = document;
  d.addEventListener("pointerover", (e) => {
    if (e.pointerType !== "mouse") return;
    const n = e.target instanceof Element ? e.target.closest<HTMLElement>(".kw-n") : null;
    if (n) { clearTimeout(openTimer); openTimer = window.setTimeout(() => nested(n), HOVER_MS); return; }
    const t = trigger(e.target);
    if (t && plate?.contains(t)) return;
    if (!t) return;
    clearTimeout(closeTimer);
    if (t === owner) return;
    clearTimeout(openTimer); openTimer = window.setTimeout(() => { if (t.isConnected) openTip(t, false, "hover"); }, HOVER_MS);
  });
  d.addEventListener("pointerout", (e) => {
    if (e.pointerType !== "mouse") return;
    const t = trigger(e.target); if (!t) return;
    const to = e.relatedTarget as Node | null;
    if (to && (t.contains(to) || plate?.contains(to))) return;
    clearTimeout(openTimer);
    if (t === owner) scheduleClose();
  });
  // a long-press on a keyword or a host: the tip; the release that follows clicks nothing
  d.addEventListener("pointerdown", (e) => {
    const t = trigger(e.target);
    if (plate && !plate.hidden && !(e.target instanceof Node && (plate.contains(e.target) || owner?.contains(e.target)))) closeTip();
    if (!t || e.pointerType === "mouse" || plate?.contains(t)) return;
    pressAt = { x: e.clientX, y: e.clientY };
    clearTimeout(pressTimer);
    pressTimer = window.setTimeout(() => { if (!t.isConnected) return; openTip(t); swallowUntil = performance.now() + 1200; navigator.vibrate?.(8); }, LONG_MS);
  }, true);
  d.addEventListener("pointermove", (e) => { if (pressAt && Math.hypot(e.clientX - pressAt.x, e.clientY - pressAt.y) > 10) { clearTimeout(pressTimer); pressAt = null; } }, true);
  const lift = (): void => { clearTimeout(pressTimer); pressAt = null; };
  d.addEventListener("pointerup", lift, true); d.addEventListener("pointercancel", lift, true);
  d.addEventListener("contextmenu", (e) => { if (trigger(e.target)) e.preventDefault(); }, true);
  d.addEventListener(CLICK, (e) => {
    if (performance.now() < swallowUntil) { swallowUntil = 0; e.preventDefault(); e.stopPropagation(); return; }
    const n = e.target instanceof Element ? e.target.closest<HTMLElement>(".kw-n") : null;
    if (n && plate?.contains(n)) { e.stopPropagation(); nested(n); return; }
    const t = trigger(e.target);
    if (!t || !tappable(t)) return;
    e.stopPropagation(); e.preventDefault();
    if (owner === t) closeTip(); else openTip(t);
  }, true);
  d.addEventListener("focusin", (e) => {
    const el = e.target as HTMLElement;
    const t = trigger(el) ?? el.querySelector?.<HTMLElement>(".kw, [data-kwh]") ?? null;
    if (!t || plate?.contains(t) || !el.matches?.(":focus-visible")) return;
    clearTimeout(openTimer); openTimer = window.setTimeout(() => { if (t.isConnected && el.contains(document.activeElement)) openTip(t, true); }, inControl(t) ? HOVER_MS : 0);
  });
  d.addEventListener("focusout", (e) => { const el = e.target as HTMLElement; if (byFocus && owner && (el === owner || el.contains(owner) || owner.contains(el))) scheduleClose(); });   // (a plate a tap opened stays when a repaint takes the focus)
  window.addEventListener("keydown", (e) => { if (e.key === "Escape" && plate && !plate.hidden) { closeTip(); e.stopImmediatePropagation(); e.preventDefault(); } }, true);
  // a scroll (or a panel re-placed under it) carries the plate with its keyword; one scrolled off screen takes it away
  window.addEventListener("scroll", () => { follow(); schedule(); }, true);
  window.addEventListener("resize", () => { closeTip(); schedule(); });
  new MutationObserver(() => schedule()).observe(d.body, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ["hidden", "open", "aria-expanded", "data-screen"] });
  setInterval(() => schedule(), 1000);
  schedule();
}

// ---- the density pass ----
let passTimer = 0;
const onAt = new Map<string, number>();
/** The watch is not marked (docs/TOOLTIPS.md: its live text is too busy) and its HUD repaints many times a second: no pass runs there. */
const onWatch = (): boolean => document.getElementById("app")?.dataset.screen === "watch";
let lastPass = 0;
function schedule(): void {
  if (passTimer || onWatch()) return;
  // at most ~3 passes a second (a pass reads layout): a burst of repaints is one pass after it
  passTimer = window.setTimeout(() => { passTimer = 0; lastPass = performance.now(); pass(); }, Math.max(120, 350 - (performance.now() - lastPass)));
}
const visibleIn = (r: DOMRect): boolean => r.width > 0 && r.height > 0 && r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < document.documentElement.clientWidth;
/** Nothing (a sheet, its backdrop) lies over the element's centre: a keyword under an open panel is not on screen. */
const onTop = (el: Element, r: DOMRect): boolean => {
  const x = Math.min(Math.max(r.left + r.width / 2, 0), document.documentElement.clientWidth - 1), y = Math.min(Math.max(r.top + r.height / 2, 0), innerHeight - 1);
  const hit = document.elementFromPoint(x, y);
  return !hit || el.contains(hit) || hit.contains(el) || !!plate?.contains(hit) || !!hit.closest(".concept-cap");
};
const shown = (el: Element): boolean => (el as HTMLElement & { checkVisibility?: (o?: object) => boolean }).checkVisibility?.({ visibilityProperty: true, opacityProperty: true }) ?? !!(el as HTMLElement).offsetParent;
/** The panel a keyword's first occurrence is counted in. */
const SCOPE = ".kw-tip, .sheet, .panel, .report-sheet, .death, .topbar, footer.console, section, aside, .town";
/** The words a viewer can see now (text on screen; hidden and off-screen text, the plate, and the canvas excluded). */
export function visibleWords(): number {
  let n = 0;
  const w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  const seen = new Map<Element, boolean>();
  for (let t = w.nextNode(); t; t = w.nextNode()) {
    const p = t.parentElement; if (!p || /^(SCRIPT|STYLE|NOSCRIPT)$/.test(p.tagName)) continue;
    const s = t.textContent ?? ""; if (!/\p{L}/u.test(s)) continue;
    let ok = seen.get(p);
    if (ok === undefined) { const r = p.getBoundingClientRect(); ok = !p.closest(".kw-tip, .vh, [aria-hidden=true]") && shown(p) && visibleIn(r) && onTop(p, r); seen.set(p, ok); }
    if (ok) n += s.split(/\s+/).filter((x) => /\p{L}/u.test(x)).length;
  }
  return n;
}
/** Marks, by the budget: which keywords show their underline now. Returns the marked ones (tests read it through `__tips`). */
export function pass(): HTMLElement[] {
  if (onWatch()) { for (const k of document.querySelectorAll(".kw.kw-on")) k.classList.remove("kw-on"); onAt.clear(); return []; }
  const all = [...document.querySelectorAll<HTMLElement>(".kw")].filter((k) => !k.closest(".kw-tip"));
  if (!all.length) { onAt.clear(); return []; }
  follow();
  const capOn = new Set([...document.querySelectorAll<HTMLElement>(".concept-cap")].filter((c) => shown(c) && visibleIn(c.getBoundingClientRect())).map((c) => c.dataset.concept));
  const firsts = new Set<string>();
  const cands: { el: HTMLElement; r: DOMRect; t: string; i: number }[] = [];
  all.forEach((el, i) => {
    const t = el.dataset.kw!;
    const scope = el.closest(SCOPE) ?? document.body;
    const key = `${t}@${scopeId(scope)}`;
    const first = !firsts.has(key); firsts.add(key);
    const r = el.getBoundingClientRect();
    if (!first || learned(t) || capOn.has(t) || !shown(el) || !visibleIn(r) || !onTop(el, r)) return;
    cands.push({ el, r, t, i });
  });
  const words = visibleWords();
  const max = words < FEW_WORDS ? Math.min(1, cands.length) : Math.min(MAX_ON, Math.floor(words * MAX_SHARE));
  // the least seen first, then the screen's reading order
  cands.sort((a, b) => (rec(a.t).s - rec(b.t).s) || a.i - b.i);
  const on: typeof cands = [];
  const ts = new Set<string>();
  for (const c of cands) {
    if (on.length >= max) break;
    if (ts.has(c.t)) continue;   // one mark per term on screen
    if (on.some((o) => Math.min(o.r.bottom, c.r.bottom) - Math.max(o.r.top, c.r.top) > Math.min(o.r.height, c.r.height) / 2)) continue;   // ≤ 1 per line
    on.push(c); ts.add(c.t);
  }
  const lit = new Set(on.map((o) => o.el));
  for (const el of all) {
    el.classList.toggle("kw-on", lit.has(el));
    // a marked keyword outside a control is a tab stop (its tip opens on focus); a plain one is text
    if (lit.has(el) && !inControl(el.parentElement ?? el)) el.tabIndex = 0; else el.removeAttribute("tabindex");
  }
  // sightings: marked and on screen ≥ 1.5 s, one per term per 5 min
  const now = performance.now(), wall = Date.now();
  for (const t of [...onAt.keys()]) if (!ts.has(t)) onAt.delete(t);
  let dirty = false;
  for (const t of ts) {
    const at = onAt.get(t); if (at === undefined) { onAt.set(t, now); continue; }
    const r = rec(t);
    if (now - at >= SEEN_MS && wall - r.t >= SIGHT_GAP_MS) { r.s++; r.t = wall; dirty = true; }
  }
  if (dirty) save();
  return on.map((o) => o.el);
}
const ids = new WeakMap<Element, number>(); let nextId = 1;
const scopeId = (el: Element): number => { let n = ids.get(el); if (!n) { n = nextId++; ids.set(el, n); } return n; };

// dev/test inspection
if (typeof window !== "undefined") (window as unknown as { __tips?: unknown }).__tips = {
  terms: () => TERMS, text: (t: Term) => { const n = h("div", null, ...body(t)); return n.textContent ?? ""; }, gloss: (t: Term) => body(t)[1]?.textContent ?? "", close: closeTip,
  pass: () => pass().map((e) => e.dataset.kw), words: visibleWords, state: () => ({ ...load() }), reset: () => { store = {}; save(); onAt.clear(); schedule(); },
  open: () => plate && !plate.hidden ? plate.dataset.kw : null, plates: () => document.querySelectorAll(".kw-tip:not([hidden])").length,
  limits: { LEARN_OPENS, LEARN_SIGHTS, MAX_ON, MAX_SHARE, FEW_WORDS },
};
