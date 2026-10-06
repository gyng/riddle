import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
const checks=await p.evaluate(async()=>{const a=window.__riddle;a.runnerOn=false;let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;};
a.lineage={...a.lineage,live:null,class_offer:[{class:'fighter',signature:'shield_bash',level:1,opens:1},{class:'rogue',signature:'vanish',level:1,opens:1}],unlocks:[...a.lineage.unlocks,'rogue'],systems:[{id:'automations',open:false,new:false,trigger:'meet Lich'}]};a.go({kind:'camp'});
const before=await a.engine.save();check(!document.querySelector('.topbar .classes-offer,.topbar .cls-offer'),'class bar removed even with offers');
const {openHero}=await import('/src/ui/town.ts');openHero(a);check(!document.querySelector('.hero-class').disabled,'class change available in Details');document.querySelector('.hero-class').click();check(!!document.querySelector('.sheet .classes'),'class picker opens');
const {closeAllSheets}=await import('/src/ui/sheet.ts');closeAllSheets();
const {nextPill}=await import('/src/ui/works.ts');let focus='unseen';const node=(id,state)=>({id,name:id,kind:'worker',state}),W={nodes:[node('porter','done'),node('scout','done')],next:{kind:'system',text:'automations · meet Lich',have:1,need:3}},stub={lineage:{tree:W}},pill=nextPill(stub,id=>focus=id);document.body.append(pill.el);
check(pill.el.textContent.includes('Workers · 2 hired'),'fallback shows actual workers');check(!pill.el.querySelector('.pill-bar'),'no invented progress bar');pill.el.click();check(focus===undefined,'no unknown node focus');
W.nodes.push(node('armourer','done'));pill.paint();check(pill.el.textContent.includes('3 hired'),'count repaints');
W.next={kind:'buy',node:'porter',text:'hire porter',have:3,need:3};pill.paint();check(pill.el.textContent.includes('hire porter')&&pill.el.dataset.node==='porter','worker goal retained');pill.el.click();check(focus==='porter','correct worker focus');
for(const kind of ['send','chest']){W.next={kind,text:`actual ${kind}`,have:1,need:3};pill.paint();check(pill.el.textContent.includes(`actual ${kind}`)&&!!pill.el.querySelector('.pill-bar'),`${kind} goal retained`);}
W.nodes=[];W.next={kind:'system',text:'actual unlock'};pill.paint();check(pill.el.textContent.includes('actual unlock'),'early fallback retained');
W.nodes=[node('porter','done')];W.next={kind:'none',text:''};pill.paint();check(pill.el.textContent.includes('Workers · 1 hired'),'finished tree accessible');pill.el.remove();
check(await a.engine.save()===before,'complete save unchanged');check(document.documentElement.scrollWidth<=innerWidth,'no overflow');return n;});console.log(width,checks,'persistent camp/class placement/worker shortcut PASS');await p.close();}}finally{await b.close();}
