// Cut 118 §7 (research/IDLE_STEAM_2026-10.md §8, Scapewatch): the control earned, read as progress — one rail of rungs, weights →
// orders → (heirs, owner amendment 2) → workers → pen (the packages worn, the run setup, the hired hands, the rule pen). A rung is lit once the lineage has it; the
// first rung still dark names what opens it. Quiet: four words and one condition; the lit orders rung is the orders sheet's door (Cut 120
// §8), the rest no taps (each system has its own door). Hidden on a
// fresh lineage, on a literal (harness) lineage and once every rung is lit. All truth is the core's: the systems' triggers, the tree.
import "../ladder.css";
import type { App } from "../app";
import type { Lineage } from "../engine/types";
import { h, replace } from "./dom";
import { revealed } from "./reveal";
import { onPackages, packagesShown, penOpen } from "./packages";
import { choreWord, nodeState, workerNodes, worksView } from "./works";
import { detailHost } from "./tips";

export type RungId = "weights" | "orders" | "heirs" | "workers" | "pen";
/** round 2 §7: `retires` the chore the rung takes off the player's hands; `litBy` the feat or the time that lit it (a lit rung). */
export type Rung = { id: RungId; label: string; lit: boolean; cond: string; retires: string; litBy?: string };

/* copy:label */
const LABEL: Record<RungId, string> = { weights: "tactics", orders: "orders", heirs: "heirs", workers: "workers", pen: "pen" };
/* copy:tooltip */
const TIP: Record<RungId, string> = { weights: "packages worn · how he fights", orders: "standing orders · run setup, Legacy, ranks", heirs: "heir order · who succeeds", workers: "hired hands · chores while away", pen: "own rules · written by hand" };
/* copy:callout */
const FALLBACK: Record<RungId, string> = { weights: "second stance", orders: "first supplies", heirs: "first death", workers: "first hire", pen: "meet Mother" };

/* copy:callout */
const RETIRES: Record<RungId, string> = { weights: "each move", orders: "each send", heirs: "picking heirs", workers: "town chores", pen: "package limits" };
const trigger = (L: Pick<Lineage, "systems">, id: string): string | undefined => L.systems?.find((s) => s.id === id && !s.open)?.trigger;
/** what lit an open system: the core's `lit_by` (Cut 118 §6: its trigger, `time`, or `feat: …`), else its trigger */
const litBy = (L: Pick<Lineage, "systems">, id: string): string | undefined => { const s = L.systems?.find((x) => x.id === id && x.open); return s?.lit_by ?? s?.trigger; };
const sysOf = (L: Pick<Lineage, "systems">, id: string) => L.systems?.find((s) => s.id === id);

/** The four rungs as the lineage stands (`has`: the client's reveal steps, for the run setup the camp shows). */
export function ladderRungs(L: Lineage, has: (step: string) => boolean): Rung[] {
  const W = L.tree;
  const hired = !!W && workerNodes(W).some((n) => n.state === "done" && n.id !== "quartermaster");   // (the quartermaster is given)
  const focus = W ? worksView(W).focus : undefined;
  const first = W ? workerNodes(W).find((n) => n.state === "done" && n.id !== "quartermaster") : undefined;
  // the first hand hired: the chore it retired (`chests by hand`) and what lit it (`3 chests`, else the age fallback `6h`)
  const firstChore = first?.chore ? /* copy:callout */ `${choreWord(first.chore)} by hand` : undefined;
  const firstLit = first ? (first.need ? `${first.need} ${choreWord(first.chore)}` : first.fallback_h ? `${first.fallback_h}h` : undefined) : undefined;
  const rung = (id: RungId, lit: boolean, cond: string | undefined, by?: string, retires?: string): Rung =>
    ({ id, label: LABEL[id], lit, cond: cond || FALLBACK[id], retires: retires || RETIRES[id], ...(lit ? { litBy: by || FALLBACK[id] } : {}) });
  // owner amendment 2: the heir order's rung (`heirs` · retires picking heirs · lit by the first death) — only on a core that sends it
  const heirs = sysOf(L, "heirs");
  return [
    rung("weights", packagesShown(L), trigger(L, "stances"), litBy(L, "stances")),
    rung("orders", !!L.orders && ["loadout", "vault", "cage", "start"].some(has), trigger(L, "loadout"), litBy(L, "loadout")),
    ...(heirs ? [rung("heirs", heirs.open, heirs.trigger, litBy(L, "heirs"))] : []),
    rung("workers", hired, focus ? `${focus.name} · ${nodeState(focus)}` : undefined, firstLit, firstChore),
    rung("pen", onPackages(L) && penOpen(L), trigger(L, "pen"), litBy(L, "pen")),
  ];
}

/** A rung's tip (hover / long-press): what it is, the chore it retires, the feat or time that lit it (or what opens it). */
function rungTip(el: HTMLElement, r: Rung): HTMLElement {
  return detailHost(el, () => [h("div", { class: "kw-tip-head" }, h("b", null, r.label)),
    h("div", { class: "num" }, /* copy:label */ "Retires", ` · ${r.retires}`),
    h("div", { class: "num" }, r.lit ? /* copy:label */ "Lit by" : /* copy:label */ "Opens", ` · ${r.lit ? r.litBy : r.cond}`),
    h("div", { class: "kw-tip-gloss" }, TIP[r.id])]);
}

/** Cut 120 §8: a lit rung with a door opens it (the orders rung: the orders sheet); the others stay words. */
export function controlLadder(app: App, doors: Partial<Record<RungId, () => void>> = {}): { el: HTMLElement; dispose: () => void } {
  const el = h("div", { class: "control-ladder", "data-ladder": "" });
  let last = "";
  const paint = (): void => {
    const L = app.lineage, R = revealed(app);
    const rungs = ladderRungs(L, (s) => R.has(s as Parameters<typeof R.has>[0]));
    const next = rungs.find((r) => !r.lit);
    const show = onPackages(L) && L.best_depth > 0 && !!next;
    el.hidden = !show;
    el.dataset.ladder = rungs.filter((r) => r.lit).map((r) => r.id).join(",");
    const key = JSON.stringify([show, rungs]);
    if (!show || key === last) { if (!show) { last = key; replace(el); } return; }
    last = key;
    replace(el, h("div", { class: "ladder-rail", role: "list" },
      ...rungs.flatMap((r, i) => [i ? h("span", { class: "ladder-step", "aria-hidden": "true" }, "›") : "",
        rungTip(h("span", { class: `ladder-rung${r.lit ? " lit" : ""}${r === next ? " next" : ""}`, role: "listitem", "data-rung": r.id, "data-retires": r.retires, "data-lit-by": r.litBy ?? "" }, h("span", { class: "ladder-pip", "aria-hidden": "true" }),
          r.lit && doors[r.id] ? h("button", { class: "ladder-door", "data-door": r.id, onclick: (e: Event) => { e.stopPropagation(); doors[r.id]!(); } }, r.label) : r.label), r)])),
      h("small", { class: "ladder-next num", "data-rung": next!.id }, next!.cond));
  };
  paint();
  const off = app.onChange(paint);
  return { el, dispose: off };
}
