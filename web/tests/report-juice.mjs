// Owner 2026-10-10 ("class level up matters · juice up while you were gone"): a level crossed is a highlight right after a boss slain
// (`Fighter L4`, its gain on hover, ≤ 3 highlights); the return's reveal (ui/report-juice.ts) never shifts layout, never holds the action,
// skips on a tap, ends within 2.5 s and marks `main.report[data-settled]`; reduced motion / juice off is settled at once.
import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
const base={elapsed_s:8*3600,runs:17,sampled:false,learned:[],bests:['D7','boss:rat_king'],found:[],deaths:[{cause:'rat',n:1}],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],salvaged:[],
  xp:{class:'fighter',gained:340,level_ups:1},renown:{gained:0,rank:0,ranks_up:0},gold:{home:1200,salvage:0,passage:0,wake:0,spent:0,net:1252},deepest:7,
  finds:{finds:[{id:'hearth_rug',name:'hearth rug',kind:'set'}],sealed:0},feats:[]};
let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;};
try{
  for(const juice of ['1','0']){
    const p=await b.newPage({viewport:{width:400,height:800}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
    await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=7&juice=${juice}&runclear=0`);await p.waitForFunction(()=>window.__riddle?.booted);
    const r=await p.evaluate((rep)=>{const a=window.__riddle;a.lineage.classes={...(a.lineage.classes??{}),fighter:{level:4,xp:10}};a.go({kind:'report',report:rep,absence:true});
      const main=document.querySelector('main.report'),hl=[...document.querySelectorAll('.report-sheet > .report-hl')].map(e=>e.dataset.hl);
      const rects=()=>[...document.querySelectorAll('.report-basics > .tile, .report-sheet > .report-hl, .collect-send')].map(e=>{let x=0,y=0;for(let o=e;o;o=o.offsetParent){x+=o.offsetLeft;y+=o.offsetTop;}return [x,y,e.offsetWidth,e.offsetHeight].join(",");}).join('|');
      return {settled:main.dataset.settled,hl,level:document.querySelector('.report-levelup')?.textContent.trim(),rects:rects(),t0:performance.now()};},base);
    check(r.hl[0]==='boss'&&r.hl[1]==='level'&&r.hl.length<=3,`level-up ranks right after the boss (${r.hl})`);
    check(/^Fighter\s+L4$/.test(r.level.replace(/\s+/g,' ')),`level-up reads Fighter L4 (${r.level})`);
    if(juice==='0'){check(r.settled==='1','juice off: settled at once');}
    else{
      check(r.settled==='0','juice on: the reveal runs');
      await p.waitForTimeout(250);
      const mid=await p.evaluate(()=>({rects:[...document.querySelectorAll('.report-basics > .tile, .report-sheet > .report-hl, .collect-send')].map(e=>{let x=0,y=0;for(let o=e;o;o=o.offsetParent){x+=o.offsetLeft;y+=o.offsetTop;}return [x,y,e.offsetWidth,e.offsetHeight].join(",");}).join('|'),
        text:document.querySelector('.report-basics .report-gold b').textContent,counting:document.querySelectorAll('.rj-count').length}));
      check(mid.rects===r.rects,`no layout shift mid-reveal ${r.rects} vs ${mid.rects}`);check(mid.text.includes('+$1252'),'the DOM text is the truth while counting');check(mid.counting>0,'tiles count');
      await p.waitForFunction(()=>document.querySelector('main.report')?.dataset.settled==='1',null,{timeout:4000});
      const dt=await p.evaluate((t0)=>performance.now()-t0,r.t0);check(dt<=2700,`settles within 2.5 s (${Math.round(dt)} ms)`);
      const end=await p.evaluate(()=>({breathe:!!document.querySelector('.collect-send.rj-breathe'),wait:document.querySelectorAll('.rj-wait, .rj-count').length,coins:document.querySelectorAll('.rj-coins').length}));
      check(end.breathe&&end.wait===0&&end.coins===0,'end state: the action breathes, nothing waits, the coins gone');
      // a tap anywhere skips; the action works from the first frame
      await p.evaluate((rep)=>window.__riddle.go({kind:'report',report:rep,absence:true}),base);
      await p.waitForTimeout(80);
      await p.mouse.click(20,400);
      check(await p.evaluate(()=>document.querySelector('main.report').dataset.settled==='1'&&!document.querySelector('.rj-wait, .rj-count')),'a tap skips to the end state');
      await p.evaluate((rep)=>window.__riddle.go({kind:'report',report:rep,absence:true}),base);
      await p.waitForTimeout(60);
      check(await p.evaluate(()=>{const g=document.querySelector('.collect-go');return !!g&&!g.disabled&&getComputedStyle(g).pointerEvents!=='none';}),'collect & send live at once');
    }
    if(errors.length)throw Error(errors.join('\n'));await p.close();
  }
  console.log(`${n} level-up highlight/reveal timing/no shift/skip/settled checks PASS`);
}finally{await b.close();}
