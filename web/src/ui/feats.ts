// Cut 118 (docs/CUT118_IDLE_LESSONS.md): the client half of the core's `feats.rs` — boss tokens (a seek chip), the weekly trials (one
// quiet card), the hero's wish (one chip), the sinks after Kit complete and the heir order (rows of the run setup), the siege
// (`Queen · 4 tries · best 22%`), the graves on their floors (`Ada's pack · $1240`), the finds' one reveal and the set log, and the
// report's feat lines. All truth is the core's (`Lineage.feats`, `ReturnReport.finds`, `ReturnReport.feats`); nothing here is required:
// every chip has a default the core keeps (`in order`, not opted, the sink order `both`, the heir order `answer`), and no chip waits on a
// tap. Quiet: short words on the face, the rest on hover (`title` / `detailHost`).
import "../feats.css";
import type { App } from "../app";
import type { FeatNews, FeatsWire, FindsReveal, GraveWire, Lineage, ReturnReport, SeekOption, SiegeWire, StandingOrders, TrialWire } from "../engine/types";
import { h, replace, toast, twoTap } from "./dom";
import { openWindow } from "./sheet";
import { detailHost } from "./tips";
import { audio } from "../audio";

/** `Queen · 4 tries · best 22%` (the siege on a band boss; `best` the lowest hp % he was left on). */
export const siegeText = (s: SiegeWire): string => /* copy:callout */ `${s.title} · ${s.tries} ${s.tries === 1 ? "try" : "tries"} · best ${s.best_pct}%`;
/** `Ada's pack · $1240` (a grave on its floor: the carry a death lost, brought home by the next heir to reach it). */
export const graveText = (g: GraveWire): string => /* copy:callout */ `${g.name}'s pack · $${g.gold}`;
export const gravesAt = (L: Pick<Lineage, "feats">, depth: number): GraveWire[] => (L.feats?.graves ?? []).filter((g) => g.depth === depth);
export const siegeOn = (L: Pick<Lineage, "feats">, boss: string): SiegeWire | undefined => L.feats?.siege.find((s) => s.boss === boss && s.tries > 0);

/** A floor's graves as a quiet mark (the chart's notch, the forecast's row): `Ada's pack · $1240`, the older on hover. */
export function graveMark(L: Pick<Lineage, "feats">, depth: number, cls = ""): HTMLElement | null {
  const gs = gravesAt(L, depth);
  if (!gs.length) return null;
  const el = h("small", { class: `grave-mark num ${cls}`.trim(), "data-grave": depth, title: /* copy:tooltip */ "carry lost here · next heir brings it home" },
    h("span", { class: "grave-glyph", "aria-hidden": "true" }, "✝"), " ", graveText(gs[gs.length - 1]), gs.length > 1 ? ` +${gs.length - 1}` : "");
  return el;
}

// ---------------------------------------------------------------- the camp: seek · trial · wish (one quiet row; each hidden without its core field)

/** The seek chip: `seek Queen ×2` (the deepest banked token; dim, the default `in order` stands) or `seeking Queen` (lit). A tap opens the
 *  tokens priced by `seekForecast()` (reach his floor · pass him, from his stone) and `in order` (the default), one tap each. */
function seekChip(app: App): HTMLElement | null {
  const F = app.lineage.feats;
  if (!F?.tokens.length || !app.engine.seekBoss) return null;
  const cur = F.seek ? F.tokens.find((t) => t.boss === F.seek) : undefined;
  const top = cur ?? [...F.tokens].sort((a, b) => b.depth - a.depth)[0];
  const b: HTMLButtonElement = h("button", { class: `chip feat-chip seek-chip num${cur ? " on" : ""}`, "data-seek": F.seek ?? "", "aria-pressed": cur ? "true" : "false",
    title: /* copy:tooltip */ "boss token · next send starts near him" },
    cur ? /* copy:callout */ `seeking ${cur.title}` : /* copy:callout */ `seek ${top.title}`, h("small", { class: "dim" }, ` ×${top.n}`));
  b.onclick = (e: Event) => { e.stopPropagation(); openSeek(app, b); };
  return b;
}

