// Juice (docs/JUICE.md) — UI motion switch and the purse's count-up. `html[data-juice="on"]` gates every rule in juice.css (the
// CSS also honours `prefers-reduced-motion`). Off under reduced motion, and in automation on a software GL (the headless client
// gates: their geometry reads and clicks never meet a moving element); `?juice=1|0` pins it. Nothing here writes game text: the
// purse's count-up is drawn by a pseudo-element over the real `$N` (transparent while it counts), so innerText is always the truth.
import "./juice.css";
import { softwareGl, reducedMotion } from "./render/quality";
import { audio } from "./audio";

function decide(): boolean {
  try {
    const q = new URLSearchParams(location.search).get("juice");
    if (q === "1") return true;
    if (q === "0") return false;
  } catch { /* no location: default */ }
  if (reducedMotion()) return false;
  if (navigator.webdriver) {
    const c = document.createElement("canvas");
    const gl = c.getContext("webgl");
    const soft = softwareGl(gl);
    gl?.getExtension("WEBGL_lose_context")?.loseContext();
    if (soft) return false;
  }
  return true;
}

const on = decide();
document.documentElement.dataset.juice = on ? "on" : "off";

let lastGold: number | null = null;
function watchGold(): void {
  const seen = new WeakSet<Element>();
  const scan = (): void => {
    for (const el of document.querySelectorAll(".topbar .stat.gold")) {
      if (seen.has(el)) continue;
      seen.add(el);
      const tn = [...el.childNodes].find((n) => n.nodeType === 3 && /\$\d/.test(n.textContent ?? ""));
      if (!tn) continue;
      const v = Number((tn.textContent ?? "").replace(/[^\d]/g, ""));
      const prev = lastGold;
      lastGold = v;
      if (prev !== null && v < prev) audio.cue("buy");   // juice pass 2: the purse fell — something was bought (a coin's two notes)
      if (prev === null || prev === v || !on) continue;
      const span = document.createElement("span");
      span.className = "jn count";
      span.style.setProperty("--from", String(prev));
      span.style.setProperty("--to", String(v));
      tn.replaceWith(span);
      span.appendChild(tn);
      if (v > prev) el.classList.add("glint");
      setTimeout(() => { span.classList.remove("count"); el.classList.remove("glint"); }, 1000);
    }
  };
  new MutationObserver(scan).observe(document.body, { childList: true, subtree: true });
  scan();
}
if (typeof MutationObserver !== "undefined") {
  if (document.body) watchGold(); else addEventListener("DOMContentLoaded", watchGold, { once: true });
}

/** Juice pass 2: a sheet's close eases out. `ui/sheet.ts` removes the sheet at once (the DOM, the stack and every tool that reads them
 *  see it gone); a ghost of its panel — inert, aria-hidden, without its close studs — fades and drops in its place for SHEET_OUT_MS. */
const SHEET_OUT_MS = 150;
export function sheetGhost(wrap: HTMLElement): void {
  if (!on) return;
  const panel = wrap.querySelector<HTMLElement>(":scope > .sheet"); if (!panel || wrap.hidden) return;
  const r = panel.getBoundingClientRect(); if (r.width < 8 || r.height < 8) return;
  const ghost = document.createElement("div");
  ghost.className = "sheet-ghost"; ghost.setAttribute("aria-hidden", "true"); ghost.inert = true;
  ghost.style.cssText = `left:${r.left}px;top:${r.top}px;width:${r.width}px;height:${r.height}px;`;
  const copy = panel.cloneNode(true) as HTMLElement;
  copy.removeAttribute("role"); copy.style.maxHeight = `${r.height}px`;
  for (const x of copy.querySelectorAll(".close-stud, .sheet-x, .sheet-back")) x.remove();
  ghost.appendChild(copy);
  document.body.appendChild(ghost);
  setTimeout(() => ghost.remove(), SHEET_OUT_MS + 30);
}

/** Juice pass 2: the chrome's sounds, read off the DOM (no screen module calls them): a tap on a button clicks (a rule's chip taps
 *  wood), the verdict word lands with a stamp, the fold line and the divergence scene have theirs. */
