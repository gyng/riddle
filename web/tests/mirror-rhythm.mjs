// Real earned clear; choose the optional counter through the visible Tactics UI.
import {execFileSync} from 'node:child_process';import {readFileSync} from 'node:fs';import assert from 'node:assert/strict';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const source=readFileSync(new URL('./fixtures/earned-first-ascension-clear.json',import.meta.url),'utf8'),native=JSON.parse(source),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900},reducedMotion:'reduce'}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?dev=1&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 assert.equal(await p.evaluate(()=>window.__riddle.kind),'wasm');
 const locked=await p.evaluate(()=>window.__riddle.lineage.packages.all.find(p=>p.id==='cadence'));assert.equal(locked.owned,false);assert.equal(locked.trigger,'Clear dungeon');
 assert.equal(await p.evaluate(async({source,loadout})=>window.__riddle.importSave(JSON.stringify({v:2,engine:source,loadout,last_seen:Date.now(),runs:359})),{source,loadout:native.loadout}),true);
 await p.locator('.ending-town').click();await p.waitForSelector('.camp');
 await p.evaluate(()=>window.__riddle.runnerOn=false);const before=await p.evaluate(()=>window.__riddle.engine.save());
 assert.equal(await p.evaluate(()=>window.__riddle.lineage.packages.all.find(p=>p.id==='cadence').owned),true);
 await p.locator('.cmd .tile[data-tile="packages"]').click();await p.waitForSelector('.pkg-panel');
 await p.locator('.pkg-sec[data-kind="tactic"] .pkg-change').first().click();
 const choice=p.locator('button.pkg.alt[data-pkg="cadence"]');await choice.waitFor({state:'visible'});
 assert.match(await choice.textContent(),/Mirror rhythm L1.*Alternate attacks against mirrors/s);
 assert.equal(await choice.locator('.icon-socket img').count(),1);assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before);
 await choice.click();await p.waitForFunction(()=>window.__riddle.lineage.packages.tactics?.includes('cadence'));
 const chosen=await p.evaluate(()=>({ended:window.__riddle.lineage.ended,points:window.__riddle.lineage.bloodline.points,gold:window.__riddle.lineage.gold,rows:window.__riddle.rules.rows.filter(r=>r.origin==='tactic:cadence')}));
 assert.equal(chosen.ended,true);assert.equal(chosen.points,native.lineage.bloodline.points);assert.equal(chosen.gold,native.lineage.gold);
 assert.equal(chosen.rows.length,1);assert.equal(chosen.rows[0].conds[0].t,'mirror');assert.equal(chosen.rows[0].conds[1].k,'hp>');
 await p.keyboard.press('Escape');await p.locator('.gem.send').click();await p.locator('.descent-choice').click();await p.locator('.descent-confirm:not([disabled])').waitFor();
 await p.keyboard.press('Escape');assert.equal(await p.evaluate(()=>window.__riddle.lineage.ended),true);
 await p.evaluate(()=>window.__riddle.flush());await p.reload();await p.waitForSelector('.ending-town');
 assert.equal(await p.evaluate(()=>window.__riddle.lineage.packages.tactics?.includes('cadence')),true);
 assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);assert.deepEqual(errors,[]);
 console.log(width,'real first-clear lock/earned availability, visible icon/equip, no currency change, one guarded row, cancel/reload PASS');await p.close();
}}finally{await b.close();}
