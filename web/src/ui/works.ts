// Cut 30.5 — the works tree, client half (docs/CUT30_5.md, docs/AUTOMATION_TREE.md §3): the `next` pill on the home screen (one goal:
// an icon, ≤ 3 words, a number, a thin bar; a tap opens the works on its node), the works sheet (the tracks panel's successor: the
// nodes done, dimmed; the lit or counting node full with its hire; the next two as silhouettes; never the whole tree; the four branches
// once the trunk is done), the hire's beat (`PORTER HIRED`, ≤ 2 words) and the report's worker lines (`apprentice · +2 steps`).
// All truth is the core's (`Lineage.tree`): the client draws the nodes, their states and the pill as the wire sends them.
import "../works.css";
import type { App } from "../app";
import type { Lineage, ReturnReport, WorkNode, WorkerAct, Works } from "../engine/types";
import { h, replace } from "./dom";
import { closeEverything, openSheet } from "./sheet";
import { icon } from "./skin";
import { audio } from "../audio";
import { heroFace, trackIcon, trackName, TRACK_IDS } from "./tracks";
import { kw, kwHost } from "./tips";

export const hasWorks = (L: Pick<Lineage, "tree"> | undefined): boolean => !!L?.tree;
export const workerNodes = (W: Works): WorkNode[] => W.nodes.filter((n) => n.kind === "worker");
/** the trunk's workers (the first session's goal); the branches open once they are all hired */
export const TRUNK = ["quartermaster", "porter", "scout", "armourer", "apprentice"];
export const trunkDone = (W: Works): boolean => workerNodes(W).filter((n) => TRUNK.includes(n.id)).every((n) => n.state === "done");

/** A node's icon: `node_<id>` (art/town-ids.md), else the name's first letter. */
export const nodeIcon = (id: string, name = id): HTMLElement => icon(`node_${id}`, (name[0] ?? "?").toUpperCase());
/** What each worker retires, ≤ 4 words, a fragment (docs/AUTOMATION_TREE.md §5.4's register; eval/copy-budgets.json `node_tip`). The
 *  client's own words: the wire's `WorkNode.tip` is a sentence, so it is never drawn. */
/* copy:node_tip */
const BLURB: Record<string, string> = {
  quartermaster: "packs heal · drill item", porter: "hauls home · while away", scout: "sends him · each rest", armourer: "wears better finds",
  apprentice: "buys forge steps", keeper: "sorts finds · never asks", clerk: "banks spare gold", drillmaster: "levels the stance",
  kennel_hand: "fields best pets", herald: "swaps stale quests", guide: "starts deeper",
};
export const blurb = (id: string): string => BLURB[id] ?? "";
/** week 2: a worker's rank as a numeral (`II`); none at rank 1 */
export const rankNum = (r?: number): string => (r && r > 1 ? ["", "I", "II", "III", "IV", "V"][r] ?? String(r) : "");
/** What the next rank adds (≤ 4 words), once the core sends it (`rank_adds`, reserved); until then the price says it all. */
const rankAdds = (n: WorkNode): string => { const a = (n as WorkNode & { rank_adds?: string }).rank_adds; return a && a.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length <= 4 ? a : ""; };
/** the chore's count word (`2/3 chests`) */
/* copy:label */
const CHORE: Record<string, string> = { chest: "chests", send: "sends", wear: "worn", forge: "steps", keep: "sorted", deposit: "deposits", level: "levels", field: "fielded", swap: "swaps", start: "starts" };
export const choreWord = (c?: string): string => (c ? CHORE[c] ?? c : "");
const price = (n: WorkNode): string => (n.price ? `$${n.price}` : /* copy:label */ "free");
/** A node's one state line (≤ 3 words + a number): `done`, `2/3 chests`, `$120`, `bank built`. */
export function nodeState(n: WorkNode): string {
  if (n.state === "done") return n.paused ? /* copy:label */ "off" : /* copy:label */ "done";
  if (n.state === "lit") return n.need ? `${Math.min(n.count ?? 0, n.need)}/${n.need} ${choreWord(n.chore)}` : price(n);   // (the price is on the hire)
  if (n.state === "shut") return n.trigger ?? "";
  if (n.need) return `${n.count ?? 0}/${n.need} ${choreWord(n.chore)}`;
  return n.trigger ?? "";
}
/** A silhouette's line: what lights it and its price (`3 sends · $96`; shut: `bank built`). */
const silLine = (n: WorkNode): string => n.state === "shut" ? n.trigger ?? "" : [n.need ? `${n.count ? `${n.count}/` : ""}${n.need} ${choreWord(n.chore)}` : n.trigger ?? "", n.state === "ready" ? price(n) : n.price ? `$${n.price}` : /* copy:label */ "free"].filter(Boolean).join(" · ");

