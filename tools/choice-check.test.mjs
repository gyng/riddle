import {test} from 'node:test';import assert from 'node:assert/strict';import {spawnSync} from 'node:child_process';import {mkdtempSync,existsSync,writeFileSync,readFileSync,rmSync,mkdirSync} from 'node:fs';import {tmpdir} from 'node:os';import {join} from 'node:path';
const script=new URL('./choice-check.mjs',import.meta.url).pathname;
test('reject malformed upgrade paths before loading or building',()=>{
 const dir=mkdtempSync(join(tmpdir(),'riddle-choice-options-'));try{
  const input=join(dir,'source.json'),output=join(dir,'output');writeFileSync(input,'invalid JSON deliberately');
  for(const paths of ['health,health','unknown','health,','health:2']){
   const r=spawnSync(process.execPath,[script,input,'--choices','bold','--upgrades',paths,'--out',output],{encoding:'utf8'});
   assert.equal(r.status,1);assert.match(r.stderr,/unknown or duplicate upgrade path/);assert.equal(existsSync(output),false);assert.equal(readFileSync(input,'utf8'),'invalid JSON deliberately');
  }
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('expanded command preserves existing output and requires option values',()=>{
 const dir=mkdtempSync(join(tmpdir(),'riddle-choice-options-'));try{
  const input=join(dir,'missing.json'),output=join(dir,'output');mkdirSync(output);writeFileSync(join(output,'marker'),'preserve');
  const r=spawnSync(process.execPath,[script,input,'--choices','bold','--upgrades','all','--out',output],{encoding:'utf8'});assert.equal(r.status,1);assert.match(r.stderr,/already exists/);assert.equal(readFileSync(join(output,'marker'),'utf8'),'preserve');
  const missing=spawnSync(process.execPath,[script,input,'--choices','bold','--upgrades'],{encoding:'utf8'});assert.equal(missing.status,1);assert.match(missing.stderr,/missing value for --upgrades/);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
