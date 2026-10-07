import {execFileSync} from 'node:child_process';
import {mkdirSync} from 'node:fs';
const shots=process.env.RULES_LOCK_SHOTS; if(shots)mkdirSync(shots,{recursive:true});
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try {
 for(const width of [320,400,1440]) {
  const page=await browser.newPage({viewport:{width,height:800}});
  await page.goto(`${url}?dev=1&engine=fake&seed=3004&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
  await page.evaluate(async()=>{
   const a=window.__riddle;
   a.lineage.packages.pen_open=false;
   a.lineage.packages.pen_needs=['Meet Bloat Mother','72h elapsed','or 120h elapsed'];
   a.go({kind:'camp'});
   window.__rulesLockSave=await a.engine.save();
   const {openPackages}=await import('/src/ui/packages.ts');openPackages(a);
  });
  await page.locator('.pkg-advanced > summary').click();
  await page.locator('.pkg-rows-btn').click();
  if(!(await page.locator('.pkg-rule-lock').innerText()).includes('Read only'))throw Error('Expanded rules must explain read-only status');
  await page.locator('.pkg-pen-lock').click();
  const body=page.locator('.pkg-pen-needs');await body.waitFor();
  const text=await body.innerText();
  for(const wanted of ['Custom rules','Meet Bloat Mother','72h elapsed','or 120h elapsed'])if(!text.includes(wanted))throw Error(`Missing core requirement ${wanted}`);
  const bounds=await body.evaluate(el=>{const r=el.closest('.sheet').getBoundingClientRect();return {left:r.left,right:r.right,center:(r.left+r.right)/2,width:innerWidth}});
  if(bounds.left<0||bounds.right>width||Math.abs(bounds.center-width/2)>2)throw Error('Lock explanation must be centered and in viewport');
  if(await page.locator('.editor:visible').count())throw Error('Lock explanation exposed editor');
  if(shots){await page.waitForTimeout(400);await page.screenshot({path:`${shots}/rules-lock-${width}.png`});}
  await page.keyboard.press('Escape');await body.waitFor({state:'detached'});
  if(!await page.evaluate(async()=>await window.__riddle.engine.save()===window.__rulesLockSave))throw Error('Reading lock changed save');
  await page.keyboard.press('Escape');
  await page.evaluate(async()=>{const a=window.__riddle; a.lineage.packages.pen_open=true;const {openPackages}=await import('/src/ui/packages.ts');openPackages(a)});
  await page.locator('.pkg-advanced > summary').click();
  await page.locator('.pkg-rows-btn').click();
  if(await page.locator('.pkg-pen-lock').count())throw Error('Open custom rules must not show lock');
  await page.close();console.log(`${width} read-only/requirements/center/Escape/exact-save/open PASS`);
 }
} finally {await browser.close();}