function openSeek(app: App, anchor: HTMLElement): void {
  openWindow((close) => {
    const list = h("div", { class: "seek-opts" });
    const pct = (x: number): string => `${Math.round(x * 100)}%`;
    const paint = (opts: SeekOption[] | null): void => {
      const F = app.lineage.feats;
      if (!F) { replace(list); return; }
      const set = async (boss: string): Promise<void> => { close(); if ((F.seek ?? "") !== boss) await app.mutate(() => app.engine.seekBoss!(boss), /* copy:callout */ boss ? "seek" : "in order"); };
      replace(list,
        h("button", { class: `chip seek-opt num${F.seek ? "" : " on"}`, "data-seek": "", onclick: () => void set("") }, /* copy:callout */ "in order", h("small", { class: "dim" }, /* copy:label */ " · default")),
        ...F.tokens.map((t) => {
          const o = opts?.find((x) => x.boss === t.boss);
          return h("button", { class: `chip seek-opt num${F.seek === t.boss ? " on" : ""}`, "data-seek": t.boss, onclick: () => void set(t.boss) },
            h("b", null, t.title), ` D${t.depth}`, h("small", { class: "dim" }, ` ×${t.n}`), t.stone > 1 ? h("small", { class: "dim" }, /* copy:callout */ ` · from D${t.stone}`) : "",
            o ? h("small", { class: "seek-price" }, /* copy:callout */ ` · reach ${pct(o.reach)} · past ${pct(o.past)}`) : opts === null ? h("small", { class: "dim" }, " …") : "");
        }));
    };
    paint(null);
    if (app.engine.seekForecast) void app.engine.seekForecast().then((o) => { if (list.isConnected) paint(o); }).catch(() => { if (list.isConnected) paint([]); });
    else paint([]);
    return h("div", { class: "sheet-body seek-sheet", "data-seek-sheet": "" }, h("div", { class: "label row-label" }, /* copy:label */ "Boss tokens"), list);
  }, { anchor });
}

/** The trials still open, newest first (a trial needs his band boss slain first: those wait unseen). */
export const openTrials = (F: FeatsWire | undefined): TrialWire[] => (F?.trials ?? []).filter((t) => t.open && !t.cleared);

/** The trial card: one quiet card for the open weeks — the boss and floor, the rule, the packages that answer it (owned lit), `+10 Legacy`
 *  and the opt-in (a toggle: `opt in` → `opted`; the next send while away plays it). Older open weeks are small chips that switch it. */
function trialCard(app: App, pickWeek: { w?: number }, repaint: () => void): HTMLElement | null {
  const ts = openTrials(app.lineage.feats);
  if (!ts.length || !app.engine.setTrial) return null;
  const t = ts.find((x) => x.week === pickWeek.w) ?? ts.find((x) => x.opted) ?? ts[0];
  const toggle = async (): Promise<void> => {
    const ok = await app.mutate(() => app.engine.setTrial!(t.opted ? -1 : t.week), /* copy:callout */ "trial");
    if (ok) toast(t.opted ? /* copy:callout */ "trial off" : /* copy:callout */ "trial on");
  };
  const owned = new Set(t.owned);
  const card = h("div", { class: `trial-card num${t.opted ? " opted" : ""}`, "data-trial": t.week, "data-opted": t.opted ? "1" : "0" },
    h("div", { class: "trial-head" }, h("span", { class: "label" }, /* copy:label */ "trial"), " ", h("b", null, t.title), ` D${t.depth}`,
      h("small", { class: "trial-pay dim" }, /* copy:callout */ ` · +${t.legacy} Legacy`)),
    h("div", { class: "trial-rule" }, t.label),
    t.answers.length ? h("div", { class: "trial-answers dim" }, /* copy:label */ "answers", " · ",
      ...t.answers.flatMap((a, i) => [i ? ", " : "", h("span", { class: `trial-answer${owned.has(a) ? " owned" : ""}`, "data-answer": a }, a.replace(/_/g, " "))])) : "",
    h("div", { class: "trial-foot" },
      h("button", { class: `chip mini trial-opt${t.opted ? " on" : ""}`, "aria-pressed": t.opted ? "true" : "false", onclick: (e: Event) => { e.stopPropagation(); void toggle(); } },
        t.opted ? /* copy:button */ "opted" : /* copy:button */ "opt in"),
      ...ts.filter((x) => x !== t).map((x) => h("button", { class: "chip mini trial-week dim", "data-week": x.week, onclick: (e: Event) => { e.stopPropagation(); pickWeek.w = x.week; repaint(); } },
        /* copy:callout */ `wk ${x.week} · ${x.title}`))));
  // (the tip hangs on the head: the card holds controls, whose taps stay their own)
  detailHost(card.querySelector<HTMLElement>(".trial-head")!, () => [h("div", { class: "kw-tip-head" }, h("b", null, /* copy:label */ "Weekly trial")),
    h("div", { class: "num" }, /* copy:label */ "Rule", ` · ${t.label}`),
    h("div", { class: "num" }, /* copy:label */ "Open", /* copy:callout */ ` · ${t.weeks_left + 1} wk left`),
    h("div", { class: "kw-tip-gloss" }, /* copy:tooltip */ "optional · played while away · never required")]);
  return card;
}

