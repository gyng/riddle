// Every catalogue kind renders; semantic item lists retain names/counts and identity knowledge.
import {execFileSync} from 'node:child_process';import {readFileSync} from 'node:fs';
import {launchBrowser} from '../../tools/browser.mjs';import {openPanel} from './lib/frame.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const kinds=[...readFileSync(new URL('../../crates/riddle-core/src/defs.rs',import.meta.url),'utf8').matchAll(/ItemDef \{ kind: "([^"]+)"/g)].map(m=>m[1]);
const browser=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await browser.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const checks=await p.evaluate(async kinds=>{
  const gem=document.querySelector('.gem.send').getBoundingClientRect();if(Math.abs(gem.width-gem.height)>0.5)throw Error('Send artwork stretched');
  const {itemIcon,iconId}=await import('/src/ui/items.ts');const {rowChips}=await import('/src/ui/editor.ts');
  const {openForge}=await import('/src/ui/forge.ts');const {closeAllSheets}=await import('/src/ui/sheet.ts');
  let checks=0;const check=(ok,text)=>{if(!ok)throw Error(text);checks++;};
  for(const kind of [...kinds,'new_unpacked_kind']){const el=itemIcon({kind,label:kind});check(!!el.querySelector('img,.glyph'),`icon/fallback ${kind}`);const pic=el.querySelector('img');if(pic)check((await fetch(pic.src)).ok,`packed image ${kind}`);}
  for(const [label,family] of [['blue potion?','potion'],['brittle scroll?','scroll'],['summon ally scroll','scroll']])check(iconId(label)===`it_${family}`,`visible family only ${label}`);
  check(iconId('sword +2')==='it_sword','enchanted gear keeps icon');
  for(const rarity of ['common','uncommon','rare','epic','legendary']){const el=itemIcon({kind:'sword',label:'sword',rarity});document.body.append(el);const style=getComputedStyle(el);check(style.boxShadow==='none'&&style.backgroundImage==='none'&&style.outlineStyle==='none',`${rarity} silhouette has no box`);el.remove();}

  const read=rowChips({conds:[{k:'item',t:'teleport'}],verb:{v:'read',a:'teleport'}});check(read.querySelectorAll('.item-ico').length===2,'rule item condition/action icons');
  const a=window.__riddle;a.lineage={...a.lineage,forge:{'blue potion?':{salvaged:2,craftable:false,tier:0,next:{need:5,label:'craftable'}}},kit:[{slot:'weapon',owned:0,steps:[{kind:'sword',label:'sword +1',price:100},{kind:'sword',label:'sword +2',price:400}],next:{price:100,affordable:true,label:'sword +1'}}]};
  openForge(a);document.querySelector('.sheet-wrap .forge-details').open=true;
  check(document.querySelector('.sheet-wrap .forge-item .item-ico'),'Forge next item icon');
  check(document.querySelectorAll('.sheet-wrap .forge-ladders .item-ico').length===1,'Forge future item icons');
  const salvage=document.querySelector('.sheet-wrap .salvage .lrow:not(.head)');check(salvage.querySelector('.k .item-ico'),'salvage item icon');check(salvage.textContent.includes('blue potion?'),'unidentified question mark retained');
  closeAllSheets();
  a.go({kind:'report',report:{elapsed_s:0,runs:1,sampled:false,learned:['item:blue=heal'],bests:[],found:[{label:'mystery scroll?'}],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],kept:['sword +1'],shelved:[{kind:'heal',n:2}],salvaged:[{kind:'axe',n:2,gold:60}],spent:[{kind:'heal',n:3,gold:75}],stolen:[{label:'leash',n:1}],exits:[]}});
  const sections=[...document.querySelectorAll('.report .rsec')];
  for(const label of ['found','kept','shelved','salvaged','spent','stolen']){const section=sections.find(s=>s.querySelector('.label')?.textContent===label);check(section?.querySelector('.item-ico'),`report ${label} item icons`);}
  check(document.querySelector('.report .facts .chip.item .item-ico'),'learned identity item icon');
  check(document.querySelector('.report .rsec .item-chip')?.textContent!==undefined,'names remain text');
  check(document.documentElement.scrollWidth<=innerWidth,'report no horizontal overflow');return checks;
 },kinds);console.log(width,checks,'item icon checks PASS');await p.close();
}}finally{await browser.close();}
