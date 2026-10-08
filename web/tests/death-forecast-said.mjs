// Blind c4705f9: (A) `forecast said D13 100%` over a D13 death read as a contradiction — the line counts the sends and says how many got
// past the floor; (B) a pre-pen gas death showed no trace — before the pen the screen is the cause and one lever (Cut 30 §2), never empty.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const n=await p.evaluate(async()=>{
  const {forecastSaidText}=await import('/src/ui/death.ts');let n=0;const check=(ok,m)=>{if(!ok)throw Error(m);n++;};
  const f={depths:[{depth:12,reach:1},{depth:13,reach:1},{depth:14,reach:0.625}],known_to:14,sims:8,low:13};
  check(forecastSaidText(f,13,1)==='forecast · reach D13 8/8 · past 5/8','reach counted with the floor\'s own odds: '+forecastSaidText(f,13,1));
  check(forecastSaidText(f,14,0.625)==='forecast · reach D14 5/8','frontier floor claims no past share');
  check(forecastSaidText({...f,sims:undefined},13,1)==='forecast said D13 100%','older core keeps the verbatim share');
  check(!/\b100%/.test(forecastSaidText(f,13,1)),'no bare 100% beside a death');
  const a=window.__riddle;
  const turn={t:10,row:0,verb:{v:'attack',a:'nearest'},hp:3,foes:0,telegraphs:[]};
  const base={run_id:1,depth:13,cause:'gas',margin:'',verdict:'dice',baseline:0.75,replays:12,lean:'dice',patches:[],morgue:'slain',nothing_beats_base:true,
   trace:{turns:[turn],blow:{t:11,by:'gas',dmg:3,hp:0}},package:'Guarded · bank at 25%'};
  // with the pen: the dice death's details say the counted forecast
  a.lineage.packages.pen_open=true;a.lastForecast=f;a.go({kind:'death',death:base});
  await new Promise(r=>setTimeout(r,200));
  const said=document.querySelector('.death .forecast-said')?.textContent;
  check(said==='forecast · reach D13 8/8 · past 5/8','death sheet counts the forecast: '+said);
  check(!!document.querySelector('.death .trace-panel'),'pen open: the trace is on the screen');
  // before the pen: the gas cause and exactly one lever (Cut 30 §2), never a bare banner
  a.lineage.packages.pen_open=false;a.go({kind:'death',death:{...base,lever:{kind:'spend',text:'pack +1'}}});
  await new Promise(r=>setTimeout(r,200));
  check(/gas · D13/.test(document.querySelector('.death .death-line')?.textContent??''),'pre-pen cause named');
  check(document.querySelectorAll('.death .death-lever').length===1,'pre-pen death offers its one lever');
  check(document.querySelector('.death .death-why')?.textContent?.startsWith('Guarded'),'pre-pen why names the package that acted');
  check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');
  return n;});
 console.log(width,n,'death forecast-said checks PASS');await p.close();
}}finally{await b.close();}
