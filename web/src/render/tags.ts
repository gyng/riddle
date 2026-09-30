// Second art pass (art/ui/ART_GAP.md §8): a hostile's name tag is the target's small serif plate — a lowercase serif name over a
// short framed hp bar (thin red fill on a dark trough) — drawn in the DOM over the canvas (crisp at any k), placed by the viewer
// (index.ts lays the boxes out in world texels so no two intersect and none crosses the callout; `debugLabels` reports them).
// The layer is a sibling of the canvas with the canvas's offset box, so the viewer's CSS coordinates (`toCss`) are its own.
// Pooled elements; a frame writes only what changed. Cut 19: the serif is Alegreya (Google Fonts, index.html), Georgia before it loads.

export type Tag = { id: number; text: string; x: number; y: number; w: number; hp: number; ally?: boolean; boss?: boolean;
                    stair?: "taken" | "other" };   // Cut 26 §2: a fork floor's stair plate (its lane), the route's stair lit   // x centre, y bottom (CSS px); hp 0..1, <0 = no bar; ally: the green plate

/** QA e75ec29: an ally's plate — its kind's last word and its name (`jackal Skog`), a nameless (summoned) ally its kind alone. */
export function allyName(kind: string, name: string): string {
  const k = kind.replace(/^(spectral|boss)_/, "").split("_").pop() ?? kind;
  if (!name || name === k || name.replace(/ /g, "_") === kind || name.endsWith(k)) return /* copy:callout */ k;
  return /* copy:callout */ `${k} ${name}`;
}

/** gfx round 1 (the blind raters: "giant 'ATTACK NEAREST' pixel text over the sprites"; watch.png's `R2 ATTACK` plate): the callout
 *  over the hero and the fight frame's caption are a small iron plate with the serif caps and a notch pointing down at him — DOM,
 *  crisp at any k, placed by the viewer like the name tags. `CALL_*` are its CSS metrics (the viewer lays its box out from them). */
