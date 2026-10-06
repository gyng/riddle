import '../legacy.css';
import type {App} from '../app';
import type {Lineage} from '../engine/types';
import {h} from './dom';
import {icon} from './skin';
import {detailHost} from './tips';
import {openWindow} from './sheet';

type Upgrade=NonNullable<Lineage['legacy_upgrades']>[number];
const ICONS:Record<string,string>={health:'v_drink',damage:'v_attack',armour:'v_shield',restoration:'v_rest',mending:'v_drink',renewal:'v_shadow',control:'v_shield',venom:'v_throw',debilitate:'v_magic',clear_lungs:'v_explore',fireward:'v_read',brace:'v_corridor'};
/* copy:label */
const ROOT_BRANCH:Record<string,string>={health:'Recovery',damage:'Control',armour:'Warding'};
const respecKey=(L:Lineage):string=>JSON.stringify([L.selected_bloodline,L.bloodline,L.legacy_respec,!!L.live,L.town?.home]);

function reviewRespec(app:App,onChange:()=>void):void {
  const before=app.lineage,asked=respecKey(before),spent=before.legacy_respec?.refund??0;
  openWindow(close=>{
    let busy=false;
    const feedback=h('div',{class:'legacy-feedback',role:'status'});
    const cancel=h('button',{class:'chip legacy-respec-cancel',onclick:()=>{if(!busy)close();}},/* copy:button */'Cancel');
    const confirm=h('button',{class:'chip legacy-respec-confirm',onclick:()=>{
      if(busy||!app.engine.respecLegacy)return;
      if(respecKey(app.lineage)!==asked){confirm.disabled=true;feedback.textContent=/* copy:callout */'Review changed';return;}
      busy=true;confirm.disabled=true;cancel.disabled=true;
      let reason=/* copy:callout */'Respec refused';
      void app.mutate(async()=>{try{return await app.engine.respecLegacy!();}catch(error){reason=error instanceof Error?error.message:reason;throw error;}},/* copy:callout */'Respec').then(ok=>{
        if(ok){close();onChange();return;}
        busy=false;cancel.disabled=false;confirm.disabled=false;feedback.textContent=reason;
      });
    }},/* copy:button */'Reset upgrades');
    return h('div',{class:'sheet-body legacy-respec-review'},
      h('div',{class:'label'},/* copy:label */'Respec Legacy'),
      h('b',null,/* copy:label */`Bloodline ${before.selected_bloodline??1}`),
      h('p',{class:'num'},/* copy:label */`Refund ${spent} Legacy`),
      before.legacy_respec?.points_after!==undefined&&before.legacy_respec.points_after!==null?
        h('p',{class:'num'},/* copy:label */`Total ${before.legacy_respec.points_after} Legacy`):null,
      h('div',{class:'chips'},cancel,confirm),feedback);
  });
}

export function renderLegacy(app:App,onChange:()=>void,opened?:Set<string>):HTMLElement {
  const L=app.lineage,upgrades=L.legacy_upgrades??[],feedback=h('div',{class:'legacy-feedback',role:'status'});
  const card=(u:Upgrade):HTMLElement=>{
    const name=u.name??u.id,chosen=u.cap===1&&u.rank>0;
    const buy=h('button',{class:'chip legacy-buy',disabled:!u.affordable||!app.engine.upgradeHero,'data-upgrade':u.id,
      'aria-label':u.rank>=u.cap?/* copy:label */`${u.id} complete`:/* copy:label */`Upgrade ${u.id}`,
      'aria-description':u.blocked??`${u.price} Legacy`,onclick:(event:Event)=>{
        const button=event.currentTarget as HTMLButtonElement;button.disabled=true;feedback.textContent='';
        if(app.engine.upgradeHero)void app.mutate(()=>app.engine.upgradeHero!(u.id),/* copy:callout */'Upgraded').then(ok=>{
          if(ok)onChange();else{button.disabled=!u.affordable;feedback.textContent=/* copy:callout */'Upgrade refused';}
        });
      }},u.rank>=u.cap?(chosen?/* copy:button */'Chosen':/* copy:button */'Complete'):/* copy:button */`Upgrade ${u.price}`,
      u.rank<u.cap?h('small',null,/* copy:label */'Legacy'):null);
    const title=detailHost(h('span',{class:'legacy-node-name'},h('b',null,name)),()=>[
      h('div',{class:'kw-tip-head'},icon(ICONS[u.id]??'unlocks','✦'),h('b',null,name)),
      h('div',null,u.effect),
      ...(u.parent?[h('div',{class:'dim'},/* copy:label */`Requires ${upgrades.find(x=>x.id===u.parent)?.name??u.parent}`)]:[]),
      ...(u.min_depth?[h('div',{class:'dim'},/* copy:label */`Reach D${u.min_depth}`)]:[]),
      ...(u.blocked?[h('div',{class:'dim'},u.blocked)]:[]),
    ]);
    return h('section',{class:`legacy-upgrade${u.rank?' owned':''}${chosen?' chosen':''}${u.blocked&&!u.rank?' locked':''}`,'data-upgrade':u.id},
      h('span',{class:'legacy-icon'},icon(ICONS[u.id]??'unlocks','✦')),
      h('div',{class:'upgrade-copy'},title,
        u.cap>1?h('small',{class:'upgrade-rank dim num'},/* copy:label */`Rank ${u.rank}/${u.cap}`):null,
        u.owned_effect?h('small',{class:'upgrade-owned num'},u.owned_effect):null,
        u.rank<u.cap?h('small',{class:'upgrade-next num'},u.cap>1?/* copy:label */'Next':'',u.cap>1?' ':'',u.effect):
          u.cap===1?h('small',{class:'upgrade-owned num'},u.effect):null,
        u.blocked&&u.rank<u.cap?h('small',{class:'legacy-blocked dim'},u.blocked):null),buy);
  };
  const branches=[...new Set(upgrades.map(u=>u.branch??ROOT_BRANCH[u.id]??''))];
  const tree=h('div',{class:'legacy-tree legacy-upgrades'});
  for(const branch of branches){
    const nodes=upgrades.filter(u=>(u.branch??ROOT_BRANCH[u.id]??'')===branch),root=nodes.find(u=>!u.parent),children=nodes.filter(u=>!!u.parent);
    const leaves=children.filter(u=>u.min_depth===18),follow=children.filter(u=>u.min_depth!==18);
    const open=opened?opened.has(branch):matchMedia('(min-width:900px)').matches||children.some(u=>u.rank>0);
    tree.append(h('section',{class:'legacy-branch','data-legacy-branch':branch},h('h3',null,branch),root?card(root):null,
      children.length?h('details',{class:'legacy-path',open,'data-branch':branch},h('summary',null,/* copy:button */'More upgrades'),
        ...follow.map(card),h('div',{class:'legacy-fork'},h('small',{class:'legacy-fork-label'},/* copy:label */'Choose one'),...leaves.map(card))):null));
  }
  const respec=h('button',{class:'chip legacy-respec',disabled:!L.legacy_respec?.available||!app.engine.respecLegacy,
    onclick:()=>reviewRespec(app,onChange)},/* copy:button */'Respec');
  return h('div',{class:'legacy-panel'},tree,h('div',{class:'legacy-footer'},respec,feedback));
}
