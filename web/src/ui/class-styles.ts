import '../class-styles.css';
import type {App} from '../app';
import type {ClassStyleId,ClassStyleOffer,Lineage} from '../engine/types';
import {h} from './dom';
import {icon} from './skin';
import {detailHost} from './tips';
import {openWindow} from './sheet';
import {rowLabel} from './tokens';

const ICONS:Record<ClassStyleId,string>={sentinel:'style_sentinel',hexbinder:'style_hexbinder'};
/* copy:label */
const NAMES:Record<ClassStyleId,string>={sentinel:'Sentinel',hexbinder:'Hexbinder'};
export const classStyleName=(id?:ClassStyleId):string=>id?NAMES[id]:'';
const choiceKey=(L:Lineage):string=>JSON.stringify([L.selected_bloodline,L.class,L.class_styles,!!L.live,L.town?.home]);

function preview(L:Lineage,o:ClassStyleOffer):HTMLElement {
  return h('div',{class:'class-style-preview'},
    h('b',null,o.effect),
    h('small',{class:'num'},/* copy:label */`Duration ${o.duration_ticks/10}s`, ' · ',/* copy:label */`Cooldown ${o.cooldown_ticks/10}s`),
    h('small',null,rowLabel(o.tactic)),
    h('small',{class:'dim'},L.class_styles?.automatic_row?/* copy:label */'Automatic tactic':/* copy:label */'Written rules'),
    L.class_styles?.player_overrides?h('small',{class:'dim'},/* copy:callout */'Written rules first'):null);
}
function review(app:App,o:ClassStyleOffer,remove:boolean,onChange:()=>void):void {
  const before=app.lineage,asked=choiceKey(before);
  openWindow(close=>{
    let busy=false;
    const feedback=h('div',{class:'class-style-feedback',role:'status'});
    const cancel=h('button',{class:'chip',onclick:()=>{if(!busy)close();}},/* copy:button */'Cancel');
    const confirm=h('button',{class:'chip class-style-confirm',onclick:()=>{
      if(busy||!app.engine.setSpecialization)return;
      if(choiceKey(app.lineage)!==asked){confirm.disabled=true;feedback.textContent=/* copy:callout */'Review changed';return;}
      busy=true;confirm.disabled=true;cancel.disabled=true;
      let reason=/* copy:callout */'Choice refused';
      void app.mutate(async()=>{try{return await app.engine.setSpecialization!(remove?'none':o.id);}catch(error){reason=error instanceof Error?error.message:reason;throw error;}},remove?/* copy:callout */'Path removed':o.name,true).then(ok=>{
        if(ok){close();onChange();return;}
        busy=false;cancel.disabled=false;confirm.disabled=false;feedback.textContent=reason;
      });
    }},remove?/* copy:button */'Remove path':/* copy:button */'Choose path');
    return h('div',{class:'sheet-body class-style-review'},
      h('div',{class:'label'},remove?/* copy:label */'Remove path':/* copy:label */'Class path'),
      h('div',{class:'class-style-heading'},icon(ICONS[o.id],'✦'),h('b',null,o.name)),
      h('small',{class:'num'},/* copy:label */`Bloodline ${before.selected_bloodline??1}`),
      remove?h('p',null,/* copy:label */'Tactic removed'):preview(before,o),
      h('p',{class:'num'},/* copy:label */'Free choice'),
      h('div',{class:'chips'},cancel,confirm),feedback);
  });
}
export function renderClassStyles(app:App,onChange:()=>void):HTMLElement|null {
  const L=app.lineage,c=L.class_styles,o=c?.offers.find(o=>o.parent===L.class);
  // Keep the empty-town/first-send experience small; expose later class progress.
  if(!c||!o||L.town?.home===false||o.level<8&&o.deepest<18)return null;
  const info=detailHost(h('span',{class:'class-style-title'},h('b',null,o.name)),()=>[
    h('div',{class:'kw-tip-head'},icon(ICONS[o.id],'✦'),h('b',null,o.name)),preview(L,o),
    h('small',{class:'num'},/* copy:label */`${o.parent} L${o.required_level} · D${o.required_depth}`),
    ...(o.blocked?[h('small',{class:'dim'},o.blocked)]:[]),
  ]);
  return h('section',{class:`class-style-panel${o.selected?' chosen':''}`,'data-style':o.id},
    h('div',{class:'label'},/* copy:label */'Class path'),
    h('div',{class:'class-style-card'},h('span',{class:'class-style-icon'},icon(ICONS[o.id],'✦')),
      h('div',{class:'class-style-copy'},info,h('small',{class:'num'},/* copy:label */`${o.parent} L${o.level}/${o.required_level} · D${Math.min(o.deepest,o.required_depth)}/${o.required_depth}`),
        !o.selected? h('small',{class:'num'},/* copy:label */'Class XP',` ${o.xp}${o.next?` / ${o.next}`:''}`):null,
        preview(L,o),!o.selected&&o.blocked?h('small',{class:'dim'},o.blocked):null),
      h('button',{class:'chip class-style-choose',disabled:!(o.selected?c.remove_available:o.available)||!app.engine.setSpecialization,
        onclick:()=>review(app,o,o.selected,onChange)},o.selected?/* copy:button */'Remove path':/* copy:button */'Choose path')));
}
