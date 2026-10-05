// Cut 5 §2: the lineage chronicle — one line per ended heir, from the core (`Lineage.chronicle`, cap 40),
// newest first, monospace, nothing added. A core without the field shows an empty sheet.
// Cut 9 §7: a line whose heir's grave carries `death_id` (the core keeps the last 5 deaths' traces) is a button that opens
// that death (trace + patches, `engine.death(id)`); every other line stays plain text.
// QA on 952e306 ("chronicle empty: only '·'"; "old death screen, no back/close, Escape inert"): the sheet carries its label
// and nothing else while empty; a kept death opens with `kept`, so Escape (and `edit`) lead back to the camp.
import type { App } from "../app";
import type { Lineage } from "../engine/types";
import { h } from "./dom";
import { closeAllSheets, openSheet } from "./sheet";

/** The kept death behind a chronicle line: the line's `♟N` names the heir, the newest grave of that heir carries the id. */
export function keptDeath(L: Lineage, line: string): number | undefined {
  const heir = Number(/^♟(\d+)/.exec(line)?.[1]);
  if (!heir) return undefined;
  const grave = [...(L.graveyard ?? [])].reverse().find((g) => g.heir === heir);
  return grave?.death_id;
}

export function openChronicle(app: App): void {
  openSheet(() => {
    const L = app.lineage;
    const others=(L.hero_slots??[]).filter(s=>s.id!==L.selected_bloodline&&(s.chronicle?.length??0)>0);
    const lines = [...(L.chronicle ?? [])].reverse();
    return h("div", { class: "sheet-body" }, h("div", { class: "label" }, /* copy:label */ "chronicle"), h("div", { class: "chronicle" }, ...others.map(s=>h("section",null,h("b",null,s.name),...(s.chronicle??[]).slice().reverse().map(line=>h("button",{class:"cline",onclick:()=>void app.selectBloodline(s.id).then(ok=>{if(ok)openChronicle(app);})},line)))), ...lines.map((line) => {
      const id = keptDeath(L, line);
      if (id === undefined) return h("div", { class: "cline" }, line);
      return h("button", { class: "cline kept", onclick: () => {
        void app.busy(/* copy:label */ "verdict", () => app.engine.death(id)).then((death) => { closeAllSheets(); app.go({ kind: "death", death, kept: true }); })
          .catch((e) => console.warn("kept death", e));
      } }, line, h("small", { class: "dim" }, " ▸"));
    })));
  });
}
