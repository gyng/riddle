import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3909`);await p.waitForFunction(()=>window.__riddle?.booted);
 const checks=await p.evaluate(async()=>{
  const a=window.__riddle,{mergeReports}=await import('/src/app.ts'),{mergeClassXp}=await import('/src/ui/class-xp.ts');let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;};
  const base={elapsed_s:0,runs:1,sampled:false,learned:[],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],salvaged:[],xp:{class:'fighter',gained:39,level_ups:0},renown:{gained:0,rank:0,ranks_up:0},gold:{home:143},deepest:4};
  const slot=(id,name,xp)=>({id,name,runs:1,deepest:4,gold:10,packages:[],...(xp===undefined?{}:{xp})});
  const text=()=>document.querySelector('.report-class-xp')?.textContent??'';
  a.go({kind:'report',report:base});check(text().includes('Class XP')&&text().includes('Fighter')&&text().includes('+39 XP'),'first-run earned class XP visible');
  check(!!document.querySelector('.report-class-xp').closest('.report-details [data-group=heroes]'),'XP in the Heroes fold (owner IA pass 2026-10-10)');check(document.querySelector('.report-details').hidden,'details stay folded');
  check(!/Legacy|L\d/.test(text()),'XP distinct from Legacy and current class level');check(document.documentElement.scrollWidth<=innerWidth,'single-class reward fits viewport');
  a.go({kind:'report',report:{...base,xp:{class:'fighter',gained:0,level_ups:0}}});check(!document.querySelector('.report-class-xp'),'zero gain adds no reward row');
  a.go({kind:'report',report:{...base,xp:{class:'rogue',gained:0,level_ups:1}}});check(text().includes('Rogue')&&text().includes('+1 level')&&!text().includes('+0 XP'),'level-only reward visible');
  const report={...base,bloodlines:[slot(2,'Thorn',[{class:'rogue',gained:25,level_ups:2}]),slot(1,'Ash',[{class:'fighter',gained:39,level_ups:0}])]};
  a.go({kind:'report',report});const rows=[...document.querySelectorAll('.class-xp-line')];
  check(rows.length===2&&rows[0].dataset.bloodline==='1','slot order independent of selection');check(rows[0].textContent.includes('Ash')&&rows[0].textContent.includes('Fighter')&&rows[0].textContent.includes('+39 XP'),'first bloodline owns its gain');
  check(rows[1].textContent.includes('Thorn')&&rows[1].textContent.includes('Rogue')&&rows[1].textContent.includes('+25 XP')&&rows[1].textContent.includes('+2 levels'),'other bloodline owns class/levels');
  a.go({kind:'report',report:{...report,xp:{class:'fighter',gained:0,level_ups:0},bloodlines:[slot(1,'Ash',[{class:'fighter',gained:0,level_ups:0}]),report.bloodlines[0]]}});check(document.querySelectorAll('.class-xp-line').length===1&&text().includes('Thorn')&&text().includes('+25 XP'),'unselected gain visible while selected hero waits');
  a.go({kind:'report',report});const before=text();a.lineage.class='mage';a.lineage.selected_bloodline=2;a.emitLive();check(text()===before,'selection/class change cannot rewrite historical growth');
  const next={...base,bloodlines:[slot(1,'Ash',[{class:'fighter',gained:11,level_ups:1},{class:'mage',gained:7,level_ups:0}]),slot(2,'Thorn',[{class:'rogue',gained:5,level_ups:0}])]};
  const merged=mergeReports(report,next);check(JSON.stringify(merged.bloodlines.find(s=>s.id===1).xp)===JSON.stringify([{class:'fighter',gained:50,level_ups:1},{class:'mage',gained:7,level_ups:0}]),'slice merge retains same and changed classes');
  check(merged.bloodlines.find(s=>s.id===2).xp[0].gained===30,'other slot deltas add');check(report.bloodlines[1].xp[0].gained===39,'merge never mutates original report');
  a.go({kind:'report',report:merged});check(text().includes('+50 XP')&&text().includes('Mage')&&text().includes('+7 XP')&&text().includes('+30 XP'),'merged rewards render earned classes');
  check(document.documentElement.scrollWidth<=innerWidth,'multi-class rewards fit viewport');check(mergeClassXp(undefined,undefined)===undefined,'unknown old data stays unknown');
  a.go({kind:'report',report:{...base,bloodlines:[slot(1,'Old',undefined),slot(2,'Other',undefined)]}});check(text().includes('+39 XP')&&!text().includes('Old')&&!text().includes('Other'),'old report uses explicit top-level class without false slot attribution');
  a.go({kind:'report',report:{...base,bloodlines:[slot(1,'Ash',[]),slot(2,'Thorn',[])]}});check(!document.querySelector('.report-class-xp'),'known empty slot reports do not repeat top-level gain');
  return n;
 });if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'class XP/visible rewards/slot identity/slice merges/old wire PASS');await p.close();}}finally{await b.close();}
