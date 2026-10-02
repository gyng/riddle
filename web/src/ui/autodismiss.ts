// Auto-continue (docs/UI.md §7; the owner: "for idle, buttons need a timer to auto close"): a screen or panel that waits for a tap
// presses its own default after a while — the report's gem, the death's default, a sheet's close stud — with the countdown drawn as a
// thin gilt ring draining around the control it will press. Timing is UI only: no game truth here.
//   autoDismiss(el, { ms, onExpire?, scope?, yieldToSheets? })   onExpire defaults to `el.click()` (what the control already shows)
// Input restarts every clock; a press held, a pointer resting on the control or panel, an open tooltip, a busy engine (and for a
// screen, any sheet over it) pause it; a hidden tab pauses it and the return starts it over full. Settings: `auto continue`.
import "../autodismiss.css";

/** The defaults (docs/UI.md §7 says why). */
export const AUTO = { report: 12_000, death: 12_000, panel: 20_000, beat: 3_000 } as const;

const KEY = "riddle.autoContinue";
/** The settings toggle (on by default; a per-viewer preference). */
export function autoOn(): boolean { try { return localStorage.getItem(KEY) !== "0"; } catch { return true; } }
export function setAutoOn(on: boolean): void { try { localStorage.setItem(KEY, on ? "1" : "0"); } catch { /* private mode: this session only */ } mem = on; }
let mem: boolean | null = null;
/** `?autodismiss=0` off · `=1` on (also under automation) · `=0.1` on, clocks 10× fast. Off by default under automation, so the older
 *  client gates (which wait on screens) keep their meaning. */
function testScale(): number | null {
  try { const q = new URLSearchParams(location.search).get("autodismiss"); if (q === null) return null; const n = Number(q); return Number.isFinite(n) && n >= 0 ? n : null; } catch { return null; }
}
const scale = testScale();
const automated = typeof navigator !== "undefined" && !!navigator.webdriver;
/** Whether auto-continue runs now. */
export function autoEnabled(): boolean {
  if (scale === 0) return false;
  if (scale === null && automated) return false;
  return mem ?? autoOn();
}

type Timer = { el: HTMLElement; scope: HTMLElement; ms: number; left: number; onExpire: () => void; yieldToSheets: boolean; ring: SVGSVGElement; arc: SVGRectElement; num: SVGTextElement; w: number; h: number; done: boolean };
const live = new Set<Timer>();
let pressing = false, mouse = false, raf = 0, last = 0;
const SVG = "http://www.w3.org/2000/svg";
const reduced = (): boolean => { try { return matchMedia("(prefers-reduced-motion: reduce)").matches; } catch { return false; } };

/** Puts the countdown on `el` and presses it (or runs `onExpire`) when it runs out. Stops by itself once `el` leaves the page. */
export function autoDismiss(el: HTMLElement, o: { ms: number; onExpire?: () => void; scope?: HTMLElement; yieldToSheets?: boolean }): { stop(): void } {
  const ring = document.createElementNS(SVG, "svg");
  ring.setAttribute("class", "ad-ring"); ring.setAttribute("aria-hidden", "true");
  const track = document.createElementNS(SVG, "rect"); track.setAttribute("class", "ad-track");
  const arc = document.createElementNS(SVG, "rect"); arc.setAttribute("class", "ad-arc"); arc.setAttribute("pathLength", "100");
  const num = document.createElementNS(SVG, "text"); num.setAttribute("class", "ad-num num");
  ring.append(track, arc, num);
  el.classList.add("ad-host"); el.appendChild(ring);
  const ms = Math.max(1, o.ms * (scale && scale > 0 ? scale : 1));
  const t: Timer = { el, scope: o.scope ?? el, ms, left: ms, onExpire: o.onExpire ?? (() => el.click()), yieldToSheets: !!o.yieldToSheets, ring, arc, num, w: 0, h: 0, done: false };
  live.add(t); listen(); kick();
  return { stop: () => stop(t) };
}

function stop(t: Timer): void { if (t.done) return; t.done = true; live.delete(t); t.ring.remove(); t.el.classList.remove("ad-host", "ad-paused"); }

