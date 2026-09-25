// Juice (docs/JUICE.md) — UI motion switch and the purse's count-up. `html[data-juice="on"]` gates every rule in juice.css (the
// CSS also honours `prefers-reduced-motion`). Off under reduced motion, and in automation on a software GL (the headless client
// gates: their geometry reads and clicks never meet a moving element); `?juice=1|0` pins it. Nothing here writes game text: the
// purse's count-up is drawn by a pseudo-element over the real `$N` (transparent while it counts), so innerText is always the truth.
import "./juice.css";
import { softwareGl, reducedMotion } from "./render/quality";

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
