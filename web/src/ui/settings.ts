// Settings sheet: save export/import, rules export/import, engine badge, reset lineage (double tap).
// Cut 10 §4: `sound` row with a `mute` toggle (persisted in localStorage; no cue and no drone while muted).
// docs/UI.md §7: `auto continue` row — `on` / `off` (localStorage `riddle.autoContinue`).
// Cut 14: `slowdowns` row — `on` / `off` (`app.slowdowns`, persisted): the watch's fight / near / scene holds, or the flat rate.
import type { App } from "../app";
import { h, copyText, replace } from "./dom";
import { closeEverything, openSheet } from "./sheet";
import { audio } from "../audio";
import { autoOn, setAutoOn } from "./autodismiss";

export function openSettings(app: App): void {
  closeEverything();   // QA 23ed91f (L: the settings sheet opened over the open UNLOCKS panel — two studs): one at a time
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

    const mute = h("button", { class: `btn mute${audio.muted ? " on" : ""}`, "aria-pressed": audio.muted ? "true" : "false", onclick: () => {
      audio.unlock(); audio.setMuted(!audio.muted); mute.classList.toggle("on", audio.muted); mute.setAttribute("aria-pressed", audio.muted ? "true" : "false");
    } }, /* copy:button */ "mute");

    const slow = h("button", { class: `btn slowdowns${app.slowdowns ? " on" : ""}`, "aria-pressed": app.slowdowns ? "true" : "false", onclick: () => {
      app.setSlowdowns(!app.slowdowns); slow.classList.toggle("on", app.slowdowns); slow.setAttribute("aria-pressed", app.slowdowns ? "true" : "false"); replace(slow, app.slowdowns ? /* copy:button */ "on" : /* copy:button */ "off");
    } }, app.slowdowns ? /* copy:button */ "on" : /* copy:button */ "off");

    // docs/UI.md §7: the report, the death screen and open panels continue on their own (on by default)
    const auto = h("button", { class: `btn auto-continue${autoOn() ? " on" : ""}`, "aria-pressed": String(autoOn()), onclick: () => {
      setAutoOn(!autoOn()); auto.classList.toggle("on", autoOn()); auto.setAttribute("aria-pressed", String(autoOn())); replace(auto, autoOn() ? /* copy:button */ "on" : /* copy:button */ "off");
    } }, autoOn() ? /* copy:button */ "on" : /* copy:button */ "off");

    body.append(
      row(/* copy:label */ "sound", mute),
      row(/* copy:label */ "slowdowns", slow),
      row(/* copy:label */ "auto continue", auto),
      row(/* copy:label */ "save", saveOut, saveIn),
      row(/* copy:label */ "rules", rulesOut, rulesIn),
      area,
      row(/* copy:label */ "engine", h("span", { class: "badge num" }, app.kind === "wasm" ? `wasm ${app.version}` : app.kind)),
      // the seed in decimal (both QA players on 952e306 decoded `#d3` as 211 in hex)
      row(/* copy:label */ "lineage", h("span", { class: "num dim" }, /* copy:label */ `seed ${app.lineage.seed}`), reset),
    );
    return body;
  });
}
