import type { App, Mounted } from "../app";
import type { ReturnReport } from "../engine/types";
import { h, replace } from "./dom";
import { packageIcon } from "./skin";
import { openPackages } from "./packages";

/** Earned choices come from the report, not current ownership or selection. */
export function reportChoices(app: App, r: ReturnReport): Mounted {
  const catalogue = app.lineage.packages?.all ?? [];
  const groups = r.bloodlines?.some(s => s.packages !== undefined)
    ? [...r.bloodlines].sort((a, b) => a.id - b.id).map(s => ({ id: s.id, name: s.name, beats: s.packages ?? [] }))
    : [{ id: undefined, name: undefined, beats: r.packages ?? [] }];
  const choices = groups.flatMap(s => {
    const earned = new Set(s.beats.filter(b => b.startsWith("+")).map(b => b.slice(1)));
    return catalogue.filter(p => earned.has(p.name) && ["stance", "tactic", "temperament"].includes(p.kind))
      .map(p => ({ id: p.id, name: p.name, kind: p.kind, owner: s.id, ownerName: s.name }));
  });
  const el = h("section", { class: "report-choices", hidden: !choices.length });
  let topology = "";
  const kindLabel = (kind: string): string => kind === "stance" ? /* copy:label */ "Combat style" : kind === "tactic" ? /* copy:label */ "Extra tactic" : /* copy:label */ "Personality";
  const paint = (): void => {
    if (!choices.length) return;
    const slots = app.lineage.hero_slots ?? [];
    const nextTopology = JSON.stringify(slots.map(s => s.id));
    if (nextTopology === topology) return;
    topology = nextTopology;
    replace(el, h("b", { class: "row-label" }, /* copy:label */ "New choices"),
      h("div", { class: "report-choice-list" }, ...choices.map(c => {
        const owner = c.owner ?? (slots.length === 1 ? slots[0]!.id : undefined);
        const available = owner !== undefined && slots.some(s => s.id === owner);
        const contents = [packageIcon(c.id), h("span", { class: "report-choice-copy" }, h("b", null, c.name),
          h("small", null, [c.ownerName, kindLabel(c.kind)].filter(Boolean).join(" · ")))];
        return available ? h("button", { class: "chip report-choice", "data-choice": c.id, "data-owner": owner, onclick: async (e: Event) => {
          const anchor = e.currentTarget as HTMLElement;
          if (await app.selectBloodline(owner) && app.lineage.selected_bloodline === owner && app.lineage.hero_slots?.some(s => s.id === owner))
            openPackages(app, anchor.isConnected ? anchor : null, c.kind);
        } }, ...contents) : h("div", { class: "chip report-choice", "data-choice": c.id }, ...contents);
      })));
  };
  paint();
  const off = app.onChange(paint), offLive = app.onLive(paint);
  return { el, dispose: () => { off(); offLive(); } };
}
