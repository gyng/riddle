import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const n=await p.evaluate(async()=>{
  const a=window.__riddle;let n=0;const check=(ok,msg)=>{if(!ok)throw Error(msg);n++;};
  const base={elapsed_s:0,runs:1,sampled:false,learned:[],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],salvaged:[],renown:{gained:0,rank:0,ranks_up:0},gold:{home:10}};
  const catalogue=[{id:'bold',name:'Bold',kind:'stance'},{id:'boss_focus',name:'boss focus',kind:'tactic'},{id:'kite_archers',name:'kite archers',kind:'tactic'}].map(p=>({...p,owned:true,level:1,runs:0,description:'Known tactic'}));
  a.lineage.packages={...a.lineage.packages,all:catalogue,stance:'bold',tactics:[],tactic_slots:1,literal:false,pen_open:false};
  const slot={...(a.lineage.hero_slots?.[0]??{}),id:1,name:'Ash'};a.lineage.hero_slots=[slot];a.lineage.selected_bloodline=1;
  let equips=0,selected;a.engine.equipPackage=async()=>{equips++;return a.lineage;};a.selectBloodline=async id=>{selected=id;a.lineage.selected_bloodline=id;return true;};
  const show=r=>a.go({kind:'report',report:{...base,...r}}),text=()=>document.querySelector('.report-choices').textContent;
  show({packages:['+Bold','+boss focus','+kite archers','+boss focus','STEADY L3','DRILLED · Warlord','+3 more','+unknown']});
  check(document.querySelectorAll('.report-choice').length===3,'only explicit known choices, deduplicated');check(!document.querySelector('.report-choices').closest('.report-details')&&document.querySelector('.report-details').hidden,'earned choices outside folded details');
  check(text().includes('Combat style')&&text().includes('Extra tactic')&&!/L\d|Legacy/.test(text()),'kind clear without invented levels');check(document.querySelectorAll('.report-choice .ico').length===3,'every choice has an icon');
  check(!equips,'return never auto-equips');check(document.documentElement.scrollWidth<=innerWidth,'choices fit viewport');
  check([...document.querySelectorAll('.report-choice>.icon-socket')].every(e=>Math.round(e.getBoundingClientRect().width)===44),'icons retain readable44px size');
  const originalButton=document.querySelector('button.report-choice');a.emitLive();check(document.querySelector('button.report-choice')===originalButton,'routine live refresh preserves button DOM');
  document.querySelector('[data-choice="boss_focus"]').click();await new Promise(r=>setTimeout(r,100));
  check(selected===1,'single report opens sole active slot');check(document.querySelector('.pkg-sec[data-kind="tactic"] .pkg-choices')?.hidden===false,'new tactic opens its alternatives directly');check(!equips,'opening choices never equips');
  (await import('/src/ui/sheet.ts')).closeAllSheets();
  show({packages:['STEADY L3','DRILLED · Warlord']});check(document.querySelector('.report-choices').hidden,'no new choices from training/current ownership');
  show({packages:['+boss focus'],bloodlines:[{id:1,name:'Ash',runs:1,deepest:4,gold:10,packages:[]}]});check(document.querySelector('.report-choices').hidden,'known empty slot data suppresses top-level duplicate');
  a.lineage.hero_slots=[slot,{...slot,id:2,name:'Thorn'}];a.lineage.selected_bloodline=1;
  const report={bloodlines:[{id:2,name:'Thorn',runs:1,deepest:8,gold:10,packages:['+boss focus']},{id:1,name:'Ash',runs:1,deepest:8,gold:10,packages:['+boss focus','+Bold']}]};
  show(report);check(document.querySelectorAll('.report-choice').length===3,'same choice kept separately per owner');check(text().includes('Ash')&&text().includes('Thorn'),'reported owners visible');
  const before=text();a.lineage.class='mage';a.lineage.selected_bloodline=2;a.emitLive();check(text()===before,'selection cannot rewrite historical reward');
  document.querySelector('[data-choice="bold"][data-owner="1"]').click();await new Promise(r=>setTimeout(r,100));check(selected===1,'action selects recorded owner');check(document.querySelector('.pkg-sec[data-kind="stance"] .pkg-choices')?.hidden===false,'style reward opens style alternatives');check(!equips,'multihero action never equips');
  (await import('/src/ui/sheet.ts')).closeAllSheets();
  a.selectBloodline=async()=>{a.lineage.selected_bloodline=2;return true;};show(report);
  document.querySelector('[data-choice="bold"][data-owner="1"]').click();await new Promise(r=>setTimeout(r,100));check(!document.querySelector('.pkg-panel'),'selection superseded before opening cannot target wrong owner');
  show({packages:['+boss focus']});check(text().includes('boss focus')&&!document.querySelector('.report-choice button')&&!document.querySelector('button.report-choice'),'ambiguous old multihero reward is display only');
  show(report);a.lineage.hero_slots=[slot];a.emitLive();check(!document.querySelector('button[data-owner="2"]')&&text().includes('Thorn'),'missing owner keeps earned reward without wrong action');
  check(document.documentElement.scrollWidth<=innerWidth,'multihero reward fits viewport');return n;
 });if(errors.length)throw Error(errors.join('\n'));console.log(width,n,'earned choice visibility/ownership/direct sections/no auto-equip PASS');await p.close();
}}finally{await b.close();}