/** What the sheet draws of the trunk and the branches' workers: done (dimmed), the focus (lit, else the first counting), the next two. */
export function worksView(W: Works): { done: WorkNode[]; focus?: WorkNode; next: WorkNode[] } {
  const ws = workerNodes(W);
  const done = ws.filter((n) => n.state === "done");
  const pending = ws.filter((n) => n.state !== "done");
  const focus = pending.find((n) => n.id === W.lit) ?? pending.find((n) => n.state === "open") ?? pending[0];
  return { done, focus, next: pending.filter((n) => n !== focus).slice(0, 2) };
}

// --- the `next` pill ---

export type Pill = { el: HTMLElement; paint(): void };
/** The `next` pill (docs/AUTOMATION_TREE.md §3A): `kind` buy glows gold; a bar under it when the wire gives `have/need`. */
export function nextPill(app: App, anchorOpen: (node?: string, at?: HTMLElement) => void): Pill {
  const el: HTMLButtonElement = h("button", { class: "next-pill", hidden: true, onclick: (e: Event) => { e.stopPropagation(); anchorOpen(app.lineage.tree?.next?.node, el); } });
  let last = "";
  const paint = (): void => {
    const W = app.lineage.tree;
    if (!W) { el.hidden = true; return; }
    const p = W.next ?? { kind: "none", text: "" };
    const text = p.text || /* copy:label */ "works";
    const node = W.nodes.find((n) => n.id === p.node);
    const frac = p.kind === "buy" || p.kind === "chest" ? 1 : p.have !== undefined && p.need ? Math.max(0, Math.min(1, p.have / p.need)) : undefined;
    const key = JSON.stringify([p, text]);
    el.hidden = false; el.dataset.kind = p.kind; el.dataset.node = p.node ?? "";
    el.classList.toggle("gold", p.kind === "buy");
    if (key === last) return;
    const moved = !!last; last = key;
    // (the send's goal wears the gem's mark, the chest's the coin; a node its worker's face)
    const ico = p.kind === "send" ? icon("v_descend", "▼") : p.kind === "chest" ? icon("gold", "$") : node ? nodeIcon(node.id, node.name) : icon("gold", "▸");
    replace(el, h("span", { class: "pill-ico" }, ico),
      h("span", { class: "pill-main" }, h("span", { class: "pill-t num" }, text),
        frac !== undefined ? h("span", { class: "pill-bar", "aria-hidden": "true" }, h("span", { class: "fill", style: `width:${Math.round(frac * 100)}%` })) : ""));
    el.setAttribute("aria-label", text);
    if (moved) { el.classList.remove("bump"); void el.offsetWidth; el.classList.add("bump"); }
  };
  paint();
  return { el, paint };
}

// --- the works sheet ---