/** The hero's wish, one at a time: `a lantern` → `ok $40` (the house rule for gold); pays a little Legacy. Never decays. */
function wishChip(app: App): HTMLElement | null {
  const w = app.lineage.feats?.wish;
  if (!w || !app.engine.grantWish) return null;
  const b = twoTap([h("span", { class: "wish-glyph", "aria-hidden": "true" }, "✧"), document.createTextNode(` ${w.text}`), h("small", { class: "dim" }, ` · +${w.legacy}`)], /* copy:button */ `ok $${w.price}`,
    () => void app.mutate(() => app.engine.grantWish!(), /* copy:callout */ "wish").then((ok) => { if (ok) { audio.cue("unlock"); toast(/* copy:callout */ "wish granted"); } }),
    { class: "chip feat-chip wish-chip num", key: `wish:${w.id}`, disabled: !w.available });
  b.dataset.wish = w.id;
  b.title = /* copy:tooltip */ `hero's wish · $${w.price} · +${w.legacy} Legacy`;
  return b;
}

/** The camp's Cut 118 row: the seek chip and the wish chip on one line, the trial card under them. Hidden when none has anything. */
export function campFeats(app: App): { el: HTMLElement; dispose: () => void } {
  const el = h("div", { class: "camp-feats", "data-feats": "" });
  const pickWeek: { w?: number } = {};
  let last = "";
  const paint = (): void => {
    const L = app.lineage, F = L.feats;
    const home = !L.live && !L.ended && L.town?.home !== false;
    const key = JSON.stringify([home, F?.tokens, F?.seek, openTrials(F), F?.wish, pickWeek.w]);
    if (key === last) return;
    last = key;
    const seek = home ? seekChip(app) : null, wish = home ? wishChip(app) : null, trial = home ? trialCard(app, pickWeek, paint) : null;
    el.hidden = !seek && !wish && !trial;
    el.dataset.feats = [seek ? "seek" : "", wish ? "wish" : "", trial ? "trial" : ""].filter(Boolean).join(",");
    replace(el, seek || wish ? h("div", { class: "chips feat-chips" }, seek, wish) : "", trial);
  };
  paint();
  const off = app.onChange(paint), offL = app.onLive(paint);
  return { el, dispose: () => { off(); offL(); } };
}

// ---------------------------------------------------------------- the run setup: the heir order and the apprentice's sinks

/* copy:button */
export const HEIR_WORD: Record<string, string> = { answer: "answer", strongest: "strongest", surprise: "surprise" };
/* copy:button */
export const SINK_WORD: Record<string, string> = { both: "both", ration: "rations", tithe: "tithe", off: "off" };
export const HEIR_ORDERS = ["answer", "strongest", "surprise"] as const;
export const SINK_ORDERS = ["both", "ration", "tithe", "off"] as const;

/** The orders summary's Cut 118 bits — only an order off its default reads (`heirs strongest`, `sinks off`). */
export function featOrderBits(o: StandingOrders): string[] {
  return [o.heir && o.heir !== "answer" ? /* copy:callout */ `heirs ${HEIR_WORD[o.heir] ?? o.heir}` : "",
    o.sink && o.sink !== "both" ? /* copy:callout */ `sinks ${SINK_WORD[o.sink] ?? o.sink}` : ""].filter(Boolean);
}

/** The heir order is shown once a heir has died (the `heirs` rung lit), or as soon as the core sends a non-default one. */
export const heirOrderShown = (L: Lineage): boolean => !!L.orders?.heir && (L.systems?.some((s) => s.id === "heirs" && s.open) || (L.graveyard?.length ?? 0) > 0 || L.orders.heir !== "answer");

/** One sink by hand on this screen (round 2 §4): the survey while it is offered (once), else the tithe at its rate — two taps each. */
export function sinkHandChip(app: App, after: () => void): HTMLElement | null {
  const S = app.lineage.feats?.sinks ?? [];
  const survey = S.find((s) => s.id === "survey"), tithe = S.find((s) => s.id === "tithe");
  const s = survey && app.engine.buySurvey ? survey : tithe && app.engine.tithe ? tithe : undefined;
  if (!s) return null;
  const act = (): void => void app.mutate(() => (s.id === "survey" ? app.engine.buySurvey!() : app.engine.tithe!(1)), s.id === "survey" ? /* copy:callout */ "survey" : /* copy:callout */ "tithe").then((ok) => { if (ok) { audio.cue("unlock"); after(); } });
  const b = twoTap([document.createTextNode(s.id === "survey" ? /* copy:button */ "survey" : /* copy:button */ "tithe"), h("small", { class: "dim" }, ` · ${s.line}`)], /* copy:button */ `ok $${s.price}`, act,
    { class: `chip order sink-hand num`, key: `sink:${s.id}:${s.price}`, disabled: !s.available });
  b.dataset.sink = s.id;
  return b;
}

