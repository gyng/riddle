// Settings sheet: save export/import, rules export/import, engine badge, reset lineage (double tap).
import type { App } from "../app";
import { h, copyText, replace } from "./dom";
import { openSheet } from "./sheet";

export function openSettings(app: App): void {
  openSheet((close) => {
    const body = h("div", { class: "sheet-body settings" });
    const area = h("textarea", { class: "ta", rows: 6, spellcheck: false });
    const row = (label: string, ...btns: HTMLElement[]): HTMLElement => h("div", { class: "srow" }, h("span", { class: "label" }, label), ...btns);
    const stamp = (b: HTMLElement, ok: boolean): void => { const t = b.textContent; b.textContent = ok ? "✓" : "×"; setTimeout(() => { b.textContent = t; }, 900); };

    const saveOut = h("button", { class: "btn", onclick: async () => { const s = app.exportSave(); area.value = s; stamp(saveOut, await copyText(s)); } }, /* copy:button */ "export");
    const saveIn = h("button", { class: "btn", onclick: async () => { const ok = await app.importSave(area.value.trim()); stamp(saveIn, ok); if (ok) close(); } }, /* copy:button */ "import");
    const rulesOut = h("button", { class: "btn", onclick: async () => { const s = await app.engine.exportRules(); area.value = s; stamp(rulesOut, await copyText(s)); } }, /* copy:button */ "export");
    const rulesIn = h("button", { class: "btn", onclick: async () => { const t = area.value.trim(); if (!t) { stamp(rulesIn, false); return; } try { await app.setRulesText(t); stamp(rulesIn, true); close(); } catch { stamp(rulesIn, false); } } }, /* copy:button */ "import");

    let armed = 0;
    const reset = h("button", { class: "btn danger", onclick: () => {
      if (Date.now() - armed < 2500) { close(); void app.resetLineage(); return; }
      armed = Date.now(); replace(reset, /* copy:button */ "again"); setTimeout(() => replace(reset, /* copy:button */ "reset"), 2500);
    } }, /* copy:button */ "reset");

    body.append(
      row(/* copy:label */ "save", saveOut, saveIn),
      row(/* copy:label */ "rules", rulesOut, rulesIn),
      area,
      row(/* copy:label */ "engine", h("span", { class: "badge num" }, app.kind === "wasm" ? `wasm ${app.version}` : app.kind)),
      row(/* copy:label */ "lineage", h("span", { class: "num dim" }, `#${app.lineage.seed.toString(16)}`), reset),
    );
    return body;
  });
}