/** Opens the works on `focus` (a node id; the pill's), anchored to `anchor`. */
export function openWorks(app: App, focus?: string, anchor?: HTMLElement | null): void {
  if (!app.lineage.tree) return;
  closeEverything();
  openSheet((close) => {
    const body = h("div", { class: "sheet-body works-sheet" });
    const paint = (): void => {
      const W = app.lineage.tree; if (!W) { close(); return; }
      const v = worksView(W);
      const card = (n: WorkNode): HTMLElement => {
        const lit = n.state === "lit";
        const frac = n.need ? Math.min(1, (n.count ?? 0) / n.need) : lit ? 1 : 0;
        const hire = lit ? h("button", { class: `btn primary hire-btn${n.affordable === false ? " short" : ""}`, "data-node": n.id, disabled: n.affordable === false || !app.engine.hire,
          onclick: () => void hireNode(app, n, close) }, n.affordable === false ? `$${app.lineage.gold + W.chest}/$${n.price}` : /* copy:button */ `hire · ${price(n)}`) : "";
        return h("div", { class: `wnode cur${lit ? " lit" : ""}`, "data-node": n.id, "data-state": n.state },
          h("span", { class: "wn-ico" }, nodeIcon(n.id, n.name)),
          h("div", { class: "wn-main" },
            h("div", { class: "wn-head" }, h("b", { class: "wn-name" }, n.name), h("span", { class: "wn-state num" }, nodeState(n))),
            n.need || lit ? h("span", { class: "wn-bar", "aria-hidden": "true" }, h("span", { class: "fill", style: `width:${Math.round(frac * 100)}%` })) : "",
            blurb(n.id) ? h("small", { class: "wn-tip" }, blurb(n.id)) : ""),
          hire);
      };
      const sil = (n: WorkNode): HTMLElement => h("div", { class: "wnode sil", "data-node": n.id, "data-state": n.state },
        h("span", { class: "wn-ico" }, nodeIcon(n.id, n.name)), h("div", { class: "wn-main" }, h("span", { class: "wn-name" }, n.name), h("small", { class: "wn-state num" }, silLine(n))));
      const done = v.done.length ? h("div", { class: "works-done" }, h("span", { class: "wd-mark", "aria-hidden": "true" }, "✓"),
        ...v.done.map((n) => h("span", { class: `wd${n.paused ? " off" : ""}`, "data-node": n.id, "data-rank": n.rank ?? 1, title: blurb(n.id) || n.name }, nodeIcon(n.id, n.name), h("small", null, n.name),
          rankNum(n.rank) ? h("b", { class: "wd-rank num" }, ` ${rankNum(n.rank)}`) : ""))) : "";
      // week 2: the one rank on offer (`Works.lit_rank`, only while no hire is lit) — its card and its promote, above the done row
      const rk = W.lit_rank ? W.nodes.find((n) => n.id === W.lit_rank) : undefined;
      const rankCard = rk ? (() => {
        const to = rankNum((rk.rank ?? 1) + 1), price = rk.rank_price ?? 0, have = app.lineage.gold + W.chest, can = have >= price && !!app.engine.promote;
        return h("div", { class: "wnode cur lit rank", "data-node": rk.id, "data-state": "rank" },
          h("span", { class: "wn-ico" }, nodeIcon(rk.id, rk.name)),
          h("div", { class: "wn-main" }, h("div", { class: "wn-head" }, h("b", { class: "wn-name" }, `${rk.name} ${to}`), rankAdds(rk) ? h("span", { class: "wn-state num" }, rankAdds(rk)) : ""),
            h("small", { class: "wn-tip" }, kw("rank", /* copy:label */ "rank"), ` ${rankNum(rk.rank) || "I"} → ${to}`)),
          h("button", { class: `btn primary promote-btn${can ? "" : " short"}`, "data-node": rk.id, disabled: !can, onclick: () => void promoteNode(app, rk, close) }, can ? /* copy:button */ `promote · $${price}` : `$${have}/$${price}`));
      })() : "";
      // the four branches (the tracks: their stage, the next and its trigger) once the trunk is done
      const branches = trunkDone(W) ? h("div", { class: "works-branches" }, ...TRACK_IDS.map((t) => {
        const st = W.nodes.filter((n) => n.kind === "stage" && n.branch === t);
        const now = [...st].reverse().find((n) => n.state === "done"), nx = st.find((n) => n.state === "next");
        if (!now && !nx) return "";
        return h("div", { class: "wbranch", "data-branch": t }, h("span", { class: "wb-ico" }, trackIcon(t, heroFace(app.lineage))),
          h("small", { class: "wb-name" }, trackName(t)), h("b", { class: "wb-now" }, now?.name ?? ""),
          nx ? h("small", { class: "wb-next num" }, /* copy:callout */ `next · ${nx.name}${nx.trigger ? ` · ${nx.trigger}` : ""}`) : h("small", { class: "wb-next dim" }, "✓"));
      })) : "";
      replace(body, h("div", { class: "label row-label" }, kw("works", /* copy:label */ "works")),
        // (a rank on offer takes the second silhouette's place: the sheet stays ≤ 7 nodes on a phone)
        h("div", { class: "works-trunk" }, ...v.next.slice(0, rankCard ? 1 : 2).reverse().map(sil), v.focus ? card(v.focus) : "", rankCard, done), branches);
      const f = focus && body.querySelector<HTMLElement>(`.wnode[data-node="${focus}"]`);
      if (f) { f.classList.add("focus"); setTimeout(() => f.scrollIntoView?.({ block: "nearest" }), 0); }
    };
    paint();
    const off = app.onChange(() => { if (body.isConnected) paint(); else off(); });
    return body;
  }, { anchor });
}