// ---------------------------------------------------------------- the report: finds, feats, siege

/* copy:label */
const FIND_KIND: Record<string, string> = { cosmetic: "keepsake", shard: "shard", piece: "set piece" };

/** Cut 118 §3 (round 2: one reveal, best first, never a click per find): `found · scholar's quill +6`, the shards' Legacy, a set completed.
 *  The whole list is the tip (hover / long-press). */
export function findsReveal(f: FindsReveal | undefined): HTMLElement | null {
  if (!f || !f.finds.length) return null;
  const best = f.best ?? f.finds[0];
  const el = h("div", { class: "report-reveal num", "data-sealed": f.sealed, "data-best": best.id },
    h("span", { class: "reveal-glyph", "aria-hidden": "true" }, "✦"), " ", h("b", null, best.name), h("small", { class: "dim" }, ` · ${FIND_KIND[best.kind] ?? best.kind}`),
    f.finds.length > 1 ? h("small", { class: "dim" }, ` +${f.finds.length - 1}`) : "",
    f.legacy ? h("small", { class: "up" }, /* copy:callout */ ` · +${f.legacy} Legacy`) : "",
    ...(f.sets ?? []).map((s) => h("small", { class: "reveal-set up" }, /* copy:callout */ ` · set ${s}`)));
  detailHost(el, () => [h("div", { class: "kw-tip-head" }, h("b", null, /* copy:label */ "Away finds")),
    ...f.finds.slice(0, 8).map((x) => h("div", { class: "num" }, x.name, h("small", { class: "dim" }, ` · ${FIND_KIND[x.kind] ?? x.kind}`))),
    ...(f.finds.length > 8 ? [h("div", { class: "num dim" }, `+${f.finds.length - 8}`)] : []),
    h("div", { class: "kw-tip-gloss" }, /* copy:tooltip */ "one sealed per run away · more after 6h")]);
  return el;
}

/** The set log (a fold under the report's details): each set's pieces, had or not, and its small bonus. */
export function setLog(F: FeatsWire | undefined): HTMLElement | null {
  const sets = F?.sets ?? [];
  if (!sets.some((s) => s.have.some(Boolean))) return null;
  return h("details", { class: "set-log", "data-sets": sets.filter((s) => s.done).length },
    h("summary", { class: "dim num" }, /* copy:button */ "set log", ` ${sets.filter((s) => s.done).length}/${sets.length}`),
    h("ul", { class: "lines" }, ...sets.map((s) => h("li", { class: `num set-line${s.done ? " done" : ""}`, "data-set": s.id },
      h("b", null, s.title), " ", ...s.pieces.map((p, i) => h("span", { class: `set-piece${s.have[i] ? " have" : ""}`, title: p.replace(/_/g, " ") }, s.have[i] ? "■" : "□")),
      h("small", { class: "dim" }, ` · ${s.bonus}`)))));
}

/** The feat kinds the report's lines name, most notable first (the heir chosen and the graves recovered are the report's quiet facts). */
const FEAT_RANK = ["siege_won", "title", "trial", "seek", "siege", "set", "heir", "recovered", "wish", "swift", "trial_failed"];
/** The report's feat lines (`ReturnReport.feats`): the first three on the card, the rest under details. Siege tries are summed into the
 *  siege line (`Queen · 4 tries · best 22%`), one per boss. */
