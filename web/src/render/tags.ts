// Second art pass (art/ui/ART_GAP.md §8): a hostile's name tag is the target's small serif plate — a lowercase serif name over a
// short framed hp bar (thin red fill on a dark trough) — drawn in the DOM over the canvas (crisp at any k), placed by the viewer
// (index.ts lays the boxes out in world texels so no two intersect and none crosses the callout; `debugLabels` reports them).
// The layer is a sibling of the canvas with the canvas's offset box, so the viewer's CSS coordinates (`toCss`) are its own.
// Pooled elements; a frame writes only what changed.

export type Tag = { id: number; text: string; x: number; y: number; w: number; hp: number };   // x centre, y bottom (CSS px); hp 0..1, <0 = no bar

export const TAG_H = 19;          // CSS px: the name's line (13) + the bar (4) + gaps
export const TAG_CHAR = 6.4;      // CSS px per character of the 12 px serif (lowercase average; the plate's width estimate)
export const TAG_PAD = 8;         // CSS px of side padding inside the plate

const CSS = `
.rtags { position: absolute; pointer-events: none; overflow: hidden; }
.rtag { position: absolute; left: 0; top: 0; height: ${TAG_H}px; display: flex; flex-direction: column; align-items: center; justify-content: flex-end; will-change: transform; }
.rtag b { font: 600 12px/13px "Alegreya", "Cormorant Garamond", Georgia, "Times New Roman", serif; color: #f1e6cf; white-space: nowrap; letter-spacing: .01em;
  text-shadow: 0 1px 0 #000, 1px 0 0 #000, -1px 0 0 #000, 0 -1px 0 #000, 0 1px 3px rgba(0,0,0,.8); }
.rtag i { display: block; width: 24px; height: 4px; margin-top: 2px; background: #1a0f0c; border: 1px solid #0b0706; box-shadow: 0 0 0 1px rgba(120,90,60,.45); box-sizing: border-box; }
.rtag i > s { display: block; height: 100%; background: linear-gradient(#e0433c, #9e1f1c); text-decoration: none; }
.rtag.nobar i { display: none; }
`;

export class TagLayer {
  private root: HTMLDivElement | null = null;
  private els: { el: HTMLDivElement; name: HTMLElement; fill: HTMLElement; key: string }[] = [];
  private box = "";
  private canvas: HTMLCanvasElement;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const parent = canvas.parentElement;
    if (!parent || typeof document === "undefined") return;
    if (!document.getElementById("rtags-css")) {
      const s = document.createElement("style"); s.id = "rtags-css"; s.textContent = CSS; document.head.appendChild(s);
    }
    this.root = document.createElement("div");
    this.root.className = "rtags";
    parent.insertBefore(this.root, canvas.nextSibling);
  }

  /** keep the layer on the canvas's box (both are positioned against the same offsetParent) */
  private place(): void {
    const c = this.canvas, r = this.root!;
    const key = `${c.offsetLeft},${c.offsetTop},${c.clientWidth},${c.clientHeight}`;
    if (key === this.box) return;
    this.box = key;
    Object.assign(r.style, { left: `${c.offsetLeft}px`, top: `${c.offsetTop}px`, width: `${c.clientWidth}px`, height: `${c.clientHeight}px` });
  }

  sync(tags: readonly Tag[]): void {
    const r = this.root; if (!r) return;
    this.place();
    while (this.els.length < tags.length) {
      const el = document.createElement("div"); el.className = "rtag";
      const name = document.createElement("b"), bar = document.createElement("i"), fill = document.createElement("s");
      bar.appendChild(fill); el.append(name, bar); r.appendChild(el);
      this.els.push({ el, name, fill, key: "" });
    }
    for (let i = 0; i < this.els.length; i++) {
      const e = this.els[i]!, t = tags[i];
      if (!t) { if (e.key !== "") { e.el.style.display = "none"; e.key = ""; } continue; }
      const hp = t.hp < 0 ? -1 : Math.max(0, Math.min(1, t.hp));
      const key = `${t.text}|${Math.round(t.x)}|${Math.round(t.y)}|${Math.round(t.w)}|${hp.toFixed(2)}`;
      if (key === e.key) continue;
      e.key = key;
      e.el.style.display = "";
      if (e.name.textContent !== t.text) e.name.textContent = t.text;
      e.el.classList.toggle("nobar", hp < 0);
      if (hp >= 0) e.fill.style.width = `${Math.round(hp * 100)}%`;
      e.el.style.width = `${Math.round(t.w)}px`;
      e.el.style.transform = `translate(${Math.round(t.x - t.w / 2)}px, ${Math.round(t.y - TAG_H)}px)`;
    }
  }

  dispose(): void { this.root?.remove(); this.root = null; this.els = []; }
}