/** The hire: the core's (`hire(id)`), then the beat — `PORTER HIRED` — while the worker walks to his post (the town's). */
export async function hireNode(app: App, n: WorkNode, close?: () => void): Promise<boolean> {
  if (!app.engine.hire) return false;
  const ok = await app.mutate(() => app.engine.hire!(n.id));
  if (!ok) return false;
  audio.cue("unlock");
  close?.();
  hireBeat(n.name);
  return true;
}
/** week 2: the promotion — the core's (`promote(id)`), then the beat `PORTER II`. */
export async function promoteNode(app: App, n: WorkNode, close?: () => void): Promise<boolean> {
  if (!app.engine.promote) return false;
  const ok = await app.mutate(() => app.engine.promote!(n.id));
  if (!ok) return false;
  audio.cue("level");
  close?.();
  const r = app.lineage.tree?.nodes.find((x) => x.id === n.id)?.rank ?? (n.rank ?? 1) + 1;
  beat(`${n.name.toUpperCase()} ${rankNum(r)}`);
  return true;
}
/** The hire's beat over the well: the worker's name and `HIRED` (2 words), ~1.8 s. */
export const hireBeat = (name: string): void => beat(`${name.toUpperCase()} `, /* copy:callout */ "HIRED");
function beat(...text: string[]): void {
  const host = document.querySelector<HTMLElement>(".camp .well-wrap") ?? document.body;
  for (const old of host.querySelectorAll(".works-beat")) old.remove();
  const b = h("div", { class: "works-beat", role: "status" }, h("b", null, ...text));
  host.appendChild(b);
  setTimeout(() => b.remove(), 1900);
}

// --- the send before the scout ---

/** The gem's mark (camp): before the scout the send counter toward him (`1/3`); after, `auto` — the hero goes down after each rest. */
export function sendMark(L: Lineage): HTMLElement | "" {
  const W = L.tree; if (!W) return "";
  if (W.auto_send) return kwHost(h("small", { class: "send-auto", "data-auto": "1" }, /* copy:label */ "auto"), "scout");
  const s = W.nodes.find((n) => n.id === "scout");
  return s?.need && s.state !== "done" ? kwHost(h("small", { class: "send-count num", "data-n": s.count ?? 0 }, `${Math.min(s.count ?? 0, s.need)}/${s.need}`), "scout") : "";
}

// --- the report ---

/** the worker's name (the node's; the id's words when the tree is not at hand) */
const nameOf = (L: Lineage, id: string): string => L.tree?.nodes.find((n) => n.id === id)?.name ?? id.replace(/_/g, "-");
/** An absence's worker lines, compact (`apprentice · +2 steps`), and the haul left in the chest (`chest +$120`); null when none. */
export function workersBlock(L: Lineage, r: Pick<ReturnReport, "workers" | "chest">): HTMLElement | null {
  const acts = (r.workers ?? []).filter((a) => a.n > 0 || a.what);
  if (!acts.length && !r.chest) return null;
  return h("div", { class: "works-acts" },
    ...acts.slice(0, 4).map((a) => h("span", { class: `chip work-act${a.first ? " first" : ""}`, "data-worker": a.id }, h("span", { class: "wa-ico" }, nodeIcon(a.id, nameOf(L, a.id))), h("b", null, nameOf(L, a.id)), h("span", { class: "num" }, ` · ${a.what}`))),
    acts.length > 4 ? h("small", { class: "dim" }, /* copy:callout */ `+${acts.length - 4} more`) : "",
    r.chest ? h("span", { class: "chip work-act chest", "data-worker": "chest" }, h("span", { class: "num gold" }, /* copy:callout */ `chest +$${r.chest}`)) : "");
}
/** Slices of one absence merge: each worker's acts summed (its line's number is the sum), `first` if any slice's was. */
export function mergeWorkers(a: WorkerAct[] | undefined, b: WorkerAct[] | undefined): WorkerAct[] | undefined {
  if (!a?.length) return b; if (!b?.length) return a;
  const out = a.map((x) => ({ ...x }));
  for (const y of b) {
    const x = out.find((z) => z.id === y.id);
    if (!x) { out.push({ ...y }); continue; }
    const n = x.n + y.n;
    x.what = /\d/.test(y.what) ? y.what.replace(/\d+/, String(n)).replace(/\b(step|level)$/, n > 1 ? "$1s" : "$1") : y.what; x.n = n; x.first = x.first || y.first;
  }
  return out;
}