export function featLines(r: ReturnReport, L: Pick<Lineage, "feats">): { card: HTMLElement | null; rest: HTMLElement | null } {
  const fs = [...(r.feats ?? [])];
  const sieged = new Set<string>();
  const lines: { k: string; text: string; day?: number }[] = [];
  for (const f of fs.sort((a, b) => rank(a) - rank(b))) {
    if (f.k === "siege") {
      const title = f.text.split(" · ")[0];
      if (sieged.has(title)) continue;
      sieged.add(title);
      const s = L.feats?.siege.find((x) => x.title === title);
      lines.push({ k: "siege", text: s ? siegeText(s) : f.text, day: f.day });
      continue;
    }
    lines.push({ k: f.k, text: f.text, day: f.day });
  }
  if (!lines.length) return { card: null, rest: null };
  const li = (x: { k: string; text: string; day?: number }): HTMLElement => h("li", { class: `num feat-line k-${x.k}`, "data-k": x.k, title: x.day ? /* copy:tooltip */ `day ${x.day}` : undefined }, x.text);
  return { card: h("ul", { class: "lines report-feats" }, ...lines.slice(0, 3).map(li)),
    rest: lines.length > 3 ? h("ul", { class: "lines report-feats more" }, ...lines.slice(3).map(li)) : null };
}
const rank = (f: FeatNews): number => { const i = FEAT_RANK.indexOf(f.k); return i < 0 ? FEAT_RANK.length : i; };

// ---------------------------------------------------------------- the town's graveyard

/** A stone: a chunky pixel headstone (an inline primitive: no sprite to fail). */
function stoneSvg(): string {
  return /* copy:none */ '<svg viewBox="0 0 8 10" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">'
    + '<rect x="2" y="0" width="4" height="1" fill="#8e8a7e"/><rect x="1" y="1" width="6" height="7" fill="#8e8a7e"/><rect x="1" y="1" width="1" height="7" fill="#b3ae9f"/>'
    + '<rect x="3" y="2" width="2" height="1" fill="#55524a"/><rect x="3" y="3" width="2" height="1" fill="#55524a"/>'
    + '<rect x="0" y="8" width="8" height="2" fill="#4c5a32"/></svg>';
}

const STONES_SHOWN = 4;
/** Cut 118 (owner amendment §4): the town's graveyard — a few stones at the scene's foot (the newest), each its epitaph on hover; a tap opens
 *  the whole yard (every stone, newest first) and the line's titles (`Motherbane`). Hidden with no stone. */
export function townGraveyard(app: App): { el: HTMLElement; paint(home: boolean): void } {
  const el: HTMLButtonElement = h("button", { class: "town-graveyard game-control", hidden: true, "data-stones": "0", "aria-label": /* copy:label */ "graveyard" });
  const open = (): void => {
    const F = app.lineage.feats;
    if (!F) return;
    openWindow(() => h("div", { class: "sheet-body graveyard-sheet", "data-graveyard": "sheet" },
      h("div", { class: "label row-label" }, /* copy:label */ "Graveyard"),
      F.titles.length ? h("div", { class: "graveyard-titles num" }, ...F.titles.flatMap((t, i) => [i ? " · " : "", h("b", { class: "line-title" }, t)])) : "",
      h("ul", { class: "lines graveyard-lines" }, ...[...F.graveyard].reverse().map((s) => h("li", { class: "num stone-line", "data-heir": s.heir },
        h("b", null, s.name), ` · ${s.epitaph}`, s.try_n ? h("small", { class: "dim" }, /* copy:callout */ ` · try ${s.try_n}`) : "",
        h("small", { class: "dim" }, /* copy:callout */ ` · day ${s.day}`))))), { anchor: el });
  };
  el.onclick = (e: Event) => { e.stopPropagation(); open(); };
  let last = "";
  const paint = (home: boolean): void => {
    const F = app.lineage.feats;
    const stones = F?.graveyard ?? [];
    const show = home && stones.length > 0;
    el.hidden = !show;
    el.dataset.stones = String(show ? stones.length : 0);
    const key = JSON.stringify([show, stones.length, stones.at(-1)?.heir, F?.titles]);
    if (key === last) return;
    last = key;
    if (!show) { replace(el); return; }
    const shown = stones.slice(-STONES_SHOWN);
    replace(el, ...shown.map((s) => { const st = h("span", { class: "stone", "data-heir": s.heir, title: `${s.name} · ${s.epitaph}` }); st.innerHTML = stoneSvg(); return st; }),
      stones.length > STONES_SHOWN ? h("small", { class: "stone-more num" }, `+${stones.length - STONES_SHOWN}`) : "",
      F!.titles.length ? h("small", { class: "stone-title num" }, F!.titles[F!.titles.length - 1]) : "");
  };
  detailHost(el, () => {
    const F = app.lineage.feats, s = F?.graveyard.at(-1);
    return s ? [h("div", { class: "kw-tip-head" }, h("b", null, s.name)), h("div", { class: "num" }, s.epitaph),
      ...(F!.titles.length ? [h("div", { class: "num" }, F!.titles.join(" · "))] : []), h("div", { class: "kw-tip-gloss" }, /* copy:tooltip */ "fallen heirs · their walls remember")] : [];
  });
  return { el, paint };
}
