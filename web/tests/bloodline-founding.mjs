import {execFileSync} from 'node:child_process';
import {readFileSync,mkdirSync} from 'node:fs';
import {launchBrowser} from '../../tools/browser.mjs';
const root=new URL('../../',import.meta.url);
const engine=readFileSync(new URL('./fixtures/earned-gunner-home.json',import.meta.url),'utf8');
const shots=process.env.BLOODLINE_SHOTS;if(shots)mkdirSync(shots,{recursive:true});
const url=execFileSync('bash',['tools/dev.sh'],{cwd:root,encoding:'utf8'}).trim();
const browser=await launchBrowser();
try {
 for(const width of [320,400,1440]) {
  const page=await browser.newPage({viewport:{width,height:900}});
  await page.goto(`${url}?dev=1&seed=4103&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
  await page.evaluate(async engine=>{const a=window.__riddle;if(a.kind!=='wasm')throw Error('Real WASM required');await a.importSave(JSON.stringify({v:2,engine,loadout:[],last_seen:Date.now(),runs:0}));},engine);
  const openRoster=async()=>{if(width<1024)await page.locator('.hero-mobile .hero-expand').click();};
  const roster=width<1024?'.sheet-wrap:not([hidden]) .heroes-sheet':'.hero-desktop';
  await openRoster();
  const before=await page.evaluate(async()=>({save:await window.__riddle.engine.save(),gold:window.__riddle.lineage.gold,price:window.__riddle.lineage.bloodline_price}));
  await page.locator(`${roster} .hero-add`).click();
  const founding=page.locator('.bloodline-founding');await founding.waitFor();
  if(!(await founding.innerText()).includes('Scout sends immediately')||!(await founding.innerText()).includes(`$${before.price}`))throw Error('Missing true Scout behavior or actual price');
  if(!await page.evaluate(async saved=>await window.__riddle.engine.save()===saved,before.save))throw Error('Opening founding spends or mutates');
  const box=await founding.boundingBox();if(box.x<0||box.x+box.width>width)throw Error('Founding outside viewport');
  if(shots){await page.waitForTimeout(400);await page.screenshot({path:`${shots}/founding-${width}.png`});}
  await page.keyboard.press('Escape');
  if(!await page.evaluate(async saved=>await window.__riddle.engine.save()===saved,before.save))throw Error('Cancel changed exact save');
  await page.locator(`${roster} .hero-add`).click();await founding.getByRole('button',{name:'Scout settings'}).click();
  const scout=page.locator('.settings .worker-switch[data-worker="scout"]');await scout.waitFor();
  if(await scout.getAttribute('aria-checked')!=='true')throw Error('Route must reach existing on Scout control');
  if(!await page.evaluate(async saved=>await window.__riddle.engine.save()===saved,before.save))throw Error('Reading Scout controls changed save');
  await scout.click();await page.waitForFunction(()=>window.__riddle.lineage.tree.auto_send===false);
  await page.keyboard.press('Escape');await openRoster();
  await page.locator(`${roster} .hero-add`).click();await page.waitForFunction(()=>window.__riddle.lineage.hero_slots.length===2);
  if(await founding.count())throw Error('Scout-off founding should stay direct');
  const off=await page.evaluate(()=>window.__riddle.lineage);
  if(off.gold!==before.gold-before.price||off.hero_slots.find(s=>s.id===2).state!=='waits')throw Error('Scout-off founding pays once and leaves hero Ready');
  await page.evaluate(async()=>{const a=window.__riddle;await a.mutate(()=>a.engine.setWorker('scout',true));});
  await page.locator(`${roster} .hero-add`).click();await founding.waitFor();
  await founding.locator('.bloodline-confirm').click();await page.waitForFunction(()=>window.__riddle.lineage.hero_slots.length===3);
  if(!await page.evaluate(price=>window.__riddle.lineage.gold===price,before.gold-before.price*2))throw Error('Confirmed founding must pay actual price once');
  if(await page.locator(`${roster} .hero-add`).count())throw Error('Full roster must not offer fourth slot');
  if(width<1024&&await page.locator(`${roster} .row-label`).first().textContent()!=='Active heroes')throw Error('Founding lost roster heading');
  console.log(`${width} real-WASM cancel/exact save/Scout controls/off Ready/confirmed price/cap PASS`);await page.close();
 }
} finally {await browser.close();}
