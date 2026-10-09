// Cut 118 §8 (research/IDLE_STEAM_2026-10.md §9, Scapewatch's broadcasts; round 2: the hero's voice, the diary): the town crier — a
// small pixel herald at the town's top right who rings his bell once on the camp after an absence and cries the hero's most notable
// act in his own words, ≤ 3 (`slew the Warlord +2`). A tap unrolls the hero's diary: one dated line per notable act (`day 3 · I slew the
// Goblin Warlord on D8`), this return's first, then the earlier ones, newest first. The acts are the report's real events (its bests,
// its exits' news — the run traces' firsts and avengings —, the packages' beats, the oath, the workers' first acts, the bounty, the
// tamed); nothing here is game truth. The diary is kept per viewer (browser storage, a convenience: it is rebuilt from the reports it
// heard). Read once, he falls quiet until the next absence. Cut 118 core: each act's own lineage day (`News.day`, `FeatNews.day`) dates
// its diary line, and the core's feats (`ReturnReport.feats`: a siege won, a title, a trial cleared, a set completed, a sought boss slain,
// a wish granted, the swift floors, a grave recovered) are cried in his words.
import "../crier.css";
import type { App } from "../app";
import type { FeatNews, Lineage, ReturnReport } from "../engine/types";
import { h, replace } from "./dom";
import { openWindow } from "./sheet";
import { bossName } from "./report-bosses";

export type Cry = { k: string; short: string; long: string; day?: number };
const MOST = 6;
/** most notable first */
const RANK = ["boss", "siege_won", "title", "trial", "record", "seek", "avenged", "set", "first", "quest", "oath", "built", "recovered", "worker", "wish", "swift", "bounty", "tamed"];
/** Cut 118: a core feat in the hero's voice — `short` ≤ 3 words for the chrome, `long` the diary's (≤ 8 words); null for the report's quiet
 *  facts (the heir chosen, a siege try, a trial failed). */
function featCry(f: FeatNews): Cry | null {
  const head = f.text.split(" · ");
  const word1 = (t: string): string => t.split(/\s+/)[0] ?? t;
  switch (f.k) {
    case "siege_won": return { k: f.k, day: f.day, short: /* copy:callout */ `${word1(head[0])} fell`, long: /* copy:diary_line */ `We wore down the ${head[0]} · ${head[1] ?? ""}`.replace(/ · $/, "") };
    case "title": return { k: f.k, day: f.day, short: f.text.replace(/^the line is /, "").split(/\s+/).slice(0, 3).join(" "), long: /* copy:diary_line */ `Our line is called ${f.text.replace(/^the line is /, "")}` };
    case "trial": return { k: f.k, day: f.day, short: /* copy:callout */ "trial cleared", long: /* copy:diary_line */ `I cleared the trial · ${head[1] ?? ""}`.replace(/ · $/, "") };
    case "seek": return /slain$/.test(f.text) ? { k: f.k, day: f.day, short: /* copy:callout */ `sought ${word1(head[0])}`, long: /* copy:diary_line */ `I sought the ${head[0]} and slew him` } : null;
    case "set": return { k: f.k, day: f.day, short: /* copy:callout */ "set complete", long: /* copy:diary_line */ `A set complete · ${head[0]}` };
    case "wish": return { k: f.k, day: f.day, short: /* copy:callout */ "wish granted", long: /* copy:diary_line */ `${f.text.replace(/^granted /, "Granted: ").split(" · ")[0]}` };
    case "swift": return { k: f.k, day: f.day, short: f.text.split(/\s+/).slice(0, 2).join(" "), long: /* copy:diary_line */ `The early floors pass quickly now · ${f.text}` };
    case "recovered": return { k: f.k, day: f.day, short: /* copy:callout */ "pack recovered", long: /* copy:diary_line */ `I brought home ${head[0]}` };
    default: return null;
  }
}