/** Fits the ring to the control's box and corners (a round gem, a stud, a squared tile). */
function fit(t: Timer, track: Element): void {
  const w = t.el.offsetWidth, h = t.el.offsetHeight; if (w === t.w && h === t.h) return;
  t.w = w; t.h = h;
  const cs = getComputedStyle(t.el).borderTopLeftRadius || "0";
  const px = (v: string, d: number): number => v.endsWith("%") ? (parseFloat(v) / 100) * d : parseFloat(v) || 0;
  const rx = Math.min(w / 2, px(cs, w)), ry = Math.min(h / 2, px(cs.split(" ")[1] ?? cs, h));
  t.ring.setAttribute("viewBox", `0 0 ${w} ${h}`);
  for (const r of [track, t.arc]) { r.setAttribute("x", "1.5"); r.setAttribute("y", "1.5"); r.setAttribute("width", String(Math.max(0, w - 3))); r.setAttribute("height", String(Math.max(0, h - 3))); r.setAttribute("rx", String(Math.max(0, rx - 1.5))); r.setAttribute("ry", String(Math.max(0, ry - 1.5))); }
  // the seconds (reduced motion): on the ring at its upper right — at 45° on a round control, in the corner of a square one
  const round = rx >= w / 2 - 2 && ry >= h / 2 - 2;
  t.num.setAttribute("x", String(round ? w / 2 + 0.707 * (w / 2) + 4 : w - 6)); t.num.setAttribute("y", String(round ? h / 2 - 0.707 * (h / 2) + 4 : 11));
}

function paused(t: Timer): boolean {
  if (document.hidden || pressing) return true;
  if (!autoEnabled()) return true;
  if (mouse && (t.el.matches(":hover") || t.scope.matches(":hover"))) return true;
  const tip = document.getElementById("kw-tip"); if (tip && !tip.hidden) return true;   // a keyword tooltip open (docs/TOOLTIPS.md)
  if (document.querySelector(".busy")) return true;
  if (t.yieldToSheets && document.querySelector(".sheet-wrap")) return true;
  if (!t.el.getClientRects().length) return true;   // not rendered (a sheet hidden under the one that replaced it)
  return false;
}

function frame(now: number): void {
  raf = 0;
  const dt = last ? Math.min(250, now - last) : 0; last = now;
  for (const t of [...live]) {
    if (!t.el.isConnected) { stop(t); continue; }
    const p = paused(t);
    t.el.classList.toggle("ad-paused", p);
    t.ring.style.display = autoEnabled() ? "" : "none";
    if (!autoEnabled()) { t.left = t.ms; continue; }
    if (!p) t.left -= dt;
    fit(t, t.ring.firstElementChild!);
    const still = reduced();
    t.arc.style.strokeDashoffset = still ? "0" : String(100 * (1 - Math.max(0, t.left) / t.ms));
    t.num.textContent = still ? String(Math.max(1, Math.ceil(t.left / 1000))) : "";
    if (t.left <= 0) { stop(t); try { t.onExpire(); } catch (e) { console.warn("auto-continue", e); } }
  }
  kick();
}
function kick(): void { if (!raf && live.size) raf = requestAnimationFrame(frame); if (!live.size) last = 0; }

/** Any input starts every clock over. */
function restart(): void { for (const t of live) t.left = t.ms; }
let listening = false;
function listen(): void {
  if (listening) return; listening = true;
  const opt = { capture: true, passive: true } as const;
  document.addEventListener("pointerdown", (e) => { pressing = true; mouse = e.pointerType === "mouse"; restart(); }, opt);
  document.addEventListener("pointerup", () => { pressing = false; restart(); }, opt);
  document.addEventListener("pointercancel", () => { pressing = false; }, opt);
  document.addEventListener("pointermove", (e) => { mouse = e.pointerType === "mouse"; if (mouse) restart(); }, opt);
  for (const ev of ["keydown", "wheel", "scroll", "touchstart"]) document.addEventListener(ev, restart, opt);
  // a hidden tab stands still; the return starts the clock over full (the player always gets to see the screen)
  document.addEventListener("visibilitychange", () => { last = 0; if (!document.hidden) restart(); });
}

/** dev/test inspection: each live clock's control and what is left. */
if (typeof window !== "undefined") Object.defineProperty(window, "__autodismiss", { configurable: true, value: {
  live: () => [...live].map((t) => ({ cls: t.el.className, text: t.el.textContent ?? "", left: Math.round(t.left), ms: t.ms, paused: paused(t) })),
  enabled: autoEnabled,
} });
