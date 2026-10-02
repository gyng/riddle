// Cut 30 §5 — the quest board (docs/CUT30.md): one quest at a time, one plain goal line (≤ 5 words, the core's: `reach D10 · no return`),
// the reward as a picture (the vignettes in art/ui/oath/, packed to /ui/quest/), a progress bar. Kept → `QUEST DONE` (a beat, a report
// line) and a new quest the next day; one free swap a day. No stake: no price, forswear, slots, draws or commissions. It arrives with the
// Warlord slain (the core's `quests` system) as a tile on the building bar; the board prop on the town scene is the town's.
import "../cut30.css";
import type { App } from "../app";
import type { Lineage, Quest } from "../engine/types";
import { h, replace } from "./dom";
import { openSheet } from "./sheet";
import { sysOpen } from "./systems";
import skin from "./skin.json";
import { kw } from "./tips";

const PICTURES = new Set((skin as { quest?: string[] }).quest ?? []);
/** The reward's picture (`title` · `row` · `slot` · `card`), else null (the caller draws the glyph). */
export const rewardSrc = (reward: string): string | null => (PICTURES.has(reward) ? `${import.meta.env.BASE_URL}ui/quest/${reward}.webp` : null);
/** The fallback glyph per reward (art never blocks the game). */
const GLYPH: Record<string, string> = { title: "✠", row: "▤", slot: "☗", card: "✦" };

/** The board stands: the core opened it (the Warlord slain) and a quest is on it. */
export const questShown = (L: Lineage | undefined): boolean => !!L?.town?.quest && sysOpen(L, "quests");

/** The reward as a picture in an iron frame (no words: the picture is the reward). */
export function rewardPicture(reward: string): HTMLElement {
  const src = rewardSrc(reward);
  return h("span", { class: "quest-reward", "data-reward": reward, "aria-hidden": "true" },
    src ? h("img", { src, alt: "", draggable: "false" }) : h("span", { class: "quest-glyph" }, GLYPH[reward] ?? "✦"));
}
/** The progress bar (0..1), the share beside it. */
export const questBar = (q: Pick<Quest, "progress" | "done">): HTMLElement => h("span", { class: `quest-bar${q.done ? " done" : ""}`, "aria-hidden": "true" }, h("span", { class: "fill", style: `width:${Math.round(Math.max(0, Math.min(1, q.done ? 1 : q.progress)) * 100)}%` }));

/** Opens the board (a sheet anchored to its tile). */
export function openQuest(app: App, anchor?: HTMLElement | null): void {
  openSheet((close) => {
    const body = h("div", { class: "sheet-body quest-panel" });
    const paint = (): void => {
      const q = app.lineage.town?.quest; if (!q) { close(); return; }
      const swap = app.engine.swapQuest && !q.done
        ? h("button", { class: "chip mini quest-swap", disabled: !q.swap, onclick: () => void app.mutate(() => app.engine.swapQuest!(), undefined).then(() => paint()) }, q.swap ? /* copy:button */ "new quest" : /* copy:button */ "swapped today")
        : "";
      replace(body, h("div", { class: "label row-label" }, kw("quest")),
        h("div", { class: `quest-card${q.done ? " done" : ""}` },
          rewardPicture(q.reward),
          h("div", { class: "quest-main" },
            h("b", { class: "quest-goal" }, q.goal),
            h("div", { class: "quest-prog" }, questBar(q), h("small", { class: "num" }, `${Math.round((q.done ? 1 : q.progress) * 100)}%`)),
            q.done ? h("span", { class: "quest-stamp" }, /* copy:callout */ "QUEST DONE") : "")),
        h("div", { class: "quest-foot" }, swap, (app.lineage.town?.quests_done ?? 0) > 0 ? h("small", { class: "quest-n num dim" }, /* copy:callout */ `${app.lineage.town!.quests_done} done`) : ""));
    };
    paint();
    return body;
  }, { anchor });
}
