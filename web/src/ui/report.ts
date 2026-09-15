// Return report: learned · bests · found · deaths · pending · reel · marks. Delta, not totals.
import type { App, Mounted } from "../app";
import type { ReturnReport } from "../engine/types";
import { h } from "./dom";
import { available } from "./unlocks";

export function renderReport(app: App, r: ReturnReport): Mounted {
  const L = app.lineage;
  const deathsN = r.deaths.reduce((n, d) => n + d.n, 0);
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile" }, h("b", { class: "num" }, n), h("span", { class: "label" }, label));
  const tiles = h("div", { class: "tiles" },
    tile(`${r.sampled ? "~" : ""}${r.runs}`, /* copy:label */ "runs"),
    tile(`${deathsN}`, /* copy:label */ "deaths"),
    tile(`D${L.best_depth}`, /* copy:label */ "best"),
    tile(`◆${r.marks_earned > 0 ? "+" : ""}${r.marks_earned}`, /* copy:label */ "marks"),
  );
  const section = (label: string, body: Node | null): HTMLElement | null => body ? h("section", { class: "rsec" }, h("div", { class: "label" }, label), body) : null;
  const chips = (xs: string[], cls = "chip"): HTMLElement | null => xs.length ? h("div", { class: "chips" }, ...xs.map((x) => h("span", { class: cls }, x.replace(/_/g, " ")))) : null;
  const lines = (xs: string[]): HTMLElement | null => xs.length ? h("ul", { class: "lines" }, ...xs.map((x) => h("li", null, x))) : null;
  const affordable = available(L.unlocks).filter((u) => L.marks >= u.cost);
  const pendingLines = affordable.length ? r.pending.filter((p) => !/^unlock\b/.test(p)) : r.pending;
  const pending = pendingLines.length || affordable.length
    ? h("div", null, lines(pendingLines), affordable.length ? h("div", { class: "cards" }, ...affordable.map((u) => h("button", { class: "card", onclick: () => { app.buy(u.id); app.go({ kind: "report", report: r }); } }, h("span", null, u.label), h("span", { class: "num cost" }, `◆${u.cost}`)))) : null)
    : null;
  const open = r.worst_death ? h("button", { class: "btn", onclick: () => app.go({ kind: "death", death: r.worst_death!, lost: r.lost ?? [] }) }, /* copy:button */ "open") : null;
  const camp = h("button", { class: "btn primary", onclick: () => app.go({ kind: "camp" }) }, /* copy:button */ "camp");
  const el = h("main", { class: "report" },
    tiles,
    section(/* copy:label */ "learned", chips(r.learned, "chip fact")),
    section(/* copy:label */ "tamed", chips(r.tamed ?? [], "chip ally")),
    section(/* copy:label */ "hatched", chips(r.hatched ?? [], "chip ally")),
    section(/* copy:label */ "lost", chips((r.lost ?? []).map((k) => `◯ ${k}`), "chip egg")),
    section(/* copy:label */ "bests", lines(r.bests)),
    r.xp && r.xp.gained > 0 ? section(/* copy:label */ "xp", h("div", { class: "xp-line num" }, `${r.xp.class} +${r.xp.gained}`, " · ", /* copy:label */ `L${L.classes?.[r.xp.class]?.level ?? 1}`, r.xp.level_ups > 0 ? h("b", null, ` ↑${r.xp.level_ups}`) : "")) : null,
    section(/* copy:label */ "found", chips(r.found.map((i) => i.label))),
    section(/* copy:label */ "deaths", r.deaths.length ? h("ul", { class: "lines" }, ...r.deaths.map((d) => h("li", null, d.cause, " ", h("b", { class: "num" }, `×${d.n}`)))) : null),
    section(/* copy:label */ "salvaged", r.salvaged?.length ? h("ul", { class: "lines" }, ...r.salvaged.map((s) => h("li", null, s.kind.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num gold" }, `$${s.gold}`)))) : null),
    section(/* copy:label */ "renown", r.renown && r.renown.gained > 0 ? h("div", { class: "num" }, `+${r.renown.gained} · ★${r.renown.rank}`, r.renown.ranks_up > 0 ? h("b", { class: "up" }, ` ↑${r.renown.ranks_up}`) : "") : null),
    section(/* copy:label */ "pending", pending),
    section(/* copy:label */ "reel", lines(r.reel.map((x) => x.text))),
    h("div", { class: "btn-row" }, open, camp),
  );
  return { el };
}
