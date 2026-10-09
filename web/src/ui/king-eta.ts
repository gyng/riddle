// Cut 118 §9 (research/IDLE_STEAM_2026-10.md §10, Gnorp, Nodebuster): the ending in sight — when the King (the Mirror King, D33, the
// dungeon's bottom) may fall at the current pace: `King · ~day 23`. CRUDE, and it says so (the `~` and its tip): the client has no
// sim of days, only the lineage's age (`age_h`) and its best floor, and the runs log's records (`RunRec.best`, `clock_s`). The depth a
// lineage reaches grows slower the deeper it goes (the dayplayer's IDLE: D8 on day 1, D23 by day 12 — about depth ∝ age^0.45), so the
// line is a power curve through today's best, its bend read off the log's records when they span a few hours, else 0.45. The
// estimate is the day the curve reaches D33. Not a forecast of the rules: a pace. (Core field wanted: `Lineage.king_eta_h`, the core's
// own projection from its sims, would replace this.)
import type { Lineage } from "../engine/types";
import { h } from "./dom";

/** The King's floor (core `descent::BOSS_DEPTHS`: `mirror_king` at 33). */
export const KING_DEPTH = 33;
const BEND = 0.45;

export type KingEta = { day: number; today: number; far: boolean; bend: number };

/** The day (1 = the lineage's first 24 h) the King may fall at the current pace; null before there is a pace to read (best < D5, under
 *  an hour old), once he is slain, or on an ended lineage. `far` past day 99. */
export function kingEta(L: Pick<Lineage, "age_h" | "best_depth" | "walls" | "ended" | "runs" | "clock_s" | "trophies">): KingEta | null {
  const age = L.age_h, best = L.best_depth;
  if (age === undefined || !(age >= 1) || best < 5 || L.ended) return null;
  if (L.walls?.some((w) => w.boss === "mirror_king" && w.slain) || (L.trophies ?? []).some((t) => /mirror_king/.test(t))) return null;
  // the bend off the log: the oldest record still held against today's best, when they are hours apart and the depth moved
  let bend = BEND;
  const recs = (L.runs ?? []).filter((r) => r.best && r.depth > 0).sort((a, b) => a.clock_s - b.clock_s);
  const now = L.clock_s;
  if (recs.length && now !== undefined) {
    const r0 = recs[0], back = (now - r0.clock_s) / 3600, age0 = age - back;
    if (back >= 3 && age0 >= 0.5 && best > r0.depth) {
      const k = Math.log(best / r0.depth) / Math.log(age / age0);
      if (Number.isFinite(k)) bend = Math.min(1, Math.max(0.2, k));
    }
  }
  const today = Math.floor(age / 24) + 1;
  const left = Math.max(KING_DEPTH, best + 0.5) / best;
  const atH = age * Math.pow(left, 1 / bend);
  const day = Math.max(today, Math.floor(atH / 24) + 1);
  return { day, today, far: day > 99, bend };
}

/** Round 2 §9: how far down to the King, 0..1 (the best floor of his 33). */
export const kingShare = (L: Pick<Lineage, "best_depth">): number => Math.max(0, Math.min(1, L.best_depth / KING_DEPTH));
export const kingSlain = (L: Pick<Lineage, "walls" | "ended" | "trophies">): boolean =>
  !!L.ended || !!L.walls?.some((w) => w.boss === "mirror_king" && w.slain) || (L.trophies ?? []).some((t) => /mirror_king/.test(t));

/** `King · ~day 23` (`King · far off`), a thin % bar to him and what comes after him (`then ascend`: the ending's next descent); its tip
 *  names the crudeness. Slain: `King slain · then ascend`. Null when there is nothing to say yet. */
export function kingLine(L: Parameters<typeof kingEta>[0], cls = ""): HTMLElement | null {
  const slain = kingSlain(L);
  const e = slain ? null : kingEta(L);
  if (!slain && !e) return null;
  const pct = Math.round(kingShare(L) * 100);
  return h("span", { class: `king-eta num ${cls}`.trim(), "data-day": slain ? "slain" : e!.far ? "far" : e!.day, "data-pct": slain ? 100 : pct,
    title: /* copy:tooltip */ "crude · current pace carried on · not a promise" },
    h("b", null, /* copy:label */ "King"), " · ", slain ? /* copy:callout */ "slain" : e!.far ? /* copy:callout */ "far off" : /* copy:callout */ `~day ${e!.day}`,
    h("span", { class: "king-bar", role: "img", "aria-label": `${slain ? 100 : pct}%` }, h("span", { class: "fill", style: `width:${slain ? 100 : pct}%` })),
    h("small", { class: "king-pct" }, `${slain ? 100 : pct}%`),
    h("small", { class: "king-after dim" }, /* copy:callout */ " · then ascend"));
}
