// Ascension choices explain their rules; only the reviewed action restarts play.
import type { App, Mounted } from "../app";
import { VARIANTS } from "../engine/types";
import { h, replace } from "./dom";
import { heirOrd } from "./tokens";
import { openWindow } from "./sheet";
import { unitPortrait } from "./unit-icon";

const CHALLENGES: Record<string, { name: string; detail: string }> = /* copy:ending */ {
  no_rest: { name: "No rest", detail: "No dungeon rest · half camp recovery" },
  short_list: { name: "Six rules", detail: "Up to six rules · tactic cards kept" },
  bones_only: { name: "No vault", detail: "Saved gear cleared · vault unavailable" },
  hunted: { name: "Hunted", detail: "Previous nemesis · if still hostile · floor 3+" },
};

export function renderEnding(app: App): Mounted {
  const L = app.lineage, engine = app.engine, selected = L.selected_bloodline ?? 1;
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile" }, h("b", { class: "num" }, n), h("span", { class: "label" }, label));
  const status = h("span", { class: "ascension-status", role: "status" });
  let busy = false;
  const chips = h("div", { class: "chips variants" }, ...VARIANTS.map(v => {
    const choice = CHALLENGES[v];
    return h("button", { class: "chip verb ascension-choice", 'data-variant': v, onclick: () => {
      if (busy) return;
      status.textContent = "";
      openWindow(close => {
        const feedback = h("p", { class: "ascension-status", role: "status" });
        const cancel = h("button", { class: "chip", onclick: close }, /* copy:button */ "Cancel");
        const confirm = h("button", { class: "chip ascension-confirm", onclick: () => {
          if (busy) return;
          if (app.engine !== engine || (app.lineage.selected_bloodline ?? 1) !== selected) { close(); return; }
          busy = true; confirm.disabled = true; cancel.disabled = true;
          for (const b of chips.querySelectorAll("button")) b.disabled = true;
          feedback.textContent = "";
          void app.ascend(v).then(ok => {
            if (ok || !chips.isConnected) return;
            busy = false; confirm.disabled = false; cancel.disabled = false;
            for (const b of chips.querySelectorAll("button")) b.disabled = false;
            status.textContent = feedback.textContent = /* copy:callout */ "Try again";
          }).catch(() => {
            if (!chips.isConnected) return;
            busy = false; confirm.disabled = false; cancel.disabled = false;
            for (const b of chips.querySelectorAll("button")) b.disabled = false;
            status.textContent = feedback.textContent = /* copy:callout */ "Try again";
          });
        } }, /* copy:button */ "Start again");
        return h("div", { class: "sheet-body ascension-review" },
          h("h3", null, choice.name), h("p", null, choice.detail),
          h("p", { class: "num" }, /* copy:label */ `Bloodline ${selected}`),
          h("p", null, /* copy:ending */ "Keeps: Legacy · class XP · training · town · Savings · forge progress"),
          h("p", null, /* copy:ending */ "Restarts: heir · depth · supplies · gear upgrades"),
          (L.hero_slots?.length ?? 0) > 1 ? h("p", null, /* copy:ending */ "Other heroes continue") : null,
          h("p", { class: "num ascension-gold" }, /* copy:label */ `Shared gold: $${app.lineage.gold} → $0`),
          h("div", { class: "chips" }, cancel, confirm), feedback);
      }, { stay: true, onClose: () => chips.querySelector<HTMLButtonElement>(`[data-variant="${v}"]`)?.focus() });
      document.querySelector<HTMLButtonElement>('.ascension-review button')?.focus();
    } }, h("b", null, choice.name), h("span", { class: "ascension-description" }, choice.detail));
  }));
  const progression = L.endgame;
  const numbered = progression && engine.descentOffer && engine.beginDescent;
  const next = h("button", { class:"chip verb descent-choice", onclick: () => reviewDescent(progression?.unlocked ?? 0) },
    h("b", null, /* copy:button */ `Ascension ${progression?.unlocked ?? 0}`),
    h("span", { class:"ascension-description" }, /* copy:ending */ "Harder dungeon · same bloodline"));
  function reviewDescent(initial: number): void {
    if (busy || !progression || !engine.descentOffer) return;
    let closed = false, request = 0, tier = initial, ready = false;
    openWindow(close => {
      const heading = h("h3", null), facts = h("div", {class:"descent-facts"});
      const feedback = h("p", {class:"ascension-status",role:"status"});
      const cancel = h("button", {class:"chip",onclick:close}, /* copy:button */ "Cancel");
      const confirm = h("button", {class:"chip descent-confirm",disabled:true,onclick:() => {
        if (busy || !ready || closed || app.engine!==engine || (app.lineage.selected_bloodline??1)!==selected) return;
        busy=true;confirm.disabled=true;cancel.disabled=true;input.disabled=true;lower.disabled=true;higher.disabled=true;
        next.disabled=true;
        for(const b of chips.querySelectorAll("button"))b.disabled=true;
        void app.beginDescent(tier).then(ok => {
          if(ok||closed||!next.isConnected)return;
          busy=false;confirm.disabled=false;cancel.disabled=false;input.disabled=false;lower.disabled=tier===0;higher.disabled=tier===progression.unlocked;next.disabled=false;
          for(const b of chips.querySelectorAll("button"))b.disabled=false;
          feedback.textContent=/* copy:callout */ "Try again";
        }).catch(() => {
          if(closed||!next.isConnected)return;
          busy=false;confirm.disabled=false;cancel.disabled=false;input.disabled=false;lower.disabled=tier===0;higher.disabled=tier===progression.unlocked;next.disabled=false;
          for(const b of chips.querySelectorAll("button"))b.disabled=false;
          feedback.textContent=/* copy:callout */ "Try again";
        });
      }}, /* copy:button */ "Begin descent");
      const input=h("input",{class:"descent-tier num",type:"number",min:0,max:progression.unlocked,value:initial,"aria-label":/* copy:label */ "Ascension",onchange:() => {
        const value=Number(input.value);
        if(!Number.isInteger(value)||value<0||value>progression.unlocked){input.value=String(tier);return;}
        tier=value;void refresh();
      }});
      const lower=h("button",{class:"chip", "aria-label":/* copy:button */ "Easier dungeon",onclick:() => {if(tier>0){tier--;void refresh();}}}, "−");
      const higher=h("button",{class:"chip", "aria-label":/* copy:button */ "Harder dungeon",onclick:() => {if(tier<progression.unlocked){tier++;void refresh();}}}, "+");
      async function refresh(): Promise<void> {
        const token=++request;ready=false;confirm.disabled=true;input.value=String(tier);lower.disabled=tier===0;higher.disabled=tier===progression!.unlocked;
        heading.textContent=tier===0?/* copy:label */ "Original dungeon":/* copy:label */ `Ascension ${tier}`;
        feedback.textContent=/* copy:label */ "Loading";replace(facts);
        try {
          const offer=await engine.descentOffer!(tier);
          if(closed||token!==request)return;
          replace(facts,
            h("p",{class:"num"}, /* copy:ending */ `Enemy HP +${offer.hp_bonus_percent}% · damage +${offer.attack_bonus_percent}%`),
            ...offer.affixes.map(m=>h("div",{class:"descent-modifier"},h("b",null,m.name),h("span",null,m.effect),h("span",{class:"dim"}, /* copy:label */ "Counter", " · ",m.counter))),
            offer.boss?h("div",{class:"descent-modifier"},h("b",null,/* copy:label */ "Mirror King"),h("span",null,offer.boss.effect),h("span",{class:"dim"},offer.boss.counter)):null,
            offer.elites.length?h("details",{class:"descent-elites"},h("summary",null,/* copy:label */ "Elite enemies"),h("p",null,/* copy:ending */ `One in ${offer.elite_rate_denominator} natural enemies · one ability each`),...offer.elites.map(m=>h("div",{class:"descent-modifier"},h("b",null,m.name),h("span",null,m.effect),h("span",{class:"dim"},m.counter)))):null);
          ready=true;confirm.disabled=false;feedback.textContent="";
        } catch {if(!closed&&token===request)feedback.textContent=/* copy:callout */ "Preview unavailable";}
      }
      void refresh();
      return h("div",{class:"sheet-body descent-review"},heading,
        h("div",{class:"descent-stepper"},lower,input,higher),facts,
        h("p",{class:"num"},/* copy:label */ `Bloodline ${selected} · D1`),
        h("p",null,/* copy:ending */ "Keeps: hero · gold · Legacy · class XP · gear · supplies · training · town"),
        h("p",null,/* copy:ending */ "Restarts: depth · checkpoints"),
        L.ascension?.variant?h("p",null,/* copy:ending */ "Previous challenge rules removed"):null,
        (L.hero_slots?.length??0)>1?h("p",null,/* copy:ending */ "Other heroes continue"):null,
        h("div",{class:"chips"},cancel,confirm),feedback);
    },{stay:true,onClose:() => {closed=true;request++;next.focus();}});
    document.querySelector<HTMLButtonElement>('.descent-review .chips button')?.focus();
  }
  const challenges=numbered?h("details",{class:"ending-challenges"},h("summary",null,/* copy:label */ "Challenge restarts"),chips):chips;
  const el = h("main", { class: "ending" },
    h("img", { class: "title-art", src: `${import.meta.env.BASE_URL}art/title.png`, alt: "" }),
    h("div", { class: "ending-body sheet-body" },
      h("div", { class: "ending-victory" }, unitPortrait('mirror_king', 76), h("h1", null, L.endgame?.tier ? /* copy:callout */ `Ascension ${L.endgame.tier} cleared` : /* copy:callout */ "Dungeon cleared")),
      h("div", { class: "ending-heir num" }, heirOrd(L.heir), " ", h("span", { class: "dim" }, `D${L.best_depth}`)),
      h("div", { class: "tiles" }, tile(`${app.totalRuns()}`, /* copy:label */ "runs"), tile(`${L.graveyard.length}`, /* copy:label */ "deaths"),
        tile(`${L.facts.length}`, /* copy:label */ "facts"), tile(`${L.renown}`, /* copy:label */ "reputation")),
      h("h2", null, /* copy:label */ "Next descent"), numbered?next:null, challenges, status));
  return { el };
}