export type Plate = { kind: "callout" | "caption" | "boss"; text: string; x: number; y: number; w: number };   // x centre, y bottom (CSS px)
export const CALL_H = 24;        // CSS px: the plate (the notch hangs below it)
export const CALL_CHAR = 8.6;    // CSS px per character of the 12 px Cinzel caps
export const CALL_PAD = 20;      // CSS px of side padding
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
/* gfx round 18 (raters, every round: "a boss HP bar under his name plate"): a boss's plate carries a long framed bar with its notches */
.rtag.boss b { font-size: 14px; color: #ffd98a; letter-spacing: .06em; font-variant-caps: all-small-caps; }
.rtag.boss i { width: 84px; height: 8px; border: 1px solid #000; box-shadow: 0 0 0 1px #a8742e, 0 0 0 2px #1a0808, 0 0 10px rgba(220, 80, 30, .45); }
.rtag.boss i > s { background: linear-gradient(#ff6a4a, #c0261e 55%, #6a0e0a); box-shadow: inset 0 1px 0 rgba(255, 220, 190, .5); }
.rtag.ally b { color: #b9f0a4; }
.rtag.ally i { box-shadow: 0 0 0 1px rgba(90,150,70,.55); }
.rtag.ally i > s { background: linear-gradient(#6fd35a, #2f8a2a); }
.rtag.stair b { font-size: 11px; color: #cdb892; letter-spacing: .04em; }
.rtag.stair.taken b { color: #ffd76a; text-shadow: 0 0 6px rgba(255, 200, 90, .9), 0 1px 0 #000, 1px 0 0 #000, -1px 0 0 #000, 0 -1px 0 #000; }
.rtag.stair.other b { opacity: .7; }
.rcall { position: absolute; left: 0; top: 0; height: ${CALL_H}px; display: flex; align-items: center; justify-content: center; box-sizing: border-box;
  padding: 0 8px; white-space: nowrap; font: 700 12px/1 "Cinzel", "Trajan Pro", Georgia, serif; letter-spacing: .06em; text-transform: uppercase; color: #f4e3b8;
  background: linear-gradient(#2c241d, #15100c); border: 1px solid #7a5a32; border-radius: 3px;
  box-shadow: inset 0 1px 0 rgba(255, 220, 150, .18), inset 0 0 0 1px #0a0705, 0 2px 6px rgba(0, 0, 0, .75); text-shadow: 0 1px 0 #000, 0 0 6px rgba(255, 180, 80, .35); will-change: transform; }
.rcall::after { content: ""; position: absolute; left: 50%; bottom: -6px; width: 9px; height: 9px; margin-left: -5px; background: #15100c; border: solid #7a5a32; border-width: 0 1px 1px 0; transform: rotate(45deg); }
.rcall.caption::after { display: none; }
.rcall.caption { color: #ffd98a; }
.rcall.boss { height: 40px; font-size: 22px; letter-spacing: .12em; color: #ffe2a0; border-color: #b0822f; background: linear-gradient(#3a1512, #1a0808);
  box-shadow: inset 0 1px 0 rgba(255, 200, 150, .25), inset 0 0 0 1px #0a0404, 0 0 0 2px #1a0808, 0 0 22px rgba(200, 60, 30, .55), 0 4px 12px rgba(0, 0, 0, .8);
  text-shadow: 0 2px 0 #000, 0 0 12px rgba(255, 120, 40, .8); }
.rcall.boss::after { display: none; }
.rnum { position: absolute; left: 0; top: 0; pointer-events: none; white-space: nowrap; will-change: transform, opacity;
  font: 800 18px/1 "Barlow Condensed", "Arial Narrow", sans-serif; letter-spacing: .01em;
  text-shadow: 0 2px 0 #000, 2px 0 0 #000, -2px 0 0 #000, 0 -2px 0 #000, 1.5px 1.5px 0 #000, -1.5px 1.5px 0 #000, 1.5px -1.5px 0 #000, -1.5px -1.5px 0 #000, 0 0 8px rgba(0, 0, 0, .8); }
.rnum.big { font-size: 24px; }
.rshatter { position: absolute; left: 0; top: 0; width: 92px; height: 92px; pointer-events: none; }
.rshatter::before { content: ""; position: absolute; inset: -30%; border-radius: 50%; background: radial-gradient(circle, rgba(255, 230, 170, .85), rgba(255, 150, 60, .35) 40%, transparent 70%); animation: rsh-glow 1.1s ease-out both; }
@keyframes rsh-glow { 0% { opacity: 0; transform: scale(.4); } 15% { opacity: 1; transform: scale(1); } 100% { opacity: 0; transform: scale(1.5); } }
.rshatter { width: 120px; height: 120px; }
.rshatter.px { width: 104px; height: 104px; }
.rshatter.px i, .rshatter.px b { image-rendering: pixelated; filter: drop-shadow(0 2px 0 #000); }
.rshatter.px b { width: 42px; height: 42px; left: 27px; top: 27px; }
.rshatter b { position: absolute; left: 30%; top: 30%; width: 40%; height: 40%; background: center / contain no-repeat; filter: drop-shadow(0 2px 2px #000) drop-shadow(0 0 5px rgba(255, 200, 120, .7));
  animation: rsh-shard 1.3s cubic-bezier(.15, .7, .35, 1) .18s both; }
@keyframes rsh-shard { 0% { opacity: 0; transform: scale(.6); } 8% { opacity: 1; } 60% { opacity: 1; } 100% { opacity: 0; transform: translate(var(--dx), calc(var(--dy) + 40px)) rotate(var(--rot)) scale(.9); } }
.rstamp { position: absolute; left: 0; top: 0; pointer-events: none; white-space: nowrap; padding: 4px 14px 3px;
  font: 700 22px/1 "Cinzel", "Trajan Pro", Georgia, serif; letter-spacing: .14em; color: #ffe2a0; text-shadow: 0 2px 0 #000, 0 0 14px rgba(255, 150, 50, .9);
  background: linear-gradient(#3a1510, #1a0806); border: 2px solid #a8742e; box-shadow: inset 0 0 0 1px #000, 0 0 0 2px #1a0808, 0 0 26px rgba(220, 90, 30, .6), 0 6px 14px rgba(0, 0, 0, .8);
  animation: rstamp 1.6s cubic-bezier(.2, 1.4, .4, 1) both; }
.rstamp.slain { color: #fff0c8; border-color: #d8b060; background: linear-gradient(#4a3010, #1e1206); box-shadow: inset 0 0 0 1px #000, 0 0 0 2px #1a1008, 0 0 30px rgba(255, 200, 90, .7), 0 6px 14px rgba(0, 0, 0, .8); }
@keyframes rstamp { 0% { opacity: 0; scale: 2.2; rotate: -6deg; } 14% { opacity: 1; scale: .94; rotate: -3deg; } 22% { scale: 1; } 78% { opacity: 1; } 100% { opacity: 0; scale: 1.04; rotate: -3deg; } }
.rshatter i { position: absolute; inset: 0; background: center / 100% 100% no-repeat; filter: drop-shadow(0 0 6px rgba(255, 220, 160, .9)) drop-shadow(0 2px 2px #000); }
.rshatter i:first-of-type { clip-path: polygon(0 0, 58% 0, 44% 38%, 56% 62%, 40% 100%, 0 100%); animation: rsh-l 1.2s cubic-bezier(.2, .7, .4, 1) both; }
.rshatter i:last-of-type { clip-path: polygon(58% 0, 100% 0, 100% 100%, 40% 100%, 56% 62%, 44% 38%); animation: rsh-r 1.2s cubic-bezier(.2, .7, .4, 1) both; }
@keyframes rsh-l { 0% { transform: scale(1.6); opacity: 0; } 6% { transform: scale(1); opacity: 1; } 12% { transform: translate(-10px, -4px) rotate(-10deg); opacity: 1; } 55% { transform: translate(-50px, 30px) rotate(-55deg); opacity: 1; } 100% { transform: translate(-64px, 76px) rotate(-80deg); opacity: 0; } }
@keyframes rsh-r { 0% { transform: scale(1.6); opacity: 0; } 6% { transform: scale(1); opacity: 1; } 12% { transform: translate(10px, -4px) rotate(10deg); opacity: 1; } 55% { transform: translate(50px, 34px) rotate(60deg); opacity: 1; } 100% { transform: translate(64px, 82px) rotate(85deg); opacity: 0; } }   /* (round 14: split from 20 %, was 42 % — "the shield hangs static") */
/* gfx round 22 (raters: "the shield over the boss muddles him", "shards hard to parse"): the pixel shield sits clear above his head,
   flashes white-hot and cracks (a glowing seam) before its halves split in wide arcs, growing as they fly; the shards leave on the split */
.rshatter.px i { transform-origin: 50% 50%; }
.rshatter.px i:first-of-type { animation: rshp-l 1.45s linear both; }
.rshatter.px i:last-of-type { animation: rshp-r 1.45s linear both; }
@keyframes rshp-l { 0% { transform: scale(1.5); opacity: 0; filter: brightness(4) drop-shadow(0 2px 0 #000); } 6% { transform: scale(1); opacity: 1; } 10% { filter: brightness(3) drop-shadow(0 0 8px #fff3c0) drop-shadow(0 2px 0 #000); }
  14% { transform: translate(-2px, 1px); } 18% { transform: translate(1px, -1px); } 22% { transform: translate(0, 0); filter: brightness(1.15) drop-shadow(0 0 6px rgba(255, 210, 140, .9)) drop-shadow(0 2px 0 #000); animation-timing-function: linear; }
  34% { transform: translate(-14px, -14px) rotate(-10deg) scale(1.05); } 50% { transform: translate(-33px, -18px) rotate(-24deg) scale(1.1); }
  66% { transform: translate(-52px, -7px) rotate(-38deg) scale(1.15); } 82% { transform: translate(-71px, 20px) rotate(-52deg) scale(1.2); opacity: 1; }
  100% { transform: translate(-92px, 66px) rotate(-68deg) scale(1.22); opacity: 0; filter: brightness(1) drop-shadow(0 2px 0 #000); } }
@keyframes rshp-r { 0% { transform: scale(1.5); opacity: 0; filter: brightness(4) drop-shadow(0 2px 0 #000); } 6% { transform: scale(1); opacity: 1; } 10% { filter: brightness(3) drop-shadow(0 0 8px #fff3c0) drop-shadow(0 2px 0 #000); }
  14% { transform: translate(2px, -1px); } 18% { transform: translate(-1px, 1px); } 22% { transform: translate(0, 0); filter: brightness(1.15) drop-shadow(0 0 6px rgba(255, 210, 140, .9)) drop-shadow(0 2px 0 #000); }
  34% { transform: translate(15px, -15px) rotate(11deg) scale(1.05); } 50% { transform: translate(35px, -19px) rotate(26deg) scale(1.1); }
  66% { transform: translate(55px, -8px) rotate(41deg) scale(1.15); } 82% { transform: translate(75px, 21px) rotate(56deg) scale(1.2); opacity: 1; }
  100% { transform: translate(96px, 70px) rotate(72deg) scale(1.22); opacity: 0; filter: brightness(1) drop-shadow(0 2px 0 #000); } }
.rshatter.px s { position: absolute; left: 41%; top: -6%; width: 20%; height: 112%; background: #fffbe8;
  clip-path: polygon(50% 0, 80% 0, 45% 36%, 72% 38%, 30% 64%, 58% 66%, 22% 100%, 10% 100%, 38% 70%, 12% 68%, 52% 40%, 24% 38%);
  filter: drop-shadow(0 0 4px #fff) drop-shadow(0 0 10px #ffb040); transform-origin: 50% 0; animation: rshp-crack 1.45s linear both; }
@keyframes rshp-crack { 0% { opacity: 0; transform: scaleY(0); } 8% { opacity: 1; transform: scaleY(.2); } 16% { opacity: 1; transform: scaleY(1); } 24% { opacity: 1; } 32% { opacity: 0; transform: scaleY(1.1); } 100% { opacity: 0; } }
.rshatter.px b { animation-delay: .32s; }
.rshatter.px::before { animation-duration: 1.45s; }
@media (prefers-reduced-motion: reduce) { .rshatter i, .rshatter s { animation-duration: .01s !important; } }
@media (prefers-reduced-motion: no-preference) { html[data-juice="on"] .rcall.boss.fresh { animation: rboss-in 2.2s cubic-bezier(.2, 1.1, .3, 1) both; } }
@keyframes rboss-in { 0% { opacity: 0; scale: 1.8; letter-spacing: .5em; } 14% { opacity: 1; scale: 1; } 80% { opacity: 1; } 100% { opacity: 0; } }
@media (prefers-reduced-motion: no-preference) { html[data-juice="on"] .rcall.fresh { animation: rcall-in .22s cubic-bezier(.2, 1.5, .4, 1) both; } }
@keyframes rcall-in { from { opacity: 0; scale: .7; } to { opacity: 1; scale: 1; } }
`;

import skin from "../ui/skin.json";
/** gfx round 10: the painted boss shield and its shards (art/ui/fx, Codex) when packed; else the verb icon split in two */
const FX = new Set<string>((skin as { fx?: string[] }).fx ?? []);
// gfx round 21 (raters: "the painted shield clashes with the pixel sprites"): the pixel-register cut (tools/ui-skin.py `_px`), drawn pixelated
const PX = FX.has("shield_px");
const SHIELD = PX ? "/ui/fx/shield_px.png" : FX.has("shield") ? "/ui/fx/shield.webp" : "/ui/icons/v_shield.png";
const SHARDS = [0, 1, 2, 3].filter((i) => FX.has(`shard_${i}${PX ? "_px" : ""}`)).map((i) => `/ui/fx/shard_${i}${PX ? "_px.png" : ".webp"}`);
let shieldOk = false;
/** gfx round 22: how far (CSS px) the pixel shield's centre sits above the point the viewer passes (his head) */
export const SHIELD_LIFT = 84;
if (typeof Image !== "undefined") { const im = new Image(); im.onload = () => { shieldOk = true; }; im.src = SHIELD; for (const u of SHARDS) new Image().src = u; }

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
      const key = `${t.ally ? 1 : 0}${t.boss ? 1 : 0}|${t.stair ?? ""}|${t.text}|${Math.round(t.x)}|${Math.round(t.y)}|${Math.round(t.w)}|${hp.toFixed(2)}`;
      if (key === e.key) continue;
      e.key = key;
      e.el.style.display = "";
      if (e.name.textContent !== t.text) e.name.textContent = t.text;
      e.el.classList.toggle("nobar", hp < 0);
      e.el.classList.toggle("ally", !!t.ally); e.el.classList.toggle("boss", !!t.boss);
      e.el.classList.toggle("stair", !!t.stair); e.el.classList.toggle("taken", t.stair === "taken"); e.el.classList.toggle("other", t.stair === "other");
      if (hp >= 0) e.fill.style.width = `${Math.round(hp * 100)}%`;
      e.el.style.width = `${Math.round(t.w)}px`;
      e.el.style.transform = `translate(${Math.round(t.x - t.w / 2)}px, ${Math.round(t.y - TAG_H)}px)`;
    }
  }

  private calls: { el: HTMLDivElement; key: string; text: string }[] = [];
  /** gfx round 1: the callout / caption plates this frame (≤ 2). A plate whose text changes pops in (`fresh`). */
  plates(list: readonly (Plate | null)[]): void {   // fixed slots (boss · callout · caption): a slot's element keeps its plate
    const r = this.root; if (!r) return;
    this.place();
    while (this.calls.length < list.length) { const el = document.createElement("div"); el.className = "rcall"; el.style.display = "none"; r.appendChild(el); this.calls.push({ el, key: "", text: "" }); }
    for (let i = 0; i < this.calls.length; i++) {
      const c = this.calls[i]!, p = list[i];
      if (!p) { if (c.key !== "") { c.el.style.display = "none"; c.key = ""; } continue; }
      const key = `${p.kind}|${p.text}|${Math.round(p.x)}|${Math.round(p.y)}|${Math.round(p.w)}`;
      if (key === c.key) continue;
      c.key = key; c.el.style.display = "";
      if (c.text !== p.text) { c.text = p.text; c.el.textContent = p.text; c.el.classList.remove("fresh"); void c.el.offsetWidth; c.el.classList.add("fresh"); }
      c.el.classList.toggle("caption", p.kind === "caption"); c.el.classList.toggle("boss", p.kind === "boss");
      const H = p.kind === "boss" ? 40 : CALL_H;
      c.el.style.width = `${Math.round(p.w)}px`;
      c.el.style.transform = `translate(${Math.round(p.x - p.w / 2)}px, ${Math.round(p.y - H)}px)`;
    }
  }

  /** gfx round 1 (raters: "no shield-shatter", "no clear break"): a boss's guard breaks — his shield icon splits in two over him and
   *  falls away (CSS, 0.8 s). Only once the icon has loaded (art never blocks: no icon, no shatter). */
  shatter(x: number, y: number): void {
    const r = this.root; if (!r || !shieldOk) return;
    const el = document.createElement("div"); el.className = PX ? "rshatter px" : "rshatter";
    el.style.transform = `translate(${Math.round(x - (PX ? 52 : 60))}px, ${Math.round(y - (PX ? 52 + SHIELD_LIFT : 96))}px)`;   // (round 14: above his head, not over his face; round 22: clear of it)
    for (let i = 0; i < 2; i++) { const h = document.createElement("i"); h.style.backgroundImage = `url(${SHIELD})`; el.appendChild(h); }
    if (PX) el.appendChild(document.createElement("s"));   // gfx round 22: the crack's white-hot seam
    // gfx round 10 (raters: "burst the shield into big shards"): the painted shards fly out, spinning, and fall
    SHARDS.forEach((u, i) => { const b = document.createElement("b"); b.style.backgroundImage = `url(${u})`; b.style.setProperty("--dx", `${[-70, 64, -34, 44][i]}px`); b.style.setProperty("--dy", `${[-30, -44, 60, 38][i]}px`); b.style.setProperty("--rot", `${[-220, 260, -140, 190][i]}deg`); el.appendChild(b); });
    r.appendChild(el); setTimeout(() => el.remove(), 1600);
  }

  /** gfx round 10 (the coordinator approved `BROKEN` / `SLAIN`; raters: "a 'BROKEN' stamp", "a 'WARLORD SLAIN' plaque"): a word stamped
   *  over a boss at his break or his fall — slams in, holds, fades (CSS, 1.6 s) */
  stamp(text: string, x: number, y: number, kind: "broken" | "slain"): void {
    const r = this.root; if (!r) return;
    const el = document.createElement("div"); el.className = `rstamp ${kind}`; el.textContent = text;
    el.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px) translate(-50%, -50%)`;
    r.appendChild(el); setTimeout(() => el.remove(), 1700);
  }

  private nums: { el: HTMLDivElement; key: string }[] = [];
  /** gfx round 6: the damage numbers (CSS px: x centre, y baseline) — pooled, the game's face with a black outline, popping (sc). */
  numbers(list: readonly { x: number; y: number; text: string; col: readonly number[]; sc: number; a: number; big: boolean }[]): void {
    const r = this.root; if (!r) return;
    this.place();
    while (this.nums.length < list.length) { const el = document.createElement("div"); el.className = "rnum"; el.style.display = "none"; r.appendChild(el); this.nums.push({ el, key: "" }); }
    for (let i = 0; i < this.nums.length; i++) {
      const n = this.nums[i]!, p = list[i];
      if (!p) { if (n.key !== "") { n.el.style.display = "none"; n.key = ""; } continue; }
      const c = `rgb(${Math.round(p.col[0]! * 255)},${Math.round(p.col[1]! * 255)},${Math.round(p.col[2]! * 255)})`;
      const key = `${p.text}|${c}|${p.big ? 1 : 0}`;
      if (key !== n.key) { n.key = key; n.el.textContent = p.text; n.el.style.color = c; n.el.classList.toggle("big", p.big); n.el.style.display = ""; }
      n.el.style.opacity = p.a.toFixed(2);
      n.el.style.transform = `translate(${Math.round(p.x)}px, ${Math.round(p.y)}px) translate(-50%, -100%) scale(${(p.sc / 0.75).toFixed(2)})`;
    }
  }

  dispose(): void { this.root?.remove(); this.root = null; this.els = []; this.calls = []; this.nums = []; }
}
