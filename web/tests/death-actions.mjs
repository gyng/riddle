import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);const n=await p.evaluate(async()=>{
 const {deathAction}=await import('/src/ui/death.ts'),{rowChips}=await import('/src/ui/editor.ts'),{closeAllSheets}=await import('/src/ui/sheet.ts');let checks=0;const check=(ok,name)=>{if(!ok)throw Error(name);checks++;};
 const rows=[{conds:[{k:'foes>=',n:1}],verb:{v:'attack',a:'nearest'},origin:'stance:steady'},{conds:[{k:'hp<',n:30}],verb:{v:'drink',a:'heal'},origin:'stance:steady'}];
 const turn=(row,verb=rows[0].verb)=>({t:10,row,verb,hp:2,foes:1,telegraphs:[]});
 const base={run_id:1,depth:8,cause:'goblin_warlord',margin:'2 hp short',verdict:'gap',baseline:0,replays:12,patches:[],morgue:'',nothing_beats_base:true,trace:{turns:[turn(0),turn(-1,{v:'explore'})],blow:{t:11,by:'goblin_warlord',dmg:2,hp:0}},rules:{rows},package:'Steady · foes 1+ → attack',lever:{kind:'spend',text:'sword +1'}};
 check(deathAction(base)==='Steady · attack nearest','negative-row chores do not replace selected action');
 check(deathAction({...base,cause_row:1})==='Steady · drink heal at 30%','cause row wins over last trace action');
 check(deathAction({...base,cause_row:0,trace:{turns:[turn(1,rows[1].verb)]}})==='Steady · attack nearest','cause row zero is explicit');
 check(deathAction({...base,rules:undefined})==='Steady · attack nearest','missing rules use matching trace verb');
 check(deathAction({...base,cause_row:5,rules:undefined})==='Steady','missing selected-row history never invents another action');
 check(deathAction({...base,trace:{turns:[]},rules:undefined})==='Steady','missing history retains known source only');
 check(deathAction({...base,package:'drill · Warlord · foes 1+ → attack'})==='drill · Warlord · attack nearest','drill attribution retained');
 check(deathAction({...base,package:undefined})==='','no unsolicited package attribution');
 const a=window.__riddle;a.lineage.packages.pen_open=false;a.sets[a.active]={rows:[{conds:[],verb:{v:'return'}}]};check(a.rules.rows[0].verb.v==='return','camp rules really edited');a.go({kind:'death',death:base,kept:true});
 check(document.querySelector('.death-why').textContent==='Steady · attack nearest','historical death rules win over edited camp rules');
 const image=document.querySelector('.death-lever .item-pic');await image.decode();check(image.src.endsWith('/it_sword.png'),'recommendation uses sword silhouette');const icon=image.parentElement;check(getComputedStyle(icon).boxShadow==='none'&&getComputedStyle(icon).backgroundImage==='none','recommendation silhouette unframed');
 document.querySelector('.death-lever').click();check(!!document.querySelector('.sheet .forge'),'spend still opens Forge');closeAllSheets();
 const go=a.go.bind(a),mutate=a.mutate.bind(a),equip=a.engine.equipPackage,calls=[],dest=[];a.go=s=>dest.push(s.kind);a.mutate=async fn=>fn();a.engine.equipPackage=async(id,slot)=>{calls.push([id,slot]);return a.lineage;};
 const guard=a.lineage.packages.all.find(p=>p.id==='guarded');guard.owned=true;
 go({kind:'death',death:{...base,lever:{kind:'package',text:'Guarded'}},kept:true});check(document.querySelector('.death-lever img').src.endsWith('/pkg_guarded_v4.png'),'package recommendation uses expressive style');document.querySelector('.death-lever').click();await new Promise(r=>setTimeout(r,20));check(JSON.stringify(calls)==='[["guarded",0]]'&&dest.at(-1)==='camp','owned recommendation equips exact package/slot');
 guard.owned=false;go({kind:'death',death:{...base,lever:{kind:'package',text:'Guarded'}},kept:true});document.querySelector('.death-lever').click();check(calls.length===1&&dest.at(-1)==='camp','unowned recommendation never equips');
 go({kind:'death',death:{...base,lever:{kind:'wait',text:'Steady L2'}},kept:true});document.querySelector('.death-lever').click();check(dest.at(-1)==='camp','historical wait cannot send');
 go({kind:'death',death:{...base,lever:{kind:'wait',text:'Steady L2'}},kept:false});document.querySelector('.death-lever').click();check(dest.at(-1)==='watch','current wait can send');
 a.go=go;a.mutate=mutate;a.engine.equipPackage=equip;
 for(const [verb,kind] of [[{v:'throw',a:'fire,tag:boss'},'fire'],[{v:'drink',a:'unknown'},'potion'],[{v:'read',a:'unknown'},'scroll']]){const el=rowChips({conds:[],verb});check(el.querySelector('.item-ico').dataset.kind===kind,'targeted/unknown item family');}
 check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');return checks;
});console.log(width,n,'death action/illustration checks PASS');await p.close();}}finally{await b.close();}
