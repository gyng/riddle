// Report ownership/old-wire checks plus actual earned WASM balance reconciliation.
import {execFileSync} from 'node:child_process';
import {readFileSync,mkdirSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const engine=readFileSync(new URL('./fixtures/earned-gunner-home.json',import.meta.url),'utf8');
const browser=await launchBrowser();
try {for(const width of [320,400,1440]) {
  const page=await browser.newPage({viewport:{width,height:900},reducedMotion:'reduce'}),errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`${url}?fresh=1&seed=52`);await page.waitForFunction(()=>window.__riddle?.booted);
  const result=await page.evaluate(async ({engine,multi})=>{
    const app=window.__riddle;
    if(!await app.importSave(JSON.stringify({v:2,engine,loadout:[],last_seen:Date.now(),runs:0})))throw Error('actual earned import');
    if(multi) {await app.engine.addBloodline();await app.engine.addBloodline();}
    const before=await app.engine.lineage();
    const report=await app.engine.runOfflineQuick(8*3600);
    await app.refresh();
    const after=await app.engine.lineage();
    const delta=after.hero_slots.reduce((n,s)=>n+s.legacy.points-(before.hero_slots.find(p=>p.id===s.id)?.legacy.points??0),0);
    if(!(report.legacy_earned>0 && report.legacy_earned===delta))throw Error(`earned mismatch ${report.legacy_earned} / ${delta}`);
    app.go({kind:'report',report,absence:true});
    return {report,delta};
  },{engine,multi:width===400});
  const actualText=await page.locator('.report-legacy-earned').innerText();
  if(result.report.bloodlines?.length) {
    assert.equal(result.report.bloodlines.reduce((n,s)=>n+(s.legacy_earned??0),0),result.delta);
    for(const slot of result.report.bloodlines)if(slot.legacy_earned>0)assert.match(actualText,new RegExp(`\\+${slot.legacy_earned} Legacy`));
  } else assert.match(actualText,new RegExp(`\\+${result.delta} Legacy`));
  const saved=await page.evaluate(()=>window.__riddle.engine.save());
  if(process.env.RIDDLE_QA_SHOTS) {
    mkdirSync(process.env.RIDDLE_QA_SHOTS,{recursive:true});
    await page.screenshot({path:`${process.env.RIDDLE_QA_SHOTS}/earned-legacy-${width}.png`});
    writeFileSync(`${process.env.RIDDLE_QA_SHOTS}/earned-legacy-${width}.json`,JSON.stringify(result,null,2));
  }
  await page.locator('.report .details-fold').click();await page.locator('.report .details-fold').click();
  assert.equal(await page.evaluate(()=>window.__riddle.engine.save()),saved);
  const checks=await page.evaluate(async report=>{
    const {mergeReports}=await import('/src/app.ts'),{legacyEarnedBlock}=await import('/src/ui/legacy-earned.ts');
    let n=0;const check=(ok,msg)=>{if(!ok)throw Error(msg);n++;};
    const show=r=>window.__riddle.go({kind:'report',report:r,absence:true});
    const rows=()=>[...document.querySelectorAll('.report-legacy-earned .class-xp-line')];
    const old={...report};delete old.legacy_earned;delete old.bloodlines;
    check(legacyEarnedBlock(old)===null,'old report cannot infer gain');
    check(legacyEarnedBlock({...old,legacy_earned:0})===null,'zero gain stays quiet');
    const a={...old,legacy_earned:8,bloodlines:[{id:2,name:'Thorn',runs:1,deepest:8,gold:0,legacy_earned:3},{id:1,name:'Ash',runs:1,deepest:8,gold:0,legacy_earned:5}]};
    const b={...old,legacy_earned:2,bloodlines:[{id:1,name:'Ash',runs:1,deepest:8,gold:0,legacy_earned:2}]};
    const merged=mergeReports(a,b);show(merged);
    check(merged.legacy_earned===10,'aggregate sums known deltas');
    check(rows().length===2 && rows()[0].dataset.bloodline==='1','positive owners sorted');
    check(rows()[0].textContent.includes('Ash') && rows()[0].textContent.includes('+7 Legacy'),'first owner sum');
    check(rows()[1].textContent.includes('Thorn') && rows()[1].textContent.includes('+3 Legacy'),'second owner preserved');
    const text=document.querySelector('.report-legacy-earned').textContent;
    window.__riddle.lineage.selected_bloodline=2;window.__riddle.emitLive();
    check(document.querySelector('.report-legacy-earned').textContent===text,'selection does not rewrite history');
    check(mergeReports(old,old).legacy_earned===undefined,'old merge stays unknown');
    check(mergeReports(old,a).legacy_earned===8,'mixed report uses only known gain');
    show({...a,bloodlines:a.bloodlines.map(s=>({...s,legacy_earned:0}))});
    check(rows().length===0,'known empty owners suppress global fallback');
    show(old);check(rows().length===0,'later old report does not repeat gain');
    return n;
  },result.report);
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
  assert.deepEqual(errors,[]);console.log(width,checks,'Legacy owner/merge/old wire and actual WASM gain',result.delta,'PASS');
  await page.close();
}} finally {await browser.close();}
