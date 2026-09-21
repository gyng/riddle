// Return report: learned · bests · found · deaths · pending · reel · marks. Delta, not totals.
// Cut 10 §3: the exit tiles read `banked · returned · deaths`, `returned` first when it is the larger; each exit line leads
// with its tier and the kept sum (`returned $61`) before the engine's arithmetic; a lost companion reads `jackal Ashar fell`;
// after an absence the tiles fade in (the merged report is complete by the time this mounts).
// Cut 13 §3: the night's ledger — a `spent` section beside `salvaged` (`heal ×16 · −$640`, `ReturnReport.spent`) and one dim
// `gold` line under the tiles that reconciles the header's delta: `+$412 banked · +$96 returned · +$45 salvage · −$640 spent`
// (the banked / returned sums are the exit lines'; numbers only, a piece shows only when it is not zero).
import type { App, Mounted } from "../app";
import type { Counter, ExitLine, ReturnReport } from "../engine/types";
import { h, items, spanOf } from "./dom";
import { patchRows } from "./patches";
import { openUnlockSheet, visible, withRowsGate } from "./unlocks";
import { lostLabel, rowLabel } from "./tokens";
import { traceChip } from "./trace";
import { openGoldSheet } from "./gold";

const EXITS_SHOW = 8;

/** Cut 10 §3: an exit line's lead — the tier from its keep share and the sum kept: `returned $61` · `banked $84` · `died $0`. */
export function exitLead(x: ExitLine): string {
  const tier = x.keep_pct >= 100 ? /* copy:label */ "banked" : x.keep_pct <= 0 ? /* copy:label */ "died" : /* copy:label */ "returned";
  return `${tier} $${x.kept}`;
}

/** The ledger line with its lead in bold: the engine's text leads with `died $0 · …` (Cut 10 §3) and is split there; a text
 *  without a lead (an older slice) gets one in front — never two (`died $0 · died $0 · $190 carried` on every real report). */
export function ledgerText(x: ExitLine): (string | HTMLElement)[] {
  const m = /^((?:banked|returned|died) \$-?\d+)(?: · )?(.*)$/s.exec(x.text);
  if (m) return [h("b", { class: "lead" }, m[1]), m[2] ? " · " : "", m[2]];
  return [h("b", { class: "lead" }, exitLead(x)), " · ", x.text];
}

