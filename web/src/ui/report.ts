// Return report: learned · bests · found · deaths · pending · reel · marks. Delta, not totals.
import type { App, Mounted } from "../app";
import type { ReturnReport } from "../engine/types";
import { h } from "./dom";
import { visible } from "./unlocks";

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
  const nice = (x: string): string => x.replace(/_/g, " ");
  // repeats (three goblin archers tamed) collapse to one chip with a count
  const chips = (xs: string[], cls = "chip"): HTMLElement | null => {
    const n = new Map<string, number>(); for (const x of xs) n.set(x, (n.get(x) ?? 0) + 1);
    // "kind · name" (companions) renders the name small
    const label = (x: string): (string | HTMLElement)[] => { const i = x.indexOf(" · "); return i < 0 ? [nice(x)] : [nice(x.slice(0, i)), h("small", { class: "dim" }, ` ${x.slice(i + 3)}`)]; };
    return n.size ? h("div", { class: "chips" }, ...[...n].map(([x, k]) => h("span", { class: cls }, ...label(x), k > 1 ? h("b", { class: "num" }, ` ×${k}`) : ""))) : null;
  };
  const lines = (xs: string[]): HTMLElement | null => xs.length ? h("ul", { class: "lines" }, ...xs.map((x) => h("li", null, nice(x)))) : null;
  // identical reel lines (the same pattern in several runs) collapse to one with a count
  const reel = (xs: string[]): HTMLElement | null => {
    const n = new Map<string, number>(); for (const x of xs) n.set(x, (n.get(x) ?? 0) + 1);
    return n.size ? h("ul", { class: "lines" }, ...[...n].map(([x, k]) => h("li", null, x, k > 1 ? h("b", { class: "num" }, ` ×${k}`) : ""))) : null;
  };
  // pending: the engine's lines, with affordable unlocks shown as cards once the catalogue arrives
  const pendingBody = h("div", null);
  const pendingSec = section(/* copy:label */ "pending", pendingBody);
  const paintPending = (affordable: ReturnType<typeof visible>): void => {
    const pendingLines = affordable.length ? r.pending.filter((p) => !/^unlock\b/.test(p)) : r.pending;
    pendingBody.replaceChildren();
    const ul = lines(pendingLines); if (ul) pendingBody.appendChild(ul);
    if (affordable.length) pendingBody.appendChild(h("div", { class: "cards" }, ...affordable.map((u) => h("button", { class: "card", onclick: () => void app.buy(u.id).then(() => app.go({ kind: "report", report: r })) }, h("span", null, u.label), h("span", { class: "num cost" }, `◆${u.cost}`)))));
    if (pendingSec) pendingSec.hidden = !pendingBody.childElementCount;
  };
  paintPending([]);
  void app.engine.unlocks().then((cat) => paintPending(visible(cat).filter((u) => u.available))).catch(() => { /* lines only */ });
  const open = r.worst_death ? h("button", { class: "btn", onclick: () => app.go({ kind: "death", death: r.worst_death!, lost: r.lost ?? [] }) }, /* copy:button */ "open") : null;
  const camp = h("button", { class: "btn primary", onclick: () => app.go({ kind: "camp" }) }, /* copy:button */ "camp");
  const el = h("main", { class: "report" },
    tiles,
    section(/* copy:label */ "learned", factChips(r.learned)),
    section(/* copy:label */ "tamed", chips(r.tamed ?? [], "chip ally")),
    section(/* copy:label */ "hatched", chips(r.hatched ?? [], "chip ally")),
    section(/* copy:label */ "lost", chips((r.lost ?? []).map((k) => `◯ ${k}`), "chip egg")),
    section(/* copy:label */ "bests", lines(collapseBests(r.bests))),
    r.xp && r.xp.gained > 0 ? section(/* copy:label */ "xp", h("div", { class: "xp-line num" }, `${r.xp.class} +${r.xp.gained}`, " · ", /* copy:label */ `L${L.classes?.[r.xp.class]?.level ?? 1}`, r.xp.level_ups > 0 ? h("b", null, ` ↑${r.xp.level_ups}`) : "")) : null,
    section(/* copy:label */ "found", chips(r.found.map((i) => i.label))),
    section(/* copy:label */ "deaths", r.deaths.length ? h("ul", { class: "lines" }, ...r.deaths.map((d) => h("li", null, d.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${d.n}`)))) : null),
    section(/* copy:label */ "salvaged", r.salvaged?.length ? h("ul", { class: "lines" }, ...r.salvaged.map((s) => h("li", null, s.kind.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num gold" }, `$${s.gold}`)))) : null),
    section(/* copy:label */ "renown", r.renown && r.renown.gained > 0 ? h("div", { class: "num" }, `+${r.renown.gained} · ★${r.renown.rank}`, r.renown.ranks_up > 0 ? h("b", { class: "up" }, ` ↑${r.renown.ranks_up}`) : "") : null),
    pendingSec,
    section(/* copy:label */ "reel", reel(r.reel.map((x) => x.text))),
    h("div", { class: "btn-row" }, open, camp),
  );
  return { el };
}

/** Facts grouped for reading: `foe:x`, `foe:x:t1`, `foe:x:t2` → one chip "x · t1 · t2"; `item:f=k` → "k (f)";
 *  `biome:x` → "x"; `boss:x:counter` → "x counter"; others verbatim. */
function factChips(facts: string[]): HTMLElement | null {
  const nice = (x: string): string => x.replace(/_/g, " ");
  const foes = new Map<string, string[]>();
  const rest: HTMLElement[] = [];
  for (const f of facts) {
    const m = /^foe:([^:]+)(?::(.+))?$/.exec(f);
    if (m) { const tags = foes.get(m[1]) ?? []; if (m[2]) tags.push(m[2]); foes.set(m[1], tags); continue; }
    const it = /^item:([^=]+)=(.+)$/.exec(f);
    if (it) { rest.push(h("span", { class: "chip fact" }, nice(it[2]), h("small", null, ` ${it[1]}`))); continue; }
    const b = /^biome:(.+)$/.exec(f);
    if (b) { rest.push(h("span", { class: "chip fact" }, nice(b[1]))); continue; }
    const c = /^boss:([^:]+):counter$/.exec(f);
    if (c) { rest.push(h("span", { class: "chip fact" }, nice(c[1]), h("small", null, /* copy:label */ " counter"))); continue; }
    rest.push(h("span", { class: "chip fact" }, nice(f)));
  }
  const out = [...foes].map(([k, tags]) => h("span", { class: "chip fact" }, nice(k), tags.length ? h("small", null, ` ${tags.map(nice).join(" · ")}`) : ""));
  return out.length + rest.length ? h("div", { class: "chips" }, ...out, ...rest) : null;
}

/** "rank 1 … rank 8", "fighter L2 … L4", "D3 … D6": one line per family, the highest, in first-seen order. */
function collapseBests(xs: string[]): string[] {
  const fam = (x: string): string | null => { const m = /^(rank|D|[a-z]+ L)(\d+)$/.exec(x); return m ? m[1] : null; };
  const best = new Map<string, string>(); const out: (string | null)[] = [];
  for (const x of xs) {
    const f = fam(x);
    if (!f) { out.push(x); continue; }
    if (!best.has(f)) { best.set(f, x); out.push(null); }
    else best.set(f, x);
  }
  const it = [...best.values()];
  return out.map((x) => x ?? it.shift()!);
}
