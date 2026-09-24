// Tiny DOM helpers. No framework.
type Child = Node | string | number | null | undefined | false;
type Attrs = Record<string, string | number | boolean | ((e: Event) => void) | undefined>;

export function h<K extends keyof HTMLElementTagNameMap>(tag: K, attrs?: Attrs | null, ...children: Child[]): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  if (attrs) for (const [k, v] of Object.entries(attrs)) {
    if (v === undefined || v === false) continue;
    if (k.startsWith("on") && typeof v === "function") el.addEventListener(k.slice(2), v);
    else if (k === "class") el.className = String(v);
    else if (v === true) el.setAttribute(k, "");
    else el.setAttribute(k, String(v));
  }
  append(el, children);
  return el;
}
export function append(el: Node, children: Child[]): void {
  for (const c of children) {
    if (c === null || c === undefined || c === false) continue;
    el.appendChild(typeof c === "string" || typeof c === "number" ? document.createTextNode(String(c)) : c);
  }
}
export function clear(el: Element): void { while (el.firstChild) el.removeChild(el.firstChild); }
export function replace(el: Element, ...children: Child[]): void { clear(el); append(el, children); }
export const pct = (x: number): string => `${Math.round(x * 100)}%`;
export function flash(el: HTMLElement, cls = "hl", ms = 1600): void { el.classList.add(cls); setTimeout(() => el.classList.remove(cls), ms); }
export async function copyText(text: string): Promise<boolean> {
  try { await navigator.clipboard.writeText(text); return true; } catch { return false; }
}
/** `1 item` / `4 items` (tagged at the call site). */
export const items = (n: number): string => `${n} item${n === 1 ? "" : "s"}`;
/** Seconds → `12m` (whole minutes up); under a minute `40s`. */
export const spanOf = (s: number): string => (s >= 60 ? `${Math.ceil(s / 60)}m` : `${Math.max(0, Math.round(s))}s`);
/** QA e75ec29 (R: the `$50` hatch and the `$75` insure charged on one tap): a paid chip takes two taps — the first arms it (its label
 *  becomes `ok $50`, `.armed`), a second within 3 s pays; untouched, it disarms. */
export function twoTap(label: string, armedLabel: string, act: () => void, attrs: { class?: string; disabled?: boolean } = {}): HTMLButtonElement {
  let armed = 0;
  const b: HTMLButtonElement = h("button", { class: attrs.class ?? "chip mini", disabled: attrs.disabled, "aria-label": label, onclick: (e: Event) => {
    e.stopPropagation();
    if (armed && performance.now() - armed < 3000) { armed = 0; act(); return; }
    armed = performance.now(); b.classList.add("armed"); replace(b, armedLabel); b.setAttribute("aria-label", armedLabel);
    setTimeout(() => { if (b.isConnected && armed) { armed = 0; b.classList.remove("armed"); replace(b, label); b.setAttribute("aria-label", label); } }, 3000);
  } }, label);
  return b;
}
