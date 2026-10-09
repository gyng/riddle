import { classIcon } from './class-icons';
// Active bloodlines only; each row describes an actual Rust-owned hero slot.
import type { App } from '../app';
import type { HeroSlot, LiveRun } from '../engine/types';
import { h, replace, spanOf } from './dom';
import { paintFace } from './frame';
import { openHero } from './town';
import { openSheet, openWindow, closeAllSheets } from './sheet';
import { openChronicle } from './chronicle';
import { kwHost } from './tips';
import { penOpen } from './packages';
import {classStyleName} from './class-styles';
import { heirOrd } from './tokens';
import { openSettings } from './settings';
import { heroPresence, type ObservedPresence } from './hero-presence';
export { heroPresence } from './hero-presence';
export function heroRoster(app:App, hooks:{focus?():void;rules?():void}={}) {
  const el=h('section',{class:'hero-roster','aria-label':/* copy:label */'Active heroes'});
  let key='';
  let sheetBody:HTMLElement|undefined;
  let observed: ObservedPresence | undefined;
  const observedSlot = app.lineage.selected_bloodline ?? 1;
  const focus=async(s:HeroSlot):Promise<void>=>{
    closeAllSheets();if(!await app.selectBloodline(s.id))return;
    if(s.state==='live'){if(app.screen==='watch')window.dispatchEvent(new Event('riddle:focus-hero'));else app.go({kind:'watch'});}
    else if(hooks.focus)hooks.focus();else{app.go({kind:'camp'});requestAnimationFrame(()=>window.dispatchEvent(new Event('riddle:focus-hero')));}
  };
  const details=async(s:HeroSlot,anchor:HTMLElement):Promise<void>=>{if(await app.selectBloodline(s.id)){closeAllSheets();openHero(app,anchor.isConnected?anchor:null);}};
  const row=(s:HeroSlot):HTMLElement=>{
    const face=h('span',{class:'hero-thumb','aria-hidden':'true'});paintFace(face,s.class,64,s.look);
    const presence=heroPresence(s,observed),action=presence.text;
    const xp=s.next===0?/* copy:label */`L${s.level} · MAX`:/* copy:label */`L${s.level} · XP ${s.xp}/${s.next??'—'}`;
    const name=s.hero_name||s.name;
    const progress=s.next===0?1:s.next&&s.next>0?Math.max(0,Math.min(1,s.xp/s.next)):0;
    const body=h('button',{class:'hero-jump','aria-label':`${name} · ${s.name} · ${heirOrd(s.heir)} · ${action}`,'data-hero':s.id,onclick:()=>void focus(s)},face,
      h('span',{class:'hero-info'},h('b',null,name),h('span',{class:'hero-class-name'},classIcon(s.class,s.specialization),`${s.hero_name?s.name:heirOrd(s.heir)} · ${s.specialization?classStyleName(s.specialization):s.class}`),s.build?h('small',{class:'hero-build','data-build':s.build},/* copy:label */s.build):'',h('small',{class:'hero-xp num'},xp),h('span',{class:'hero-xp-track','aria-hidden':'true'},h('i',{style:`width:${progress*100}%`})),
        h('span',{class:'hero-action','data-activity':presence.activity,title:presence.detail},h('i',{'aria-hidden':'true',class:`lane-beat ${s.state}`}),action),s.notice?h('small',{class:'hero-notice'},/* copy:callout */'Upgrade ready'):''));
    kwHost(body,'bloodline');
    return h('article',{class:`hero-row${s.id===app.lineage.selected_bloodline?' selected':''}`,'data-slot':s.id,'data-state':s.state},body,
      h('button',{class:'hero-details','aria-label':/* copy:label */`${s.name} details`,onclick:(e:Event)=>void details(s,e.currentTarget as HTMLElement)},/* copy:button */'Details'));
  };
  const full=():HTMLElement=>{
    const L=app.lineage,slots=L.hero_slots??[];
    const add=h('button',{class:'chip hero-add',disabled:!app.engine.addBloodline||L.town?.home===false||slots.length>=(L.bloodline_cap??1)||L.gold<(L.bloodline_price??250),onclick:(e:Event)=>{
      const button=e.currentTarget as HTMLButtonElement;
      const found=():void=>{
        button.disabled=true;
        if(app.engine.addBloodline)void app.mutate(()=>app.engine.addBloodline!(),/* copy:callout */'Bloodline founded').finally(()=>{
          if(button.isConnected)button.disabled=!app.engine.addBloodline||app.lineage.town?.home===false||(app.lineage.hero_slots?.length??0)>=(app.lineage.bloodline_cap??1)||app.lineage.gold<(app.lineage.bloodline_price??250);
        });
      };
      if(!app.lineage.tree?.auto_send){found();return;}
      openWindow(close=>h('div',{class:'sheet-body bloodline-founding'},
        h('h2',null,/* copy:label */'New bloodline'),
        h('b',{class:'num gold'},`$${app.lineage.bloodline_price??250}`),
        h('p',null,/* copy:callout */'Scout sends immediately'),
        h('div',{class:'chips'},
          h('button',{class:'chip',onclick:()=>{close();openSettings(app);}},/* copy:button */'Scout settings'),
          h('button',{class:'chip bloodline-confirm',onclick:()=>{close();found();}},/* copy:button */'Found'))));
    }},/* copy:button */'New bloodline',h('span',{class:'num gold'},` $${L.bloodline_price??250}`));
    return h('div',{class:'hero-list'},...slots.map(row),L.town?.home===false?h('p',{class:'dim'},/* copy:callout */'Build a house'):'',slots.length<(L.bloodline_cap??1)&&L.town?.home!==false?add:'',
      h('button',{class:'chip hero-history',onclick:()=>openChronicle(app)},/* copy:button */'Chronicle'));
  };
  const sheetContents=()=>[h('div',{class:'label row-label'},/* copy:label */'Active heroes'),full()];
  const openHeroes=():void=>{openSheet(()=>{
    sheetBody=h('div',{class:'sheet-body heroes-sheet'},...sheetContents());return sheetBody;
  },{onClose:()=>{sheetBody=undefined;}});};
  const paint=():void=>{
    const L=app.lineage;const slots=L.hero_slots??[];
    const next=JSON.stringify([slots.map(s=>[s.id,s.name,s.hero_name,s.look,s.heir,s.class,s.specialization,s.level,s.xp,s.next,s.state,s.live?.depth,s.live?.hp,heroPresence(s,observed),spanOf(s.rest_s),s.notice]),L.gold,L.town?.home,L.selected_bloodline]);
    if(next===key)return;key=next;
    const current=slots.find(s=>s.id===L.selected_bloodline);
    replace(el,h('header',{class:'hero-roster-head'},h('h2',null,/* copy:label */'Heroes'),hooks.rules&&penOpen(L)?h('button',{class:'chip',onclick:hooks.rules},/* copy:button */'Rules'):''),
      h('div',{class:'hero-desktop'},full()),
      h('div',{class:'hero-mobile'},current?row(current):'',h('button',{class:'hero-expand game-control',onclick:openHeroes},/* copy:button */'Heroes')));
    if(sheetBody?.isConnected)replace(sheetBody,...sheetContents());
  };
  const off=app.onChange(paint),offLive=app.onLive(paint),timer=window.setInterval(paint,1000);paint();
  return {el,paint,setPresence:(live:LiveRun,ended=false)=>{if(observed&&live.run_id===observed.live.run_id&&live.heir===observed.live.heir&&live.turn<observed.live.turn)return;observed={slot:observedSlot,live,ended};paint();},dispose:()=>{off();offLive();clearInterval(timer);}};
}