/** The notable acts of a report in the hero's voice, most notable first, each once: `short` ≤ 3 words for the chrome, `long` the diary's. */
export function cries(r: ReturnReport, L: Pick<Lineage, "walls">): Cry[] {
  const out: Cry[] = [];
  const bests = [...(r.bests ?? []), ...(r.bloodlines ?? []).flatMap((b) => b.bests ?? [])];
  const title = (id: string): string => L.walls?.find((w) => w.boss === id)?.title ?? bossName(id);
  const last = (t: string): string => t.split(/\s+/).at(-1) ?? t;
  const bosses = new Set(bests.map((b) => /^boss:\s*([a-z][a-z0-9_]*)$/.exec(b)?.[1]).filter((x): x is string => !!x));
  for (const id of bosses) {
    const d = L.walls?.find((w) => w.boss === id)?.depth;
    out.push({ k: "boss", short: /* copy:callout */ `slew the ${last(title(id))}`, long: d ? /* copy:diary_line */ `I slew the ${title(id)} on D${d}` : /* copy:diary_line */ `I slew the ${title(id)}` });
  }
  const rec = Math.max(0, ...bests.map((b) => /^D(\d+)$/.exec(b)?.[1]).filter((x): x is string => !!x).map(Number));
  if (rec > 0) out.push({ k: "record", short: /* copy:callout */ `D${rec} · deepest yet`, long: /* copy:diary_line */ `Floor ${rec}, deeper than any before me` });
  // the run traces' own news (core `ExitLine.news`): a foe avenged, a first met (a boss's first fall is the bests' line already)
  for (const n of (r.exits ?? []).flatMap((x) => x.news ?? [])) {
    const av = n.k === "named" ? /^avenged (.+)$/.exec(n.text) : null;
    if (av) { out.push({ k: "avenged", day: n.day, short: /* copy:callout */ `avenged ${av[1].split(",")[0]}`, long: /* copy:diary_line */ `I avenged ${av[1]}` }); continue; }
    const f = n.k === "first" ? /^first: (.+)$/.exec(n.text) : null;
    if (f && !/slain$/.test(f[1])) out.push({ k: "first", day: n.day, short: /* copy:callout */ `met ${f[1].replace(/^the /, "")}`.split(/\s+/).slice(0, 3).join(" "), long: /* copy:diary_line */ `First time below: ${f[1]}` });
    // (the core's own day of a boss's first fall dates his line)
    const slain = n.k === "first" ? /^first: (?:the )?(.+?) slain$/i.exec(n.text) : null;
    if (slain) { const c = out.find((x) => x.k === "boss" && x.day === undefined && x.long.toLowerCase().includes(slain[1].toLowerCase())); if (c) c.day = n.day; }
  }
  for (const f of r.feats ?? []) { const c = featCry(f); if (c) out.push(c); }
  for (const beat of r.packages ?? []) {
    const q = /^quest done(?: · (.+))?$/i.exec(beat);
    if (q) { out.push({ k: "quest", short: /* copy:callout */ "quest done", long: q[1] ? /* copy:diary_line */ `I finished the quest · ${q[1]}` : /* copy:diary_line */ "I finished the board's quest" }); continue; }
    const b = /^built (\w+)$/i.exec(beat);
    if (b) out.push({ k: "built", short: /* copy:callout */ `a new ${b[1].toLowerCase()}`, long: /* copy:diary_line */ `Home again: a new ${b[1].toLowerCase()} stands` });
  }
  if (r.oath?.done) out.push({ k: "oath", short: /* copy:callout */ "oath kept", long: /* copy:diary_line */ `I kept my oath · ${r.oath.text}` });
  for (const w of (r.workers ?? []).filter((a) => a.first && (a.n > 0 || a.what))) {
    const who = w.id.replace(/_/g, " ");
    out.push({ k: "worker", short: /* copy:callout */ `${who} helps now`, long: /* copy:diary_line */ `The ${who} works for me now · ${w.what}` });
  }
  if (r.bounty?.taken) out.push({ k: "bounty", short: /* copy:callout */ "bounty claimed", long: /* copy:diary_line */ `I claimed the bounty on D${r.bounty.depth} · $${r.bounty.gold}` });
  const tamed = (r.tamed ?? []).length;
  if (tamed) out.push({ k: "tamed", short: tamed > 1 ? /* copy:callout */ `tamed ${tamed} foes` : /* copy:callout */ "tamed a foe", long: tamed > 1 ? /* copy:diary_line */ `I tamed ${tamed} foes below` : /* copy:diary_line */ "I tamed a foe below" });
  const seen = new Set<string>();
  return out.filter((c) => !seen.has(c.long) && !!seen.add(c.long)).sort((a, b) => RANK.indexOf(a.k) - RANK.indexOf(b.k)).slice(0, MOST);
}

/** The diary: dated lines, newest first, per lineage (browser storage; a per-viewer convenience — empty is fine). */
export type DiaryLine = { day: number; k: string; text: string };
const DIARY_MAX = 40;
const diaryKey = (L: Pick<Lineage, "seed">): string => /* copy:none */ `riddle.diary.${L.seed ?? 0}`;
export function readDiary(L: Pick<Lineage, "seed">): DiaryLine[] {
  try { const s = localStorage.getItem(diaryKey(L)); const v = s ? JSON.parse(s) : []; return Array.isArray(v) ? v : []; } catch { return []; }
}
/** Writes a return's cries under their days — the core's own day for each act (`News.day`, `FeatNews.day`), else the lineage's day now —
 *  once (a line already written on that day is not written again); the diary as it stands. */
