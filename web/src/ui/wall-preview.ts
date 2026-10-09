// Cut 118 round 2 (new): a roster preview of the next wall on the camp's chart — the first band boss not yet slain, his floor, and his
// counter as the lineage knows it (`counter · fire`, or `counter ?` with what would teach it). Its face carries the enemy tip (the
// facts, the ledger). The core's roster (`BossWall.affix · affix_counter · guard · guard_counter`): the heir's affix on him and its answer,
// his guard and its answer. Owner amendment §2: the siege the line has laid to him (`Queen · 4 tries · best 22%`).
import type { BossWall, Lineage } from "../engine/types";
import { h } from "./dom";
import { unitLabel } from "./unit-icon";
import { enemyHost } from "./enemy-tips";
import { siegeOn, siegeText } from "./feats";

/** The next wall: the shallowest band boss not slain (the core's `walls`, the Warlord to the first past the best). */
export function nextWall(L: Pick<Lineage, "walls">): BossWall | undefined {
  return [...(L.walls ?? [])].filter((w) => !w.slain).sort((a, b) => a.depth - b.depth)[0];
}

export function wallPreview(L: Lineage): HTMLElement | null {
  const w = nextWall(L);
  if (!w) return null;
  const counter = w.known && w.counter ? w.counter : w.learn ? `? · ${w.learn}` : "?";
  const more = [w.affix ? /* copy:callout */ `affix ${w.affix}${w.affix_counter ? ` → ${w.affix_counter}` : ""}` : "",
    w.guard ? /* copy:callout */ `guard ${w.guard}${w.guard_counter ? ` → ${w.guard_counter}` : ""}` : ""].filter(Boolean);
  const siege = siegeOn(L, w.boss);
  const face = enemyHost(unitLabel(w.boss, h("span", { class: "wall-who" }, h("b", null, w.title), " ", h("span", { class: "num" }, `D${w.depth}`)), { px: 30 }), w.boss, L);
  return h("div", { class: "wall-preview num", "data-boss": w.boss, "data-depth": w.depth },
    h("span", { class: "label wall-label" }, /* copy:label */ "next wall"), face,
    h("small", { class: "wall-counter", "data-known": w.known && w.counter ? "1" : "0" }, /* copy:label */ "counter", ` · ${counter}`),
    ...more.map((m) => h("small", { class: "wall-more dim" }, m)),
    siege ? h("small", { class: "wall-siege", "data-tries": siege.tries, title: /* copy:tooltip */ `siege edge · +${siege.edge_pct}% on him` }, siegeText(siege)) : "");
}
