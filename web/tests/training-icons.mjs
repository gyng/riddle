import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{const {trainingBlock}=await import('/src/ui/tracks.ts');const host=document.createElement('div');host.style.width='min(100%,600px)';host.id='training-check';host.appendChild(trainingBlock(['BOLD L2','LIGHT HANDS L3','BOSS FOCUS L2','DRILLED · Warlord','DRILLED · Mother','DRILLED · Lich','DRILLED · Master','DRILLED · Queen','DRILLED · King','DRILLED · future boss','A VERY LONG FUTURE STYLE WITH UNBROKENNAMEABCDEFGHIJKLMNOPQRSTUV L2']));document.body.replaceChildren(host);});
 await p.evaluate(async()=>{await document.fonts.ready;await Promise.all([...document.querySelectorAll('#training-check img')].map(i=>i.decode()));});
 const badge=beat=>p.locator(`[data-training="${beat}"]`);
 assert.match(await badge('BOLD L2').locator('img').getAttribute('src'),/pkg_bold_v7/);
 assert.match(await badge('LIGHT HANDS L3').locator('img').getAttribute('src'),/pkg_light_hands_v7/);
 assert.match(await badge('BOSS FOCUS L2').locator('img').getAttribute('src'),/v_attack/);
 for(const [short,kind] of Object.entries({Warlord:'goblin_warlord',Mother:'bloat_mother',Lich:'lich',Master:'foundry_master',Queen:'lurker_queen',King:'mirror_king'})){
  assert.equal(await badge(`DRILLED · ${short}`).locator('.unit-icon').getAttribute('data-unit'),kind);
  assert.equal(await badge(`DRILLED · ${short}`).textContent(),`${short} tactic`);
 }
 assert.equal(await badge('DRILLED · future boss').locator('.unit-icon').count(),0);
 assert.match(await badge('DRILLED · future boss').locator('img').getAttribute('src'),/unlocks/);
 for(const el of await p.locator('.beat-plaque').all()){
  const box=await el.boundingBox(),art=await el.locator(':scope > :first-child').boundingBox(),text=await el.locator('.unit-label-text').boundingBox();
  assert.equal(art.width,32);assert.equal(art.height,32);assert.ok(Math.abs(text.x-art.x-art.width-8)<1);
  assert.ok(Math.abs(text.y+text.height/2-art.y-art.height/2)<1);
  assert.ok(box.x>=0&&box.x+box.width<=width);assert.equal(await el.locator(':scope > :first-child').getAttribute('aria-hidden'),'true');
 }
 assert.deepEqual(errors,[]);assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 console.log(width,'training art/text/fallback/gap/centering/wrapping PASS');await p.close();
}}finally{await b.close();}