export function writeDiary(L: Pick<Lineage, "seed" | "age_h">, cs: Cry[]): DiaryLine[] {
  const today = Math.floor((L.age_h ?? 0) / 24) + 1;
  const old = readDiary(L);
  const fresh = cs.map((c) => ({ day: c.day ?? today, k: c.k, text: c.long })).filter((x) => !old.some((y) => y.day === x.day && y.text === x.text));
  const all = [...fresh, ...old].slice(0, DIARY_MAX);
  try { localStorage.setItem(diaryKey(L), JSON.stringify(all)); } catch { /* private mode: this visit only */ }
  return all;
}

/** The reports whose cries were read (this session): the crier falls quiet on them. */
const heard = new WeakSet<ReturnReport>();
/** … and those he has rung for (once per absence, however often the camp mounts). */
const rung = new WeakSet<ReturnReport>();

/** The crier on the town: hidden with nothing to cry, away from home, or once read. */
export function townCrier(app: App): { el: HTMLElement; paint(home: boolean): void } {
  const el: HTMLButtonElement = h("button", { class: "town-crier game-control", hidden: true, "data-crier": "0" });
  const figure = (): HTMLElement => {
    const fig = h("span", { class: "crier-fig", "aria-hidden": "true" });
    // chunky pixel herald (12 × 16 px, crisp edges): tricorn, face, coat, the bell raised — a primitive, no sprite to fail
    fig.innerHTML = /* copy:none */ '<svg viewBox="0 0 12 16" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">'
      + '<rect x="2" y="1" width="6" height="2" fill="#1c1b2b"/><rect x="1" y="3" width="8" height="1" fill="#1c1b2b"/>'
      + '<rect x="3" y="4" width="4" height="3" fill="#eadfc5"/><rect x="2" y="7" width="6" height="6" fill="#7a2030"/>'
      + '<rect x="4" y="7" width="2" height="6" fill="#b89448"/><rect x="2" y="13" width="2" height="3" fill="#1c1b2b"/><rect x="6" y="13" width="2" height="3" fill="#1c1b2b"/>'
      + '<rect x="8" y="6" width="1" height="3" fill="#eadfc5"/>'
      + '<g class="crier-bell"><rect x="9" y="2" width="1" height="2" fill="#6a5a40"/><rect x="8" y="4" width="3" height="2" fill="#dbba7c"/><rect x="9" y="6" width="1" height="1" fill="#b89448"/></g></svg>';
    return fig;
  };
  const open = (r: ReturnReport, cs: Cry[]): void => {
    heard.add(r);
    el.hidden = true; el.dataset.crier = "0";
    const diary = writeDiary(app.lineage, cs);
    const now = new Set(cs.map((c) => c.long));
    openWindow(() => h("div", { class: "sheet-body crier-scroll", "data-crier": "scroll" },
      h("div", { class: "label row-label" }, /* copy:label */ "Hero's diary"),
      h("ul", { class: "lines crier-lines" }, ...diary.map((d, i) => h("li", { class: `crier-line k-${d.k}${i < cs.length && now.has(d.text) ? " new" : ""}`, "data-k": d.k },
        h("small", { class: "crier-day num dim" }, /* copy:callout */ `day ${d.day}`), " ", d.text)))), { anchor: el });
  };
  const paint = (home: boolean): void => {
    const r = app.lastAbsence?.report;
    const cs = r && home && !heard.has(r) ? cries(r, app.lineage) : [];
    el.hidden = !cs.length;
    el.dataset.crier = String(cs.length);
    if (!r || !cs.length) { replace(el); el.onclick = null; return; }
    replace(el, figure(), h("span", { class: "crier-cry num" }, cs[0].short, cs.length > 1 ? h("small", { class: "crier-more" }, ` +${cs.length - 1}`) : ""));
    el.setAttribute("aria-label", cs[0].short);
    el.onclick = (e: Event) => { e.stopPropagation(); open(r, cs); };
    // (he rings once per absence: a repaint does not ring again)
    if (!rung.has(r)) { rung.add(r); el.classList.remove("ringing"); void el.offsetWidth; el.classList.add("ringing"); }
  };
  return { el, paint };
}