function chromeSounds(): void {
  document.addEventListener("pointerdown", (e) => {
    const t = e.target instanceof Element ? e.target.closest<HTMLElement>("button, [role=button], .tile") : null;
    if (!t || (t as HTMLButtonElement).disabled) return;
    audio.cue(t.classList.contains("chip") ? "edit" : "click");
  }, true);
  const shown = new WeakSet<Element>();
  const onAttr = (el: Element): void => {
    if (el.classList.contains("fold-line")) {
      const vis = !(el as HTMLElement).hidden && !el.classList.contains("docked");
      if (vis && !shown.has(el)) { shown.add(el); audio.cue("fold"); } else if ((el as HTMLElement).hidden) shown.delete(el);
    } else if (el.classList.contains("div-scene")) {
      const d = (el as HTMLElement).dataset, key = `${d.state}:${d.phase}`;
      if (d.state === "playing" && (el as HTMLElement & { __jk?: string }).__jk !== key) audio.cue("scene");
      (el as HTMLElement & { __jk?: string }).__jk = key;
    } else if (el.classList.contains("div-end")) {
      if (el.classList.contains("show") && !shown.has(el)) { shown.add(el); audio.cue("scene_end", { up: el.classList.contains("up") }); }
      else if (!el.classList.contains("show")) shown.delete(el);
    }
  };
  new MutationObserver((ms) => {
    for (const m of ms) {
      if (m.type === "attributes") { onAttr(m.target as Element); continue; }
      for (const n of m.addedNodes) {
        if (!(n instanceof Element)) continue;
        const cause = n.matches(".death .banner-cloth .cause") ? n : n.querySelector(".banner-cloth .cause");
        if (cause && cause.closest(".death")) setTimeout(() => audio.cue("verdict"), on ? 330 : 0);   // with the slam (juice.css j-slam's delay)
        const fl = n.matches(".fold-line") ? n : null; if (fl) onAttr(fl);
      }
    }
  }).observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ["hidden", "class", "data-state", "data-phase"] });
}
if (typeof MutationObserver !== "undefined") {
  if (document.body) chromeSounds(); else addEventListener("DOMContentLoaded", chromeSounds, { once: true });
}

/** gfx round 7 (the report's strip was a dim, empty parchment for ~0.6 s every round): a new screen's first frame can take ~0.5 s to
 *  raster on the GPU, and the document timeline does not advance meanwhile — an arrival animation from opacity 0 is frozen invisible
 *  the whole time, then lands finished. A screen gets `.arrived` two frames after it mounts (its first frame presented); the report's
 *  plaques draw at rest before that and stamp after it (juice.css). */
function arrivals(): void {
  const app = document.getElementById("app"); if (!app) return;
  const mark = (el: Element): void => { if (el.tagName === "MAIN" && !el.classList.contains("arrived")) requestAnimationFrame(() => requestAnimationFrame(() => el.classList.add("arrived"))); };
  new MutationObserver((ms) => { for (const m of ms) for (const n of m.addedNodes) if (n instanceof Element) mark(n); }).observe(app, { childList: true });
  for (const el of app.children) mark(el);
  // gfx round 7 (raters Q, R, the camp: "four frames identical; the braziers don't even flicker"): live flames in the vista's braziers —
  // two decorative tongues each (juice.css `.j-flame`, transforms only), added wherever a vista mounts
  const flames = (v: Element): void => {
    if (v.querySelector(":scope > .j-flame")) return;
    for (const cls of ["j-glow l", "j-glow r", "j-flame l", "j-flame r"]) { const f = document.createElement("i"); f.className = cls; f.setAttribute("aria-hidden", "true"); v.appendChild(f); }
  };
  new MutationObserver((ms) => {
    for (const m of ms) for (const n of m.addedNodes) {
      if (!(n instanceof Element)) continue;
      if (n.classList.contains("vista")) flames(n); else if (n.firstElementChild) for (const v of n.querySelectorAll(".vista")) flames(v);
    }
  }).observe(app, { childList: true, subtree: true });
  for (const v of app.querySelectorAll(".vista")) flames(v);
  // gfx round 10 (the coordinator: the `.arrived` pattern on every opening — the oath board's strip was "only the camp dimming"): a sheet
  // is drawn at rest in its first frame and settles once that frame is presented (juice.css)
  const sheet = (n: Node): void => { if (n instanceof HTMLElement && n.classList.contains("sheet-wrap")) requestAnimationFrame(() => requestAnimationFrame(() => n.classList.add("arrived"))); };
  new MutationObserver((ms) => { for (const m of ms) m.addedNodes.forEach(sheet); }).observe(document.body, { childList: true });
}
if (typeof MutationObserver !== "undefined" && on) {
  if (document.getElementById("app")) arrivals(); else addEventListener("DOMContentLoaded", arrivals, { once: true });
}
