#!/usr/bin/env node
// Paired headed-WASM catch-up with exact report/save comparisons. Diagnostic, not a gate.
import {launchGpu} from './browser.mjs';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const [original,candidate,out,...saves]=process.argv.slice(2);
assert.ok(original&&candidate&&out&&saves.length,'compare-catchup.mjs ORIGINAL_PKG CANDIDATE_PKG OUT SAVE...');
const hours=Number(process.env.CATCHUP_HOURS??8),pairs=Number(process.env.CATCHUP_PAIRS??7);
const plateauEstimate=process.env.CATCHUP_ALLOW_PLATEAU_ESTIMATE==='1';
assert.ok(Number.isSafeInteger(hours)&&hours>0&&Number.isSafeInteger(pairs)&&pairs>0);
const sha=b=>createHash('sha256').update(b).digest('hex'),files={};
for(const [id,path] of [['original',original],['candidate',candidate]]){
 files[`/${id}.mjs`]={contentType:'text/javascript',body:readFileSync(join(path,'riddle_wasm.js'))};
 files[`/${id}.wasm`]={contentType:'application/wasm',body:readFileSync(join(path,'riddle_wasm_bg.wasm'))};
}
mkdirSync(out,{recursive:true});
const browser=await launchGpu(),rows=[];
try{const page=await browser.newPage();await page.route('http://riddle.compare.invalid/**',r=>r.fulfill(files[new URL(r.request().url()).pathname]??{contentType:'text/html',body:'<!doctype html><title>Catch-up comparison</title>'}));await page.goto('http://riddle.compare.invalid/');
 await page.evaluate(async()=>{window.versions={};for(const id of ['original','candidate']){const m=await import(`/${id}.mjs`);await m.default({module_or_path:`/${id}.wasm`});window.versions[id]=m;}});
 for(const path of saves){const input=readFileSync(path,'utf8');
 const measured=await page.evaluate(({input,seconds,pairs,plateauEstimate})=>{
  const run=id=>{const game=new window.versions[id].Game(1);try{game.load(input);const t=performance.now(),report=game.runOfflineQuick(seconds),elapsed=(performance.now()-t)/1000;return{seconds:elapsed,report,save:game.save()};}finally{game.free();}};
  for(const id of ['original','candidate'])run(id);const rows=[],expected={};let common;
  const comparable=r=>{if(!plateauEstimate)return r.report;const parsed=JSON.parse(r.report);if(parsed.stall)delete parsed.stall.patches;return JSON.stringify(parsed);};
  for(let n=0;n<pairs;n++){const pair={};for(const id of n%2?['candidate','original']:['original','candidate']){const r=run(id),same=comparable(r);
   if(expected[id]&&(r.report!==expected[id].report||r.save!==expected[id].save))throw Error(`repeat differs: ${id} pair ${n+1}`);
   if(common&&(same!==common.report||r.save!==common.save))throw Error(`outcome differs beyond allowed estimate: ${id} pair ${n+1}`);
   common??={report:same,save:r.save};expected[id]??=r;pair[id]=r.seconds;}rows.push(pair);}
  return{pairs:rows,report:expected.original.report,candidateReport:expected.candidate.report,save:expected.original.save};
 },{input,seconds:hours*3600,pairs,plateauEstimate});
 const median=xs=>[...xs].sort((a,b)=>a-b)[Math.floor(xs.length/2)];const a=median(measured.pairs.map(p=>p.original)),b=median(measured.pairs.map(p=>p.candidate));
 if(process.env.CATCHUP_RECORD_OUTPUT==='1')writeFileSync(join(out,`${rows.length}-output.json`),JSON.stringify({report:measured.report,save:measured.save}));
 const row={fixture:resolve(path),fixtureSha256:sha(input),pairs:measured.pairs,originalMedian:a,candidateMedian:b,gainPercent:(1-b/a)*100,reportSha256:sha(measured.report),saveSha256:sha(measured.save),...(plateauEstimate?{candidateReportSha256:sha(measured.candidateReport),originalPlateau:JSON.parse(measured.report).stall,candidatePlateau:JSON.parse(measured.candidateReport).stall}:{})};rows.push(row);console.log(JSON.stringify(row));}
 mkdirSync(out,{recursive:true});writeFileSync(join(out,'comparison.json'),JSON.stringify({hours,pairs,plateauEstimate,browserVersion:browser.version(),originalWasmSha256:sha(files['/original.wasm'].body),candidateWasmSha256:sha(files['/candidate.wasm'].body),harnessSha256:sha(readFileSync(new URL(import.meta.url))),scope:`Alternating engine-call timings; compilation/load/save serialization excluded. Exact raw save and per-version repeat equality; ${plateauEstimate?'only report stall.patches may differ across versions':'exact raw report equality across versions'}. Fast vs shipping depends on supplied artifacts; no app/FPS claim.`,rows},null,2)+'\n');
}finally{await browser.close();}
