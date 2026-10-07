import {execFileSync} from 'node:child_process';
import {mkdirSync} from 'node:fs';
import {launchBrowser} from '../../tools/browser.mjs';
const shots=process.env.CLASS_LOCK_SHOTS;if(shots)mkdirSync(shots,{recursive:true});
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try {
 for(const width of [320,400,1440]) {
  const page=await browser.newPage({viewport:{width,height:800}});
  await page.goto(`${url}?dev=1&engine=fake&seed=3004&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
  await page.evaluate(async()=>{
   const a=window.__riddle;
   a.lineage.marks=32;a.lineage.live=undefined;
   const cat=await a.engine.unlocks();
   a.engine.unlocks=async()=>cat.map(u=>u.id==='caster'?{...u,owned:false,available:false,needs:'meet Lich'}:u.id==='ranger'?{...u,owned:false,available:false,needs:'◆2 more'}:u.id==='rogue'?{...u,owned:false,available:true,needs:undefined}:u);
   a.lineage.unlocks=a.lineage.unlocks.filter(x=>!['rogue','ranger','caster'].includes(x));
   window.__classLockSave=await a.engine.save();
   const {openHeroClass}=await import('/src/ui/town.ts');openHeroClass(a);
  });
  const caster=page.locator('.class-row').filter({has:page.getByRole('button',{name:/caster L/})});
  await caster.locator('.class-needs').waitFor();
  if(!(await caster.locator('.class-needs').innerText()).includes('meet Lich')||!await caster.getByRole('button').first().isDisabled())throw Error('Enough tokens must not hide the missing class milestone');
  const ranger=page.locator('.class-row').filter({has:page.getByRole('button',{name:/ranger L/})});
  if(!(await ranger.locator('.class-needs').innerText()).includes('◆2 more'))throw Error('Token shortage must show the core reason');
  const rogue=page.locator('.class-row').filter({has:page.getByRole('button',{name:/rogue L/})});
  if(await rogue.locator('.class-needs').count()||!await rogue.getByRole('button').first().isEnabled())throw Error('Buyable class has no false lock');
  const fighter=page.locator('.class-row').filter({has:page.getByRole('button',{name:/fighter L/})});
  if(await fighter.locator('.class-needs').count())throw Error('Owned class has no false lock');
  const bounds=await caster.locator('.class-needs').boundingBox();if(bounds.x<0||bounds.x+bounds.width>width)throw Error('Class requirement outside viewport');
  if(shots){await page.waitForTimeout(400);await page.screenshot({path:`${shots}/class-locks-${width}.png`});}
  await page.keyboard.press('Escape');
  if(!await page.evaluate(async()=>await window.__riddle.engine.save()===window.__classLockSave))throw Error('Reading class locks changed save');
  await page.close();console.log(`${width} gate/token-short/available/owned/viewport/exact-save PASS`);
 }
} finally {await browser.close();}
