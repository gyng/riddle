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
