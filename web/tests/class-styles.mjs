// Controlled unlock fixtures; real Rust/WASM worker transactions, no earned-play claim.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),browser=await launchBrowser();
try {for(const width of [320,400,1440]) {
 const page=await browser.newPage({viewport:{width,height:900}}),errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.goto(`${url}?fresh=1&seed=3&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 await page.evaluate(async()=>{
  const a=window.__riddle;if(a.kind!=='wasm')throw Error('real WASM required');a.runnerOn=false;
  const {openHero}=await import('/src/ui/town.ts');openHero(a);
 });
 assert.equal(await page.locator('.class-style-panel').count(),0);
 const base=await page.evaluate(async()=>{
  const a=window.__riddle,s=JSON.parse(await a.engine.save()),l=s.lineage;
  l.town.home=true;l.class='fighter';l.classes.fighter={level:9,xp:123,next:0};l.best_depth=23;l.hero_legacy.at(-1).best_depth=23;
  l.unlocks.push('caster');l.classes.caster={level:10,xp:234,next:0};
  window.styleFixture=s;
  window.showStyle=async()=>{a.lineage=await a.engine.load(JSON.stringify(window.styleFixture));a.adoptSets();await a.afterLineage();const{closeAllSheets}=await import('/src/ui/sheet.ts');closeAllSheets();const{openHero}=await import('/src/ui/town.ts');openHero(a);};
  await window.showStyle();return a.engine.save();
 });
 assert.ok(await page.locator('.class-style-choose').isDisabled());assert.match(await page.locator('.class-style-panel').textContent(),/reach class level10/);
 await page.keyboard.press('Tab');await page.locator('.class-style-title').focus();await page.locator('#kw-tip:not([hidden])').waitFor();assert.match(await page.locator('#kw-tip').textContent(),/counter 8/);await page.keyboard.press('Escape');
 await page.evaluate(async()=>{window.styleFixture.lineage.classes.fighter.level=10;window.styleFixture.lineage.best_depth=22;window.styleFixture.lineage.hero_legacy.at(-1).best_depth=22;await window.showStyle();});
 assert.ok(await page.locator('.class-style-choose').isDisabled());assert.match(await page.locator('.class-style-panel').textContent(),/reach D23/);
 await page.evaluate(async()=>{window.styleFixture.lineage.best_depth=23;await window.showStyle();});
 const ready=await page.evaluate(()=>window.__riddle.engine.save());
 await page.locator('.class-style-choose').click();assert.equal(await page.evaluate(()=>window.__riddle.engine.save()),ready);
 assert.match(await page.locator('.class-style-review').textContent(),/Automatic tactic/);await page.locator('.class-style-review button').first().click();
 assert.equal(await page.evaluate(()=>window.__riddle.engine.save()),ready);
 // Stale review checks the actual updated parent choice; no engine mutation by confirmation.
 await page.locator('.class-style-choose').click();await page.evaluate(async()=>{await window.__riddle.setClass('caster');});
 const switched=await page.evaluate(()=>window.__riddle.engine.save());await page.locator('.class-style-confirm').click();
 assert.equal(await page.locator('.class-style-feedback').textContent(),'Review changed');assert.equal(await page.evaluate(()=>window.__riddle.engine.save()),switched);
 await page.locator('.class-style-review button').first().click();await page.evaluate(async()=>window.showStyle());
 // Real worker refusal: run starts after review without changing the UI's cached review.
 await page.locator('.class-style-choose').click();await page.evaluate(async()=>window.__riddle.engine.send());
 const away=await page.evaluate(()=>window.__riddle.engine.save());await page.locator('.class-style-confirm').click();
 await page.waitForFunction(()=>document.querySelector('.class-style-feedback')?.textContent.includes('hero away'));
 assert.equal(await page.evaluate(()=>window.__riddle.engine.save()),away);await page.locator('.class-style-review button').first().click();
 await page.evaluate(async()=>window.showStyle());
 // Double activation is one actual worker call; adopting the compiled set exposes the named row.
 await page.evaluate(()=>{const a=window.__riddle,call=a.engine.setSpecialization.bind(a.engine);window.styleCalls=0;a.engine.setSpecialization=async id=>{window.styleCalls++;return call(id);};});
 await page.locator('.class-style-choose').click();await page.locator('.class-style-confirm').evaluate(el=>{el.click();el.click();});
 await page.waitForFunction(()=>window.__riddle.lineage.class_styles.selected==='sentinel');
 assert.equal(await page.evaluate(()=>window.styleCalls),1);assert.ok(await page.evaluate(()=>window.__riddle.rules.rows.some(r=>r.origin==='style:sentinel')));
 const chosen=await page.evaluate(async()=>{const a=window.__riddle;await a.flush();return a.engine.save();});
 await page.goto(`${url}?seed=3&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 assert.equal(await page.evaluate(()=>window.__riddle.engine.save()),chosen);
 await page.evaluate(async()=>{const a=window.__riddle;a.runnerOn=false;const{openHero}=await import('/src/ui/town.ts');openHero(a);});
 assert.match(await page.locator('.class-style-choose').textContent(),/Remove path/);
 await page.locator('.class-style-choose').click();assert.match(await page.locator('.class-style-review').textContent(),/Tactic removed/);await page.locator('.class-style-confirm').click();
 await page.waitForFunction(()=>window.__riddle.lineage.class_styles.selected===null);
 assert.ok(await page.evaluate(()=>!window.__riddle.rules.rows.some(r=>r.origin==='style:sentinel')));
 assert.equal(await page.evaluate(()=>window.__riddle.lineage.classes.fighter.xp),123);
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 assert.ok(await page.evaluate(()=>{const h=document.querySelector('.hero-line');return h.scrollWidth<=h.clientWidth&&[...h.children].every(c=>c.getBoundingClientRect().right<=h.getBoundingClientRect().right+1);}));assert.deepEqual(errors,[]);
 console.log(width,'real-worker gates/tooltip/review/cancel/stale/refusal/one-call/compiled row/reload/remove/XP PASS');await page.close();
}} finally {await browser.close();}
