// Cut 5 §2: the lineage chronicle — one line per ended heir, from the core (`Lineage.chronicle`, cap 40),
// newest first, monospace, nothing added. A core without the field shows an empty sheet.
import type { App } from "../app";
import { h } from "./dom";
import { openSheet } from "./sheet";

export function openChronicle(app: App): void {
  openSheet(() => {
    const lines = [...(app.lineage.chronicle ?? [])].reverse();
    return h("div", { class: "sheet-body chronicle" }, ...lines.map((line) => h("div", { class: "cline" }, line)));
  });
}