export function renderReport(app: App, r: ReturnReport, absence = false): Mounted {
  const L = app.lineage;
  const deathsN = r.deaths.reduce((n, d) => n + d.n, 0);
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile" }, h("b", { class: "num" }, n), h("span", { class: "label" }, label));
  // Cut 2 §1: `banked · returned · deaths` as a second row of three when the core reports exits; else the Cut 1 four
  const exits = r.banked !== undefined || r.returned !== undefined;
  const banked = tile(`${r.banked ?? 0}`, /* copy:label */ "banked"), returned = tile(`${r.returned ?? 0}`, /* copy:label */ "returned");
  const tiles = h("div", { class: `tiles${exits ? " six" : ""}${absence ? " fade-in" : ""}` },
    tile(`${r.sampled ? "~" : ""}${r.runs}`, /* copy:label */ "runs"),
    exits ? null : tile(`${deathsN}`, /* copy:label */ "deaths"),
    // the send's deepest floor, a delta like the tiles beside it (the lineage best is in the header; both QA players read
    // `1 RUNS · D4 BEST` as this send's); an old wire without it shows the lineage best
    r.deepest !== undefined ? tile(`D${r.deepest}`, /* copy:label */ "deepest") : tile(`D${L.best_depth}`, /* copy:label */ "best"),
    tile(`◆${r.marks_earned > 0 ? "+" : ""}${r.marks_earned}`, /* copy:label */ "marks"),
    // Cut 10 §3: `returned` leads when it is the larger (fourteen returns beside `banked 0` read as a contradiction)
    ...(exits ? ((r.returned ?? 0) > (r.banked ?? 0) ? [returned, banked] : [banked, returned]) : []),
    exits ? tile(`${deathsN}`, /* copy:label */ "deaths") : null,
  );
  const rested = r.rested_s ? h("div", { class: "rest-line dim num" }, /* copy:label */ "rested", " ", spanOf(r.rested_s)) : null;
  // Cut 13 §3: the gold line — what the exits brought (banked / returned, off the exit lines), the salvage, the automations' spending
  const goldLine = (): HTMLElement | null => {
    if (!r.spent && !r.salvaged && !r.gold) return null;
    const ex = r.exits ?? [];
    const bankedG = ex.filter((x) => x.keep_pct >= 100).reduce((a, x) => a + x.kept, 0), returnedG = ex.filter((x) => x.keep_pct > 0 && x.keep_pct < 100).reduce((a, x) => a + x.kept, 0);
    const salvageG = (r.salvaged ?? []).reduce((a, x) => a + x.gold, 0), spentG = (r.spent ?? []).reduce((a, x) => a + x.gold, 0);
    const pieces: (string | HTMLElement)[] = [];
    const WORD = /* copy:callout */ { banked: "banked", returned: "returned", home: "home", salvage: "salvage", wake: "wake", spent: "spent" };
    const piece = (n: number, sign: string, word: string, cls: string): void => { if (n > 0) pieces.push(h("span", { class: cls }, `${sign}$${n} ${word}`)); };
    // the core's summary is to the coin over every run of the absence (the exit lines are capped per slice): it wins
    if (r.gold) { piece(r.gold.home, "+", WORD.home, "up"); piece(r.gold.salvage, "+", WORD.salvage, "up"); piece(r.gold.wake, "+", WORD.wake, "up"); piece(r.gold.spent, "−", WORD.spent, "down"); }
    else { piece(bankedG, "+", WORD.banked, "up"); piece(returnedG, "+", WORD.returned, "up"); piece(salvageG, "+", WORD.salvage, "up"); piece(spentG, "−", WORD.spent, "down"); }
    if (!pieces.length) return null;
    const out: (string | HTMLElement)[] = []; pieces.forEach((p, i) => { if (i) out.push(" · "); out.push(p); });
    return h("div", { class: "gold-line dim num" }, ...out);
  };
  // Cut 6 §1: one ledger line per exit, verbatim from the engine, under the tiles (oldest first; the core keeps the last 5
  // per slice and the client merges slices, so a long absence shows its last EXITS_SHOW)
  // Cut 9 §5: an exit that carries its trace gets a `trace` chip after the line
  // Cut 10 §3: `returned $61` leads each line, the engine's arithmetic after it
  // Cut 11 §5: the line is tappable — the gold sheet filtered to that run (an exit claims the newest matching ledger exit the
  // exits after it in this report have not); its `trace` chip shows the chain (§3)
  const allExits = r.exits ?? [];
  // the client merges the absence's slices, so the report can hold more lines than EXITS_SHOW: the last EXITS_SHOW show and
  // `· N more` is a button that expands to every line (QA on e0f87e7: "`· 8 more` is inert text"); runs the tiles count that
  // have no line at all (the core keeps the last few per slice) are `· N unlisted`, dim and inert, so the count and the list
  // agree (QA on 952e306: "17 RUNS · 17 RETURNED but only 8 lines")
  const exitLines = allExits.length ? h("div", { class: "exit-lines" }) : null;
  const paintExits = (all: boolean): void => {
    if (!exitLines) return;
    const shown = all ? allExits : allExits.slice(-EXITS_SHOW);
    const hidden = allExits.length - shown.length, unlisted = Math.max(0, r.runs - allExits.length);
    exitLines.replaceChildren(...shown.map((x, i) => h("div", { class: "ledger-line num dim" },
      h("button", { class: "ledger-btn", onclick: () => openGoldSheet(app, x, shown.slice(i + 1)) }, ...ledgerText(x)),
      traceChip(x.trace, "chip mini", { rows: app.rules.rows, runId: x.run_id }))),   // Cut 11 §2: with the run, the chain's links get `watch`
      hidden > 0 ? h("button", { class: "ledger-line ledger-more num", onclick: () => paintExits(true) }, /* copy:button */ `· ${hidden} more`) : "",
      unlisted > 0 ? h("div", { class: "ledger-line num dim unlisted" }, /* copy:callout */ `· ${unlisted} unlisted`) : "");
  };
  paintExits(false);
  // Stall verdict (core README): every run came home and nothing got deeper — the row that ended them, then patches as on
  // the death screen (tap: replace / remove / insert, camp on the row). The core's line is the copy (≤ 12 words).
  const stall = r.stall ? h("section", { class: "rsec stall" },
    h("div", { class: "label" }, /* copy:label */ "stall"),
    h("div", { class: "stall-line num" }, r.stall.text, traceChip(r.stall.trace, "chip mini", { rows: app.rules.rows, runId: stallRun(r) })),   // Cut 9 §5: the trace of the last run the row ended; its rows labelled like the exits' (QA: "R1 · no item" lacked the verb); its run: the exit whose trace it is (QA on e0f87e7: no `watch` from a report)
    r.stall.patches.length ? patchRows(app, r.stall.patches) : null) : null;
  // Cut 2 §2: one line per pile recovered this send (the core sends `heir 3 · D7 · 4 items`, `bones:7:4` too; the watch
  // `D5 · 7 items`). Every line says it was found — `found ♟3's bones · D7 · 4 items` — since `bones D8 · 11 items · ♟3` read
  // as a pile still lying there (QA on e0f87e7: "survived 16 offline runs", "persisted through run 3")
  const bonesLine = (x: string): string => {
    const m = /^bones:(\d+):(\d+)$/.exec(x); if (m) return /* copy:callout */ `found bones · D${m[1]} · ${items(+m[2])}`;
    const c = /^heir (\d+) · (D\d+) · (\d+) items?$/.exec(x); if (c) return /* copy:callout */ `found ♟${c[1]}'s bones · ${c[2]} · ${items(+c[3])}`;
    const w = /^(D\d+) · (.*)$/.exec(x); if (w) return /* copy:callout */ `found bones · ${w[1]} · ${w[2]}`;
    return /^found\b/.test(x) ? x : /* copy:callout */ `found bones · ${x.replace(/^bones\s*/, "")}`;
  };
  const section = (label: string, body: Node | null): HTMLElement | null => body ? h("section", { class: "rsec" }, h("div", { class: "label" }, label), body) : null;
  const nice = (x: string): string => x.replace(/_/g, " ");
  // `R1 fired 3 of 16 runs: HP<50% → drink ?` names the row in the engine's short form; the report spells it as the
  // editor and the death screen do (`hp < 50% → drink unknown`) when the row is still in the set
  const rowSpelt = (x: string): string => {
    const m = /^R(\d+) fired (\d+) of (\d+) runs: (.*)$/.exec(x);
    const row = m && app.rules.rows[Number(m[1]) - 1];
    return row ? /* copy:death_line */ `R${m[1]} fired ${m[2]} of ${m[3]} runs: ${rowLabel(row)}` : x;
  };
  // repeats (three goblin archers tamed) collapse to one chip with a count
  const chips = (xs: string[], cls = "chip"): HTMLElement | null => {
    const n = new Map<string, number>(); for (const x of xs) n.set(x, (n.get(x) ?? 0) + 1);
    // "kind · name" (companions) renders the name small
    const label = (x: string): (string | HTMLElement)[] => { const i = x.indexOf(" · "); return i < 0 ? [nice(x)] : [nice(x.slice(0, i)), h("small", { class: "dim" }, ` ${x.slice(i + 3).replace(/ · fell$/, " fell")}`)]; };
    return n.size ? h("div", { class: "chips" }, ...[...n].map(([x, k]) => h("span", { class: cls }, ...label(x), k > 1 ? h("b", { class: "num" }, ` ×${k}`) : ""))) : null;
  };
  const lines = (xs: string[]): HTMLElement | null => xs.length ? h("ul", { class: "lines" }, ...xs.map((x) => h("li", null, nice(rowSpelt(x))))) : null;
  // identical reel lines (the same pattern in several runs) collapse to one with a count
  const reel = (xs: string[]): HTMLElement | null => {
    const n = new Map<string, number>(); for (const x of xs) n.set(x, (n.get(x) ?? 0) + 1);
    return n.size ? h("ul", { class: "lines" }, ...[...n].map(([x, k]) => h("li", null, x, k > 1 ? h("b", { class: "num" }, ` ×${k}`) : ""))) : null;
  };
  // pending: the engine's lines, with affordable unlocks shown as cards once the catalogue arrives
  const pendingBody = h("div", null);
  const pendingSec = section(/* copy:label */ "pending", pendingBody);
  const paintPending = (affordable: ReturnType<typeof visible>): void => {
    // `R1 fired n of m runs` lines come for every row (summed across slices); only the quiet ones are decisions
    const quiet = (p: string): boolean => { const m = /^R\d+ fired (\d+) of (\d+) runs/.exec(p); return !m || Number(m[1]) * 3 < Number(m[2]); };
    const pendingLines = (affordable.length ? r.pending.filter((p) => !/^unlock\b/.test(p)) : r.pending).filter(quiet);
    pendingBody.replaceChildren();
    const ul = lines(pendingLines); if (ul) pendingBody.appendChild(ul);
    // Cut 9 §2: the card opens its sheet; the buy is there, and the report repaints itself after one
    affordable = affordable.map((u) => withRowsGate(u, app.ownRows(), app.vocab.max_rows)).filter((u) => u.available);   // Cut 10 §3; Cut 12 §1: own rows
    if (affordable.length) pendingBody.appendChild(h("div", { class: "cards" }, ...affordable.map((u) => h("button", { class: "card", onclick: () => openUnlockSheet(app, u, () => app.go({ kind: "report", report: r })) }, h("span", null, u.label), h("span", { class: "num cost" }, `◆${u.cost}`)))));
    if (pendingSec) pendingSec.hidden = !pendingBody.childElementCount;
  };
  paintPending([]);
  void app.engine.unlocks().then((cat) => paintPending(visible(cat).filter((u) => u.available))).catch(() => { /* lines only */ });
  const open = r.worst_death ? h("button", { class: "btn", onclick: () => app.go({ kind: "death", death: r.worst_death!, lost: r.lost ?? [] }) }, /* copy:button */ "open") : null;
  const camp = h("button", { class: "btn primary", onclick: () => app.go({ kind: "camp" }) }, /* copy:button */ "camp");
  const el = h("main", { class: "report" },
    tiles, goldLine(), exitLines, rested, stall,
    section(/* copy:label */ "learned", factChips(r.learned, L.counters ?? [])),
    section(/* copy:label */ "tamed", chips(r.tamed ?? [], "chip ally")),
    section(/* copy:label */ "hatched", chips(r.hatched ?? [], "chip ally")),
    // Cut 10 §3: a companion `◯ jackal · Ashar fell` (the name small); Cut 12 §6: a summoned ally `ally hound fell`
    section(/* copy:label */ "lost", chips((r.lost ?? []).map((k) => k.includes(" · ") ? /* copy:callout */ `◯ ${k} fell` : lostLabel(k)), "chip egg")),
    section(/* copy:label */ "bests", lines(collapseBests(r.bests))),
    r.xp && (r.xp.gained > 0 || r.xp.level_ups > 0) ? section(/* copy:label */ "xp", h("div", { class: "xp-line num" }, `${r.xp.class} +${r.xp.gained}`, " · ", /* copy:label */ `L${L.classes?.[r.xp.class]?.level ?? 1}`, r.xp.level_ups > 0 ? h("b", null, ` ↑${r.xp.level_ups}`) : "")) : null,
    section(/* copy:label */ "found", chips(r.found.map((i) => i.label))),
    section(/* copy:label */ "bones", lines((r.bones_found ?? []).map(bonesLine))),
    section(/* copy:label */ "deaths", r.deaths.length ? h("ul", { class: "lines" }, ...r.deaths.map((d) => h("li", null, d.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${d.n}`)))) : null),
    section(/* copy:label */ "salvaged", r.salvaged?.length ? h("ul", { class: "lines" }, ...r.salvaged.map((s) => h("li", null, s.kind.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num gold" }, `$${s.gold}`)))) : null),
    // Cut 13 §3: what the automations bought this absence, per kind (`heal ×16 · −$640`)
    section(/* copy:label */ "spent", r.spent?.length ? h("ul", { class: "lines" }, ...r.spent.map((s) => h("li", null, s.kind.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num down" }, `−$${s.gold}`)))) : null),
    section(/* copy:label */ "renown", r.renown && r.renown.gained > 0 ? h("div", { class: "num" }, `+${r.renown.gained} · ★${r.renown.rank}`, r.renown.ranks_up > 0 ? h("b", { class: "up" }, ` ↑${r.renown.ranks_up}`) : "") : null),
    pendingSec,
    section(/* copy:label */ "reel", reel(r.reel.map((x) => x.text))),
    h("div", { class: "btn-row" }, open, camp),
  );
  return { el };
}

/** The run a stall's trace belongs to: the exit line that carries the same trace (the stall has no run id on the wire; the
 *  same turns, tick for tick, name the run), so its chain links can open the replay when the client holds that run. */
function stallRun(r: ReturnReport): number | undefined {
  const t = r.stall?.trace; if (!t) return undefined;
  const key = JSON.stringify(t.turns);
  return r.exits?.find((x) => x.trace && JSON.stringify(x.trace.turns) === key)?.run_id;
}

/** Facts grouped for reading: `foe:x`, `foe:x:t1`, `foe:x:t2` → one chip "x · t1 · t2"; `item:f=k` → "k (f)";
 *  `biome:x` → "x"; `boss:x:counter[=row]` → "x counter: attack boss" (Cut 6 §5: the lineage's counter text names the row);
 *  others verbatim. */
function factChips(facts: string[], counters: Counter[] = []): HTMLElement | null {
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
    const bn = /^bones:(\d+)$/.exec(f);
    if (bn) { rest.push(h("span", { class: "chip fact" }, /* copy:label */ "bones", h("small", null, ` D${bn[1]}`))); continue; }
    const c = /^boss:([^:]+):counter(?:=.*)?$/.exec(f);
    if (c) { const text = counters.find((k) => k.boss === c[1])?.text; rest.push(h("span", { class: "chip fact" }, nice(c[1]), h("small", null, /* copy:label */ " counter", text ? `: ${text}` : ""))); continue; }
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
